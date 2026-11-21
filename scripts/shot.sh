#!/bin/bash
# shot.sh OUT THEME W H CMD...: run CMD on the headless desktop as a floating
# window W x H in THEME, screenshot it to OUT, stop it.
set -euo pipefail

out="${1:?missing output path}"
theme="${2:?missing theme}"
w="${3:?missing width}"
h="${4:?missing height}"
shift 4

if ! [[ "$w" =~ ^[0-9]+$ ]] || ! [[ "$h" =~ ^[0-9]+$ ]]; then
    echo "width and height must be positive integers" >&2
    exit 1
fi

uid="$(id -u)"
runtime_dir="/run/user/$uid"
shopt -s nullglob
socks=("$runtime_dir"/sway-ipc.*)
shopt -u nullglob
if [ "${#socks[@]}" -eq 0 ]; then
    echo "no sway IPC socket found in $runtime_dir" >&2
    exit 1
fi

export WAYLAND_DISPLAY=wayland-1
export XDG_RUNTIME_DIR="$runtime_dir"
export VINO_UI_THEME="$theme"
export SWAYSOCK="${socks[0]}"

"$@" >/dev/null 2>&1 &
pid=$!

for _ in $(seq 1 40); do
  sleep 0.25
  if swaymsg -t get_tree | python3 -c 'import json,sys
pid = int(sys.argv[1])
t = json.load(sys.stdin)
def walk(n):
    if n.get("pid") == pid and n.get("type") in ("con", "floating_con"):
        return n
    for c in n.get("nodes", []) + n.get("floating_nodes", []):
        r = walk(c)
        if r:
            return r
print("found" if walk(t) else "")' "$pid" | grep -q found; then
    break
  fi
done

swaymsg "[pid=$pid] floating enable" >/dev/null
sleep 0.3
swaymsg "[pid=$pid] border none" >/dev/null
swaymsg "[pid=$pid] resize set width $w px height $h px" >/dev/null
sleep 0.3
swaymsg "[pid=$pid] move position 40 40" >/dev/null
sleep 1.5

geom="$(swaymsg -t get_tree | python3 -c 'import json,sys
pid = int(sys.argv[1])
t = json.load(sys.stdin)
def walk(n):
    if n.get("pid") == pid and n.get("type") in ("con", "floating_con"):
        return n
    for c in n.get("nodes", []) + n.get("floating_nodes", []):
        r = walk(c)
        if r:
            return r
n = walk(t)
r = n["rect"]
print(f"{r[\"x\"]},{r[\"y\"]} {r[\"width\"]}x{r[\"height\"]}")' "$pid")"

grim -g "$geom" "$out"
kill "$pid" 2>/dev/null || true
wait "$pid" 2>/dev/null || true
echo "$out $geom"
