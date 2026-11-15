//! A 16 px stroke icon set, painted from SVG path data on a 24-unit grid:
//! round caps and joins, 1.8 units wide, in the text's colour. Painted rather
//! than a font so an icon is one shape, scales with the window and never
//! falls back to a missing-glyph box.

use egui::{Color32, Painter, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2, pos2, vec2};

use crate::a11y;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Icon {
    Search,
    Folder,
    Lock,
    Model,
    Hardware,
    Integration,
    Index,
    Plus,
    Excluded,
    File,
    FileText,
    Image,
    ChevronDown,
    Clock,
    Play,
    Stop,
    Voice,
    Speaker,
    Keyboard,
    Captions,
    Mic,
    Cleanup,
    Sessions,
    Settings,
    Archive,
    More,
    ArrowLeft,
    Warning,
    Error,
    Info,
    Download,
    Flask,
    Key,
    Refresh,
    Check,
    Face,
    User,
    Motion,
    Sliders,
}

impl Icon {
    /// Every icon, in the order above.
    pub const ALL: [Icon; 39] = [
        Icon::Search,
        Icon::Folder,
        Icon::Lock,
        Icon::Model,
        Icon::Hardware,
        Icon::Integration,
        Icon::Index,
        Icon::Plus,
        Icon::Excluded,
        Icon::File,
        Icon::FileText,
        Icon::Image,
        Icon::ChevronDown,
        Icon::Clock,
        Icon::Play,
        Icon::Stop,
        Icon::Voice,
        Icon::Speaker,
        Icon::Keyboard,
        Icon::Captions,
        Icon::Mic,
        Icon::Cleanup,
        Icon::Sessions,
        Icon::Settings,
        Icon::Archive,
        Icon::More,
        Icon::ArrowLeft,
        Icon::Warning,
        Icon::Error,
        Icon::Info,
        Icon::Download,
        Icon::Flask,
        Icon::Key,
        Icon::Refresh,
        Icon::Check,
        Icon::Face,
        Icon::User,
        Icon::Motion,
        Icon::Sliders,
    ];

