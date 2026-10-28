#!/bin/bash
# shot.sh OUT THEME W H CMD...: run CMD on the headless desktop as a floating
# window W x H in THEME, screenshot it to OUT, stop it.
out=$1; theme=$2; w=$3; h=$4; shift 4
export WAYLAND_DISPLAY=wayland-1 XDG_RUNTIME_DIR=/run/user/$(id -u) VINO_UI_THEME=$theme SWAYSOCK=$(ls /run/user/$(id -u)/sway-ipc.* | head -1)
"$@" >/dev/null 2>&1 & pid=$!
for i in $(seq 1 40); do
  sleep 0.25
  swaymsg -t get_tree | python3 -c "import json,sys; t=json.load(sys.stdin)
def walk(n):
    if n.get('pid')==$pid and n.get('type') in ('con','floating_con'): return n
    for c in n.get('nodes',[])+n.get('floating_nodes',[]):
        r=walk(c)
        if r: return r
print('found' if walk(t) else '')" | grep -q found && break
done
swaymsg "[pid=$pid] floating enable" >/dev/null; sleep 0.3; swaymsg "[pid=$pid] border none" >/dev/null; swaymsg "[pid=$pid] resize set width $w px height $h px" >/dev/null; sleep 0.3; swaymsg "[pid=$pid] move position 40 40" >/dev/null
sleep 1.5
geom=$(swaymsg -t get_tree | python3 -c "import json,sys; t=json.load(sys.stdin)
def walk(n):
    if n.get('pid')==$pid and n.get('type') in ('con','floating_con'): return n
    for c in n.get('nodes',[])+n.get('floating_nodes',[]):
        r=walk(c)
        if r: return r
n=walk(t); r=n['rect']; print(f\"{r['x']},{r['y']} {r['width']}x{r['height']}\")")
grim -g "$geom" "$out"
kill $pid 2>/dev/null; wait $pid 2>/dev/null
echo "$out $geom"