    /// The SVG path data, in a 24 × 24 box.
    pub fn path(self) -> &'static str {
        match self {
            Icon::Search => "M11 4a7 7 0 1 0 0 14a7 7 0 1 0 0-14 m9 16-3.5-3.5",
            Icon::Folder => {
                "M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"
            }
            Icon::Lock => "M5 11h14v10H5z M8 11V7a4 4 0 0 1 8 0v4",
            Icon::Model => "M12 3 4 7.5v9L12 21l8-4.5v-9z M4 7.5 12 12l8-4.5M12 12v9",
            Icon::Hardware => {
                "M6 6h12v12H6z M9.5 9.5h5v5h-5z M9 2v3M15 2v3M9 19v3M15 19v3M2 9h3M2 15h3M19 9h3M19 15h3"
            }
            Icon::Integration => "M9 2v5M15 2v5M6 7h12v4a6 6 0 0 1-12 0zM12 17v5",
            Icon::Index => "M3 12h4l3-8 4 16 3-8h4",
            Icon::Plus => "M12 5v14M5 12h14",
            Icon::Excluded => "M12 3a9 9 0 1 0 0 18a9 9 0 1 0 0-18 m-6.4 2.6 12.8 12.8",
            Icon::File => "M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z M14 3v5h5",
            Icon::FileText => {
                "M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z M14 3v5h5M9 13h6M9 17h6"
            }
            Icon::Image => "M3 4h18v16H3z M9 8a2 2 0 1 0 0 4a2 2 0 1 0 0-4 m12 8-5-5-9 9",
            Icon::ChevronDown => "m6 9 6 6 6-6",
            Icon::Clock => "M12 3a9 9 0 1 0 0 18a9 9 0 1 0 0-18 M12 7v5l3 2",
            Icon::Play => "M8 5v14l11-7z",
            Icon::Stop => "M6 6h12v12H6z",
            Icon::Voice => "M12 4a4 4 0 1 0 0 8a4 4 0 1 0 0-8 M4 21a8 8 0 0 1 16 0",
            Icon::Speaker => "M11 5 6 9H3v6h3l5 4z M16 9a4 4 0 0 1 0 6",
            Icon::Keyboard => "M2 6h20v12H2z M6 10h.01M10 10h.01M14 10h.01M18 10h.01M7 14h10",
            Icon::Captions => "M3 5h18v14H3z M7 11h4M13 11h4M7 15h7",
            Icon::Mic => "M9 3h6v11H9z M5 11a7 7 0 0 0 14 0M12 18v3",
            Icon::Cleanup => {
                "M20 20H9l-5-5 9.5-9.5a2 2 0 0 1 2.8 0l3.2 3.2a2 2 0 0 1 0 2.8L11 20 m-2.5-9.5 5 5"
            }
            Icon::Sessions => {
                "M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z M14 3v5h5M9 13h6M9 17h6"
            }
            Icon::Settings => {
                "M12 9a3 3 0 1 0 0 6a3 3 0 1 0 0-6 M12 2v3M12 19v3M2 12h3M19 12h3M4.9 4.9l2.1 2.1M17 17l2.1 2.1M4.9 19.1 7 17M17 7l2.1-2.1"
            }
            Icon::Archive => "M21 8v12H3V8M1 3h22v5H1zM10 12h4",
            Icon::More => "M5 12h.01M12 12h.01M19 12h.01",
            Icon::ArrowLeft => "M19 12H5M11 6l-6 6 6 6",
            Icon::Warning => "M12 3 2 20h20z M12 10v4M12 17h.01",
            Icon::Error => "M12 3a9 9 0 1 0 0 18a9 9 0 1 0 0-18 M12 8v5M12 16h.01",
            Icon::Info => "M12 3a9 9 0 1 0 0 18a9 9 0 1 0 0-18 M12 11v5M12 8h.01",
            Icon::Download => "M12 4v11M7 10l5 5 5-5M5 20h14",
            Icon::Flask => {
                "M9 3h6M10 3v6l-5 9a2 2 0 0 0 1.7 3h10.6a2 2 0 0 0 1.7-3l-5-9V3 M7.5 15h9"
            }
            Icon::Key => "M8 11a4 4 0 1 0 0 8a4 4 0 1 0 0-8 m3 1 9-9M17 6l3 3",
            Icon::Refresh => {
                "M20 11a8 8 0 0 0-14.5-4.5L4 8M4 3v5h5 M4 13a8 8 0 0 0 14.5 4.5L20 16M20 21v-5h-5"
            }
            Icon::Check => "m5 12 5 5 9-10",
            // A face inside scan corners: face unlock, enrolment.
            Icon::Face => {
                "M4 8V6a2 2 0 0 1 2-2h2M16 4h2a2 2 0 0 1 2 2v2M20 16v2a2 2 0 0 1-2 2h-2M8 20H6a2 2 0 0 1-2-2v-2 \
                 M9 9.5h.01M15 9.5h.01M12 10v3h-1M9 15.5a4 4 0 0 0 6 0"
            }
            // Someone's own settings: a person in a circle, set apart from
            // `Voice`, which is a speaker.
            Icon::User => {
                "M12 3a9 9 0 1 0 0 18a9 9 0 1 0 0-18 M12 7a3 3 0 1 0 0 6a3 3 0 1 0 0-6 M6.6 18.2a6.5 6.5 0 0 1 10.8 0"
            }
            // A head moving: liveness by motion.
            Icon::Motion => {
                "M14 5a7 7 0 1 0 0 14a7 7 0 1 0 0-14 M11.5 10.5h.01M16.5 10.5h.01 M2 9h3M1.5 12h3M2 15h3"
            }
            // Thresholds and other tuning.
            Icon::Sliders => {
                "M4 6h8M16 6h4M14 4v4 M4 12h2M10 12h10M8 10v4 M4 18h10M18 18h2M16 16v4"
            }
        }
    }

    /// Where the three dots of `More` are heavier than the rest.
    fn stroke_scale(self) -> f32 {
        match self {
            Icon::More => 3.0 / 1.8,
            _ => 1.0,
        }
    }
}

/// The icon in a `size` square, taking its space in the layout; hidden from
/// screen readers, since the text beside it says what it is for.
pub fn icon(ui: &mut Ui, icon: Icon, size: f32, color: Color32) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
    a11y::decorative(ui, response.id);
    if ui.is_rect_visible(rect) {
        paint(ui.painter(), rect, icon, color);
    }
    response
}

/// The icon painted to fill `rect`.
pub fn paint(painter: &Painter, rect: Rect, icon: Icon, color: Color32) {
    paint_path(painter, rect, icon.path(), 1.8 * icon.stroke_scale(), color);
}

/// Any path on the icons' 24-unit grid, `width` units wide: for shapes that
/// aren't icons of their own, like the window buttons.
pub(crate) fn paint_path(painter: &Painter, rect: Rect, path: &str, width: f32, color: Color32) {
    let k = rect.width().min(rect.height()) / 24.0;
    let origin = rect.center() - vec2(12.0, 12.0) * k;
    let width = width * k;
    let stroke = Stroke::new(width, color);
    for sub in parse(path) {
        let points: Vec<Pos2> = sub
            .points
            .iter()
            .map(|p| origin + p.to_vec2() * k)
            .collect();
        match points.len() {
            0 => {}
            // "h.01": a dot, drawn as the stroke's round cap alone.
            1 => {
                painter.circle_filled(points[0], width / 2.0, color);
            }
            _ if sub.closed => {
                painter.add(Shape::closed_line(points, stroke));
            }
            _ => {
                // Round caps: egui's strokes end square.
                painter.circle_filled(points[0], width / 2.0, color);
                painter.circle_filled(points[points.len() - 1], width / 2.0, color);
                painter.add(Shape::line(points, stroke));
            }
        }
    }
}

/// One subpath as points, flattened.
#[derive(Debug, Clone, PartialEq)]
pub struct Subpath {
    pub points: Vec<Pos2>,
    pub closed: bool,
}

/// SVG path data (M, L, H, V, C, Q, A, Z and their relative forms, with
/// implicit repeats) as flattened subpaths. A dot drawn as a zero-length
/// segment keeps its one point. What it can't read ends the path there: S and
/// T, exponents, and arc flags written without a space after them ("0 01").
pub fn parse(path: &str) -> Vec<Subpath> {
    let mut tokens = Tokens::new(path);
    let mut pen = Pen::default();
    let mut command = 'M';
    while !tokens.done() {
        if let Some(named) = tokens.command() {
            command = named;
        }
        match segment(command, &mut tokens, &mut pen) {
            Some(repeat) => command = repeat,
            None => break,
        }
    }
    pen.finish()
}

/// Draws one `command` with the numbers that follow it. Some: the command
/// further numbers repeat. None: the path ends here, short of numbers or at
/// a letter that isn't a command.
fn segment(command: char, tokens: &mut Tokens, pen: &mut Pen) -> Option<char> {
    let relative = command.is_ascii_lowercase();
    let line_to = if relative { 'l' } else { 'L' };
    let from = pen.pos;
    let at = |x: f32, y: f32| {
        if relative {
            from + vec2(x, y)
        } else {
            pos2(x, y)
        }
    };
    match command.to_ascii_uppercase() {
        'M' => {
            let [x, y] = tokens.numbers()?;
            pen.move_to(at(x, y));
            // Further pairs are line-tos.
            return Some(line_to);
        }
        'L' => {
            let [x, y] = tokens.numbers()?;
            pen.line_to(at(x, y));
        }
        'H' => {
            let x = tokens.number()?;
            pen.line_to(at(x, if relative { 0.0 } else { from.y }));
        }
        'V' => {
            let y = tokens.number()?;
            pen.line_to(at(if relative { 0.0 } else { from.x }, y));
        }
        'C' => {
            let [x1, y1, x2, y2, x, y] = tokens.numbers()?;
            let (c1, c2, end) = (at(x1, y1), at(x2, y2), at(x, y));
            pen.curve_to(end, |t| cubic(from, c1, c2, end, t));
        }
        'Q' => {
            let [x1, y1, x, y] = tokens.numbers()?;
            let (c, end) = (at(x1, y1), at(x, y));
            pen.curve_to(end, |t| quadratic(from, c, end, t));
        }
        'A' => {
            let [rx, ry, rotation, large, sweep, x, y] = tokens.numbers()?;
            let end = at(x, y);
            arc(
                &mut pen.current,
                from,
                rx,
                ry,
                rotation,
                large != 0.0,
                sweep != 0.0,
                end,
            );
            pen.pos = end;
        }
        'Z' => {
            // A Z followed by numbers draws on from the start point.
            pen.close(tokens.at_number());
            return Some(line_to);
        }
        _ => return None,
    }
    Some(command)
}

/// The pen: where it is, the subpath it is drawing and the ones it has drawn.
#[derive(Default)]
struct Pen {
    done: Vec<Subpath>,
    current: Vec<Pos2>,
    pos: Pos2,
    start: Pos2,
}

impl Pen {
    fn move_to(&mut self, to: Pos2) {
        self.end_subpath(false);
        self.start = to;
        self.line_to(to);
    }

    fn line_to(&mut self, to: Pos2) {
        self.pos = to;
        self.current.push(to);
    }

    /// A curve to `end` as `CURVE_STEPS` chords; `point` is the curve from 0 to 1.
    fn curve_to(&mut self, end: Pos2, point: impl Fn(f32) -> Pos2) {
        for step in 1..=CURVE_STEPS {
            self.current.push(point(step as f32 / CURVE_STEPS as f32));
        }
        self.pos = end;
    }

    fn close(&mut self, draws_on: bool) {
        self.pos = self.start;
        self.end_subpath(true);
        if draws_on {
            self.current.push(self.pos);
        }
    }

    fn end_subpath(&mut self, closed: bool) {
        if self.current.is_empty() {
            return;
        }
        let mut points = std::mem::take(&mut self.current);
        if closed && points.len() >= 2 && points.last() == points.first() {
            points.pop();
        }
        // "M12 12h0" and "h.01" are dots.
        if points.len() == 2 && points[0].distance(points[1]) < DOT {
            points.pop();
        }
        self.done.push(Subpath { points, closed });
    }

    fn finish(mut self) -> Vec<Subpath> {
        self.end_subpath(false);
        self.done
    }
}

/// How many chords a C or a Q is flattened into.
const CURVE_STEPS: usize = 8;
/// The longest line that is drawn as a dot, in the path's units.
const DOT: f32 = 0.05;
/// The most an arc turns in one chord.
const ARC_STEP: f32 = std::f32::consts::PI / 8.0;

fn cubic(from: Pos2, c1: Pos2, c2: Pos2, end: Pos2, t: f32) -> Pos2 {
    let u = 1.0 - t;
    let p = from.to_vec2() * (u * u * u)
        + c1.to_vec2() * (3.0 * u * u * t)
        + c2.to_vec2() * (3.0 * u * t * t)
        + end.to_vec2() * (t * t * t);
    p.to_pos2()
}

fn quadratic(from: Pos2, c: Pos2, end: Pos2, t: f32) -> Pos2 {
    let u = 1.0 - t;
    let p = from.to_vec2() * (u * u) + c.to_vec2() * (2.0 * u * t) + end.to_vec2() * (t * t);
    p.to_pos2()
}

/// An SVG elliptical arc, flattened onto `into`.
#[allow(clippy::too_many_arguments)]
fn arc(
    into: &mut Vec<Pos2>,
    from: Pos2,
    rx: f32,
    ry: f32,
    rotation: f32,
    large: bool,
    sweep: bool,
    to: Pos2,
) {
    let Some(arc) = CentreArc::from_endpoints(from, rx, ry, rotation, large, sweep, to) else {
        into.push(to);
        return;
    };
    let steps = arc.steps();
    for step in 1..steps {
        into.push(arc.point(arc.theta + arc.delta * step as f32 / steps as f32));
    }
    // Land exactly where the path says.
    into.push(to);
}

/// An arc by its centre: the ellipse, its tilt, the angle it starts at and
/// how far round it turns (negative is anticlockwise on screen).
#[derive(Debug, Clone, Copy, PartialEq)]
struct CentreArc {
    centre: Pos2,
    rx: f32,
    ry: f32,
    sin: f32,
    cos: f32,
    theta: f32,
    delta: f32,
}

impl CentreArc {
    /// From SVG's two endpoints and flags (SVG 1.1 appendix F.6.5). None for
    /// what SVG draws as a straight line: a zero radius, or nowhere to go.
    fn from_endpoints(
        from: Pos2,
        rx: f32,
        ry: f32,
        rotation: f32,
        large: bool,
        sweep: bool,
        to: Pos2,
    ) -> Option<Self> {
        if rx == 0.0 || ry == 0.0 || from == to {
            return None;
        }
        let (mut rx, mut ry) = (rx.abs(), ry.abs());
        let (sin, cos) = rotation.to_radians().sin_cos();
        let d = (from - to) / 2.0;
        let x1 = cos * d.x + sin * d.y;
        let y1 = -sin * d.x + cos * d.y;
        // Radii too small to reach are scaled up until they do.
        let lambda = (x1 * x1) / (rx * rx) + (y1 * y1) / (ry * ry);
        if lambda > 1.0 {
            rx *= lambda.sqrt();
            ry *= lambda.sqrt();
        }
        let num = (rx * rx * ry * ry - rx * rx * y1 * y1 - ry * ry * x1 * x1).max(0.0);
        let den = rx * rx * y1 * y1 + ry * ry * x1 * x1;
        let mut coef = if den == 0.0 { 0.0 } else { (num / den).sqrt() };
        if large == sweep {
            coef = -coef;
        }
        let cx1 = coef * rx * y1 / ry;
        let cy1 = -coef * ry * x1 / rx;
        let mid = (from.to_vec2() + to.to_vec2()) / 2.0;
        let centre = pos2(cos * cx1 - sin * cy1 + mid.x, sin * cx1 + cos * cy1 + mid.y);
        let theta = angle_between(1.0, 0.0, (x1 - cx1) / rx, (y1 - cy1) / ry);
        let mut delta = angle_between(
            (x1 - cx1) / rx,
            (y1 - cy1) / ry,
            (-x1 - cx1) / rx,
            (-y1 - cy1) / ry,
        );
        if !sweep && delta > 0.0 {
            delta -= std::f32::consts::TAU;
        } else if sweep && delta < 0.0 {
            delta += std::f32::consts::TAU;
        }
        Some(CentreArc {
            centre,
            rx,
            ry,
            sin,
            cos,
            theta,
            delta,
        })
    }

    fn steps(&self) -> usize {
        ((self.delta.abs() / ARC_STEP).ceil() as usize).max(2)
    }

    /// The point at `angle` round the ellipse.
    fn point(&self, angle: f32) -> Pos2 {
        let (st, ct) = angle.sin_cos();
        pos2(
            self.centre.x + self.cos * self.rx * ct - self.sin * self.ry * st,
            self.centre.y + self.sin * self.rx * ct + self.cos * self.ry * st,
        )
    }
}

/// The signed angle from (ux, uy) round to (vx, vy).
fn angle_between(ux: f32, uy: f32, vx: f32, vy: f32) -> f32 {
    let dot = ux * vx + uy * vy;
    let len = (ux * ux + uy * uy).sqrt() * (vx * vx + vy * vy).sqrt();
    let a = (dot / len).clamp(-1.0, 1.0).acos();
    if ux * vy - uy * vx < 0.0 { -a } else { a }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Token {
    Command(char),
    Number(f32),
}

/// A path's tokens, read from the front.
struct Tokens {
    tokens: Vec<Token>,
    next: usize,
}

impl Tokens {
    fn new(path: &str) -> Self {
        Tokens {
            tokens: tokenize(path),
            next: 0,
        }
    }

    fn done(&self) -> bool {
        self.next >= self.tokens.len()
    }

    fn at_number(&self) -> bool {
        matches!(self.tokens.get(self.next), Some(Token::Number(_)))
    }

    /// The command letter here, if this is one.
    fn command(&mut self) -> Option<char> {
        let Token::Command(c) = *self.tokens.get(self.next)? else {
            return None;
        };
        self.next += 1;
        Some(c)
    }

    fn number(&mut self) -> Option<f32> {
        let Token::Number(n) = *self.tokens.get(self.next)? else {
            return None;
        };
        self.next += 1;
        Some(n)
    }

    /// The next `N` numbers, or None when a command or the end comes first.
    fn numbers<const N: usize>(&mut self) -> Option<[f32; N]> {
        let mut out = [0.0; N];
        for slot in &mut out {
            *slot = self.number()?;
        }
        Some(out)
    }
}

fn tokenize(path: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let bytes = path.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c.is_ascii_alphabetic() {
            out.push(Token::Command(c));
            i += 1;
        } else if c.is_ascii_digit() || c == '-' || c == '.' || c == '+' {
            let start = i;
            i += 1;
            while i < bytes.len() {
                let d = bytes[i] as char;
                // A second '.' or a '-' starts the next number: "4.5.5", "4-4".
                if d.is_ascii_digit() || (d == '.' && !path[start..i].contains('.')) {
                    i += 1;
                } else {
                    break;
                }
            }
            if let Ok(n) = path[start..i].parse::<f32>() {
                out.push(Token::Number(n));
            }
        } else {
            i += 1;
        }
    }
    out
}

/// The icon's size for a body line.
pub const SIZE: f32 = 16.0;

/// Both in one square.
pub fn size(size: f32) -> Vec2 {
    vec2(size, size)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    use kurbo::{ParamCurve, ParamCurveNearest, PathEl};

    /// Our points are on the true curve; kurbo's arcs are cubics within 0.1.
    const ON_THE_PATH: f64 = 0.11;
    /// The furthest a chord of ours strays from its curve: a 9-unit arc's
    /// sagitta over `ARC_STEP` is 0.17.
    const NEAR_OUR_CHORDS: f32 = 0.3;

    fn distance_to_chord(p: Pos2, a: Pos2, b: Pos2) -> f32 {
        let along = b - a;
        let t = if along.length_sq() == 0.0 {
            0.0
        } else {
            ((p - a).dot(along) / along.length_sq()).clamp(0.0, 1.0)
        };
        p.distance(a + along * t)
    }

    fn distance_to_subpaths(p: Pos2, subs: &[Subpath]) -> f32 {
        let mut least = f32::INFINITY;
        for sub in subs {
            let n = sub.points.len();
            let chords = if sub.closed { n } else { n.saturating_sub(1) };
            for i in 0..chords.max(1) {
                let (a, b) = (sub.points[i], sub.points[(i + 1) % n]);
                least = least.min(distance_to_chord(p, a, b));
            }
        }
        least
    }

    /// `parse` against kurbo's SVG path reader, which this crate links only
    /// for tests: the same subpaths, every point of ours on kurbo's path, and
    /// no stretch of kurbo's path that ours leaves out.
    pub(crate) fn assert_reads_as_kurbo_does(path: &str) {
        let ours = parse(path);
        let reference =
            kurbo::BezPath::from_svg(path).unwrap_or_else(|e| panic!("kurbo on {path:?}: {e}"));
        let count = |wanted: fn(&PathEl) -> bool| {
            reference.elements().iter().filter(|el| wanted(el)).count()
        };
        assert_eq!(
            ours.len(),
            count(|el| matches!(el, PathEl::MoveTo(_))),
            "subpaths of {path:?}"
        );
        assert_eq!(
            ours.iter().filter(|sub| sub.closed).count(),
            count(|el| matches!(el, PathEl::ClosePath)),
            "closed subpaths of {path:?}"
        );
        let segments: Vec<kurbo::PathSeg> = reference.segments().collect();
        for point in ours.iter().flat_map(|sub| &sub.points) {
            let at = kurbo::Point::new(f64::from(point.x), f64::from(point.y));
            let off = segments
                .iter()
                .map(|seg| seg.nearest(at, 1e-6).distance_sq.sqrt())
                .fold(f64::INFINITY, f64::min);
            assert!(
                off <= ON_THE_PATH,
                "{point:?} is {off:.3} off kurbo's {path:?}"
            );
        }
        for seg in &segments {
            for t in [0.0, 0.5, 1.0] {
                let on = seg.eval(t);
                let off = distance_to_subpaths(pos2(on.x as f32, on.y as f32), &ours);
                assert!(
                    off <= NEAR_OUR_CHORDS,
                    "{on:?} of kurbo's {path:?} is {off:.3} from ours"
                );
            }
        }
    }

    fn numbers(path: &str) -> Vec<f32> {
        tokenize(path)
            .into_iter()
            .map(|token| match token {
                Token::Number(n) => n,
                Token::Command(c) => panic!("{c} in {path:?}"),
            })
            .collect()
    }

    #[test]
    fn numbers_split_where_svg_splits_them() {
        assert_eq!(numbers("6 9,6\t6\n6"), [6.0, 9.0, 6.0, 6.0, 6.0]);
        assert_eq!(numbers("6-6"), [6.0, -6.0], "a sign starts a number");
        assert_eq!(numbers("-3.5-3.5"), [-3.5, -3.5]);
        assert_eq!(numbers("4.5.5"), [4.5, 0.5], "and so does a second dot");
        assert_eq!(numbers(".01 +2"), [0.01, 2.0]);
        assert_eq!(numbers("- 5"), [5.0], "a sign alone is dropped");
    }

    #[test]
    fn letters_are_commands_and_keep_their_case() {
        assert_eq!(
            tokenize("M12 5v14h.01z"),
            [
                Token::Command('M'),
                Token::Number(12.0),
                Token::Number(5.0),
                Token::Command('v'),
                Token::Number(14.0),
                Token::Command('h'),
                Token::Number(0.01),
                Token::Command('z'),
            ]
        );
        // No exponents: "1e3" is 1, a command e, and 3, and `parse` stops at e.
        assert_eq!(
            tokenize("1e3"),
            [Token::Number(1.0), Token::Command('e'), Token::Number(3.0)]
        );
    }

    #[test]
    fn tokens_hand_out_whole_groups_of_numbers_or_none() {
        let mut tokens = Tokens::new("M1 2 3L4");
        assert_eq!(tokens.numbers::<2>(), None, "a command is in the way");
        assert_eq!(tokens.command(), Some('M'));
        assert_eq!(tokens.command(), None);
        assert_eq!(tokens.numbers(), Some([1.0, 2.0]));
        assert!(tokens.at_number());
        assert_eq!(tokens.numbers::<2>(), None, "one short");
        assert_eq!(tokens.command(), Some('L'));
        assert_eq!(tokens.number(), Some(4.0));
        assert!(tokens.done());
        assert_eq!(tokens.number(), None);
    }

    fn points(path: &str) -> Vec<Vec<Pos2>> {
        parse(path).into_iter().map(|sub| sub.points).collect()
    }

    #[test]
    fn a_closed_box_is_four_corners() {
        let subs = parse("M6 6h12v12H6z");
        assert_eq!(subs.len(), 1);
        assert!(subs[0].closed);
        assert_eq!(
            subs[0].points,
            vec![
                pos2(6.0, 6.0),
                pos2(18.0, 6.0),
                pos2(18.0, 18.0),
                pos2(6.0, 18.0)
            ]
        );
    }

    #[test]
    fn a_path_drawn_back_to_its_start_and_closed_has_no_point_twice() {
        assert_eq!(points("M6 6h12v12H6V6z"), points("M6 6h12v12H6z"));
    }

    #[test]
    fn relative_moves_and_implicit_line_tos_follow_the_pen() {
        let subs = parse("m6 9 6 6 6-6");
        assert_eq!(subs.len(), 1);
        assert_eq!(
            subs[0].points,
            vec![pos2(6.0, 9.0), pos2(12.0, 15.0), pos2(18.0, 9.0)]
        );
        assert_eq!(
            points("M6 9 12 15 18 9"),
            points("m6 9 6 6 6-6"),
            "after M they are absolute"
        );
    }

    #[test]
    fn horizontal_and_vertical_lines_keep_the_other_coordinate() {
        assert_eq!(
            points("M2 3H10V7h-4v-2"),
            [[
                pos2(2.0, 3.0),
                pos2(10.0, 3.0),
                pos2(10.0, 7.0),
                pos2(6.0, 7.0),
                pos2(6.0, 5.0)
            ]]
        );
    }

    #[test]
    fn a_move_starts_a_new_subpath_from_where_the_pen_was() {
        let subs = parse("M1 1h4m2 2h4M0 9v1");
        let starts: Vec<Pos2> = subs.iter().map(|sub| sub.points[0]).collect();
        assert_eq!(starts, [pos2(1.0, 1.0), pos2(7.0, 3.0), pos2(0.0, 9.0)]);
        assert!(subs.iter().all(|sub| !sub.closed));
    }

    #[test]
    fn numbers_after_a_close_draw_on_from_the_start_point() {
        let subs = parse("M1 1h4v4z2 2");
        assert_eq!(subs.len(), 2);
        assert!(subs[0].closed && !subs[1].closed);
        assert_eq!(subs[1].points, [pos2(1.0, 1.0), pos2(3.0, 3.0)]);
        assert_eq!(
            points("M1 1h4v4Z2 2")[1],
            [pos2(1.0, 1.0), pos2(2.0, 2.0)],
            "absolute after Z"
        );
    }

    #[test]
    fn a_path_ends_at_what_it_cannot_read_and_keeps_what_it_drew() {
        let drawn = [[pos2(1.0, 1.0), pos2(5.0, 5.0)]];
        assert_eq!(points("M1 1L5 5 9"), drawn, "a number short");
        assert_eq!(points("M1 1L5 5S1 1 2 2L9 9"), drawn, "no S");
        assert_eq!(points("M1 1L5 5C1 1 2 2"), drawn, "half a curve");
        assert_eq!(points("M1 1L5 5A1 1 0 0 1 9"), drawn, "most of an arc");
        assert!(parse("").is_empty());
        assert!(parse("L").is_empty());
    }

    #[test]
    fn curves_are_eight_chords_that_end_on_their_end_point() {
        let cubic = &points("M0 0C0 4 4 4 4 0")[0];
        assert_eq!(cubic.len(), 1 + CURVE_STEPS);
        assert_eq!(cubic[4], pos2(2.0, 3.0), "halfway");
        assert_eq!(cubic[8], pos2(4.0, 0.0));
        let quadratic = &points("M0 0Q2 4 4 0")[0];
        assert_eq!(quadratic.len(), 1 + CURVE_STEPS);
        assert_eq!(quadratic[4], pos2(2.0, 2.0), "halfway");
        assert_eq!(quadratic[8], pos2(4.0, 0.0));
        assert_eq!(
            points("M1 1c0 4 4 4 4 0q2 4 4 0")[0][16],
            pos2(9.0, 1.0),
            "relative control points count from the curve's start"
        );
    }

    #[test]
    fn an_arc_stays_on_its_circle() {
        let subs = parse("M12 3a9 9 0 1 0 0 18a9 9 0 1 0 0-18");
        assert_eq!(subs.len(), 1);
        let points = &subs[0].points;
        assert!(points.len() > 12, "flattened finely: {}", points.len());
        for p in points {
            let r = p.distance(pos2(12.0, 12.0));
            assert!((r - 9.0).abs() < 0.2, "{p:?} is {r} from the centre");
        }
        assert!(
            points.iter().any(|p| p.x < 4.0),
            "the arc went round the left"
        );
        assert!(points.iter().any(|p| p.x > 20.0), "and the right");
    }

    fn centre_arc(from: Pos2, r: (f32, f32), flags: (bool, bool), to: Pos2) -> CentreArc {
        CentreArc::from_endpoints(from, r.0, r.1, 0.0, flags.0, flags.1, to).unwrap()
    }

    #[test]
    fn the_flags_pick_the_centre_and_the_way_round() {
        use std::f32::consts::{FRAC_PI_2, PI};
        let (from, to) = (pos2(12.0, 3.0), pos2(12.0, 21.0));
        let left = centre_arc(from, (9.0, 9.0), (true, false), to);
        assert!(left.centre.distance(pos2(12.0, 12.0)) < 1e-4);
        assert!((left.theta + FRAC_PI_2).abs() < 1e-4, "starts at the top");
        assert!((left.delta + PI).abs() < 1e-4, "half a turn, anticlockwise");
        let right = centre_arc(from, (9.0, 9.0), (true, true), to);
        assert!((right.delta - PI).abs() < 1e-4, "and clockwise with sweep");
        // A quarter turn's two centres, a corner each.
        let (from, to) = (pos2(4.0, 0.0), pos2(0.0, 4.0));
        let near = centre_arc(from, (4.0, 4.0), (false, true), to);
        let far = centre_arc(from, (4.0, 4.0), (true, true), to);
        assert!(near.centre.distance(pos2(0.0, 0.0)) < 1e-4);
        assert!(far.centre.distance(pos2(4.0, 4.0)) < 1e-4);
        assert!((near.delta - FRAC_PI_2).abs() < 1e-4);
        assert!((far.delta - 3.0 * FRAC_PI_2).abs() < 1e-4);
    }

    #[test]
    fn radii_too_small_to_reach_grow_until_they_do() {
        let arc = centre_arc(pos2(0.0, 0.0), (1.0, 1.0), (false, true), pos2(10.0, 0.0));
        assert!((arc.rx - 5.0).abs() < 1e-4 && (arc.ry - 5.0).abs() < 1e-4);
        assert!(arc.centre.distance(pos2(5.0, 0.0)) < 1e-4);
    }

    #[test]
    fn an_arc_with_no_radius_or_nowhere_to_go_is_a_line() {
        let (a, b) = (pos2(1.0, 1.0), pos2(5.0, 1.0));
        assert_eq!(
            CentreArc::from_endpoints(a, 0.0, 4.0, 0.0, false, true, b),
            None
        );
        assert_eq!(
            CentreArc::from_endpoints(a, 4.0, 4.0, 0.0, false, true, a),
            None
        );
        assert_eq!(points("M1 1a0 4 0 0 1 4 0"), [[a, b]]);
    }

    #[test]
    fn an_arc_is_a_chord_per_sixteenth_of_a_turn_and_never_one() {
        let chords = |path: &str| points(path)[0].len() - 1;
        assert_eq!(chords("M12 3a9 9 0 1 0 0 18"), 8, "half a turn");
        assert_eq!(chords("M4 0a4 4 0 0 1-4 4"), 4, "a quarter");
        assert_eq!(chords("M4 0a4 4 0 0 1-.01 .2"), 2, "a sliver");
        let half = &points("M12 3a9 9 0 1 0 0 18")[0];
        assert_eq!(half[8], pos2(12.0, 21.0), "it lands where the path says");
    }

    #[test]
    fn arcs_have_the_centres_and_angles_kurbo_gives_them() {
        use std::f64::consts::TAU;
        let cases = [
            ((12.0, 3.0), (9.0, 9.0), 0.0, (12.0, 21.0)),
            ((16.0, 20.0), (4.0, 4.0), 0.0, (20.0, 16.0)),
            ((0.0, 0.0), (6.0, 3.0), 30.0, (7.0, 2.0)),
            ((5.0, 5.0), (2.0, 8.0), -75.0, (9.0, 1.0)),
            ((0.0, 0.0), (1.0, 2.0), 45.0, (10.0, 4.0)),
        ];
        for (from, radii, degrees, to) in cases {
            for (large, sweep) in [(false, false), (false, true), (true, false), (true, true)] {
                let ours = CentreArc::from_endpoints(
                    pos2(from.0, from.1),
                    radii.0,
                    radii.1,
                    degrees,
                    large,
                    sweep,
                    pos2(to.0, to.1),
                )
                .unwrap();
                let theirs = kurbo::Arc::from_svg_arc(&kurbo::SvgArc {
                    from: (f64::from(from.0), f64::from(from.1)).into(),
                    to: (f64::from(to.0), f64::from(to.1)).into(),
                    radii: (f64::from(radii.0), f64::from(radii.1)).into(),
                    x_rotation: f64::from(degrees).to_radians(),
                    large_arc: large,
                    sweep,
                })
                .unwrap();
                let what = format!("{from:?} to {to:?}, {radii:?} at {degrees}, {large} {sweep}");
                let close = |a: f32, b: f64| (f64::from(a) - b).abs() < 1e-3;
                assert!(close(ours.centre.x, theirs.center.x), "{what}: centre x");
                assert!(close(ours.centre.y, theirs.center.y), "{what}: centre y");
                assert!(close(ours.rx, theirs.radii.x), "{what}: rx");
                assert!(close(ours.ry, theirs.radii.y), "{what}: ry");
                assert!(close(ours.delta, theirs.sweep_angle), "{what}: sweep");
                let turn = (f64::from(ours.theta) - theirs.start_angle).rem_euclid(TAU);
                assert!(turn < 1e-3 || TAU - turn < 1e-3, "{what}: start");
            }
        }
    }

    #[test]
    fn a_dot_is_one_point() {
        let subs = parse("M5 12h.01M12 12h.01");
        assert_eq!(subs.len(), 2);
        assert_eq!(subs[0].points.len(), 1);
        assert_eq!(subs[1].points, vec![pos2(12.0, 12.0)]);
    }

    #[test]
    fn every_icon_parses_inside_its_box() {
        for icon in Icon::ALL {
            let subs = parse(icon.path());
            assert!(!subs.is_empty(), "{icon:?} drew nothing");
            for sub in &subs {
                for p in &sub.points {
                    assert!(
                        (-0.5..=24.5).contains(&p.x) && (-0.5..=24.5).contains(&p.y),
                        "{icon:?} leaves the box at {p:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn every_icon_reads_as_kurbo_reads_it() {
        for icon in Icon::ALL {
            assert_reads_as_kurbo_does(icon.path());
        }
    }

    #[test]
    fn all_lists_each_icon_once() {
        let listed: std::collections::HashSet<Icon> = Icon::ALL.into_iter().collect();
        assert_eq!(listed.len(), Icon::ALL.len());
    }

    #[test]
    fn the_logo_bubble_parses_with_its_arcs() {
        let subs = parse(crate::logo::WHISPER_BUBBLE);
        assert_eq!(subs.len(), 1);
        assert!(subs[0].closed);
        assert!(subs[0].points.len() > 20);
    }
}
