//! Cut files for a Great Highland bagpipe chanter whose conical bore is a
//! tapering polygon of long strips of ply, four, six or eight, glued, from the
//! sole up into a collar of stacked plates, with a stacked tenon and reed seat
//! above it.
//!
//!     target/release/ghb_chanter --no-write                  # the checks, four sides
//!     target/release/ghb_chanter --sides=6 --no-write        # six sides, or 8
//!     target/release/ghb_chanter --out=DIR                   # the sheet
//!     target/release/ghb_chanter --plate=2.9 ...             # for ply as measured
//!     target/release/ghb_chanter --kerf=0.18 ...             # for that kerf
//!     target/release/ghb_chanter --page-to=turn.html         # the 3D view
//!     target/release/ghb_chanter --solidcraft=DIR            # the SolidCraft designs, every polygon
//!     target/release/ghb_chanter --bores                     # the bores, for bore-tmm
//!
//! THE REFERENCE is Jeremy Starcher's Dis-Enchanter (2017, Thingiverse thing
//! 2373962), a 3D-printed chanter copied from a 1970s Hardie, measured from its
//! STL: a round conical bore from a 4.76mm throat to 21.2mm at the sole, eight
//! finger holes and two side holes. It was printed in two pieces; the joint is
//! ignored. Its designer calls it mostly working, with F and G low on the first
//! print, so it is a model to follow, not a proven scale.
//!
//! The bore here is the reference's cone made a regular polygon, the same area
//! at every height, so the strips taper: each strip's face is the polygon's
//! side, wider toward the sole. Its front and back are faces, for the finger
//! holes and the thumb's. The holes stay where the reference's are. They go
//! through a 3mm strip instead of the reference's 2mm of wall, so each hole's
//! size is solved for its note to sound where the reference's does, in
//! practice-chanters' chanter_model.rs's terms, and the side holes' size for
//! low G. The model treats the bore as an exact cone, closed at its apex for
//! the reed (bore-tmm's Cone kernel); it is used only for the difference
//! between the two chanters, not for the pitch.
//!
//! A hole wider than its face, less WEB_FACE, is a stadium instead: as wide as
//! that allows and long enough for the round hole's area. The model takes it as
//! that round hole.
//!
//! From the sole up:
//!
//!   * the STRIPS, tapering, from the sole into the collar, finger-jointed and
//!     glued; round each hole its strip owns both corners. Where the faces meet
//!     at more than a right angle (six and eight sides), a finger reaches only
//!     t / tan(half that angle) past the face's edge and the strip beside it
//!     stands back from the edge between fingers, as boxes.py's angled finger
//!     joints do. There is no butt-jointed version: the holes are too wide for
//!     their face to keep ply round them unless their strip owns the corners;
//!   * the FOOT, N_FOOT layers of strips lying flat round the strips' bottom,
//!     one to a side, each layer turned the other way to the one below, each
//!     strip's outer edge an arc of the reference's sole, FOOT_D round;
//!   * the COLLAR: a plate with a hole the strips' top stands in, glued to the
//!     strips; then N_COLLAR plates with the reference's round throat, each as
//!     wide as the reference's bead at its height;
//!   * the TENON: N_TENON round plates with the reed's tapered seat through
//!     them, which goes into the chanter stock with hemp. The collar's round
//!     plates and the tenon press together on two pins.
//!
//! And a JIG, not part of the chanter: a base plate and N_FOOT rings with the
//! foot's circle cut through them, pinned into a cup the foot is glued up in.
use bore_tmm::models::{scan, Cone};
use chanters::chanter_model::{self as m, plate, HOLES};
use chanters::practice_chanters::{self as pc, cc, circle, page_part, plate_m, CPart, Checks, Parts, Sheet, C, P2};
use chanters::standard_chanter::{self as s, Notes};
use chanters::strip_chanter::{self as sc, Run};
use lasermade_tools::die;
use lasermade_tools::py::{self, g};
use lasermade_tools::turn3d::{self, Mat, F, I, J};
use square_flute as sf;
use std::f64::consts::PI;
use std::hint::black_box;
use std::sync::Mutex;

mod ghb_solidcraft;

// --- the reference chanter, measured (mm, down from the seat's top) -----------
const REF_SEAT: (f64, f64) = (7.155, -0.0991); // the reed's seat, tapered: across at 0, per mm down
const REF_SEAT_DEEP: f64 = 22.0;
const REF_THROAT: f64 = 25.25; // the cone's narrowest, 4.76mm
const REF_CONE: (f64, f64) = (3.463, 0.05132); // the bore below: across at 0, per mm down
const REF_SOLE: f64 = 345.0; // the bore's end; the print's base past it is ignored
const REF_WALL: f64 = 2.0; // the wall at every hole
const REF_TENON_D: f64 = 17.0; // hemped into the stock
/// the bead the stock's top meets: (down, across) on its outside
const REF_BEAD: [(f64, f64); 10] = [
    (22.25, 34.12),
    (24.25, 33.82),
    (26.25, 33.04),
    (28.25, 31.73),
    (30.25, 29.83),
    (32.25, 27.23),
    (34.25, 23.75),
    (36.25, 18.69),
    (37.25, 15.21),
    (38.25, 10.34),
];
/// each hole: down from the seat's top, across, and the strip it is in
const REF_HOLES: [(&str, f64, f64, &str); 8] = [
    ("hA", 64.30, 4.80, "back"),
    ("G", 79.60, 4.80, "front"),
    ("F", 104.10, 6.00, "front"),
    ("E", 128.57, 7.15, "front"),
    ("D", 153.05, 7.20, "front"),
    ("C", 175.30, 7.90, "front"),
    ("B", 203.63, 7.85, "front"),
    ("A", 234.23, 7.85, "front"),
];
const REF_VENT: (f64, f64) = (286.20, 9.60); // two side holes, always open: they set low G
const REF_FOOT_D: f64 = 36.5;
const REF_FOOT_H: f64 = 7.25; // the sole's bead
// --- this chanter ---------------------------------------------------------------
const WEB_FACE: f64 = 0.35; // a round hole this much narrower than its face, at least
const MIN_STRIP: f64 = 1.0; // no strip narrower than this anywhere
const N_TENON: usize = 7; // 21mm of seat, the reference's 22
const N_COLLAR: usize = 4; // round plates over the socket plate
const FOOT_D: f64 = REF_FOOT_D;
const N_FOOT: usize = 2; // 6mm, the reference's sole 7.25mm
const JIG: f64 = FOOT_D + 10.0;
const JIG_CLEAR: f64 = 0.3;
const BY: &str = "ghb_chanter.rs";

fn names(n: usize) -> &'static [&'static str] {
    match n {
        4 => &["front", "right", "back", "left"],
        6 => &["front", "front right", "back right", "back", "back left", "front left"],
        _ => &["front", "front right", "right", "back right", "back", "back left", "left", "front left"],
    }
}
/// the side holes' strips
fn vents(n: usize) -> (usize, usize) {
    match n {
        4 => (1, 3),
        6 => (2, 4),
        _ => (2, 6),
    }
}

/// math.atan: libm's, as Python calls it
fn matan(x: f64) -> f64 {
    extern "C" {
        fn atan(x: f64) -> f64;
    }
    let f: unsafe extern "C" fn(f64) -> f64 = atan;
    unsafe { black_box(f)(x) }
}

/// The reference's bore across, z down from the seat's top.
fn ref_d(z: f64) -> f64 {
    if z < REF_SEAT_DEEP {
        return REF_SEAT.0 + REF_SEAT.1 * z;
    }
    let cone = REF_CONE.0 + REF_CONE.1 * z;
    if z < REF_THROAT {
        let top = REF_SEAT.0 + REF_SEAT.1 * REF_SEAT_DEEP;
        return top + (cone - top) * (z - REF_SEAT_DEEP) / (REF_THROAT - REF_SEAT_DEEP);
    }
    cone
}

/// The reference's bead across, z down, straight between the measures.
fn bead_d(z: f64) -> f64 {
    for w in REF_BEAD.windows(2) {
        let ((z0, d0), (z1, d1)) = (w[0], w[1]);
        if z <= z1 {
            return d0 + (d1 - d0) * (py::max(z, z0) - z0) / (z1 - z0);
        }
    }
    REF_BEAD[9].1
}

/// The polygon's side, z down: the reference's cone's area.
fn side(z: f64, n: usize) -> f64 {
    let k = (PI * py::mtan(PI / n as f64) / n as f64).sqrt();
    k * (REF_CONE.0 + REF_CONE.1 * z)
}

/// The polygon's inradius, z down.
fn inr(z: f64, n: usize) -> f64 {
    side(z, n) / (2.0 * py::mtan(PI / n as f64))
}

/// At a corner of the polygon: how far a finger reaches past the face's edge,
/// and how far the strip beside it stands back from the edge.
fn corner(t: f64, n: usize) -> (f64, f64) {
    let th = PI - 2.0 * PI / n as f64;
    let reach = t / py::mtan(th / 2.0);
    (reach, py::max(0.0, -reach * py::mcos(th)))
}

// --- the model: an exact cone, closed near its apex, holes along it ------------
const APEX: f64 = -REF_CONE.0 / REF_CONE.1;

/// A hole: name ("vent" for a side hole), z down, across, wall.
type Hole = (&'static str, f64, f64, f64);

/// The fingering's resonance nearest `near`.
fn sounding(holes: &[Hole], note: &str, near: f64) -> f64 {
    let fing = m::fingering(note);
    let mut order: Vec<&Hole> = holes.iter().collect();
    order.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    let (mut zh, mut r, mut a, mut t, mut op) = (vec![], vec![], vec![], vec![], vec![]);
    for &&(name, z, d, wall) in &order {
        zh.push(z);
        r.push(d / 2000.0);
        a.push((REF_CONE.0 + REF_CONE.1 * z) / 2000.0);
        t.push(wall);
        op.push((name == "vent" || fing.contains(&name)) as u8);
    }
    let cone = Cone { c: m::c(), rho: m::RHO, c0: REF_CONE.0, c1: REF_CONE.1, apex: APEX, sole: REF_SOLE, zh: &zh, r: &r, a: &a, t: &t, open: &op };
    let root = scan(near, 1.25, 60, 30, |f| cone.y(f));
    if root.is_nan() {
        die(format!("{note}: no resonance near {near:.0}Hz"));
    }
    root
}

fn ref_holes() -> Vec<Hole> {
    let mut v: Vec<Hole> = REF_HOLES.iter().map(|&(h, z, d, _)| (h, z, d, REF_WALL)).collect();
    v.push(("vent", REF_VENT.0, REF_VENT.1, REF_WALL));
    v.push(("vent", REF_VENT.0, REF_VENT.1, REF_WALL));
    v
}

/// The reference's notes, in the model.
fn reference() -> Notes {
    let holes = ref_holes();
    let low_a = sounding(&holes, "low A", 480.0);
    Notes(std::array::from_fn(|i| sounding(&holes, m::NOTES[i], low_a * m::RATIOS[i])))
}

fn bisect(f: &mut dyn FnMut(f64) -> f64, goal: f64, mut lo: f64, mut hi: f64, rising: bool) -> f64 {
    for _ in 0..30 {
        let mid = (lo + hi) / 2.0;
        if (f(mid) < goal) == rising {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    (lo + hi) / 2.0
}

/// ref, got, holes (in REF_HOLES' order), the vents' size
type Solved = (Notes, Notes, Vec<Hole>, f64);
static SOLVED: Mutex<Vec<(u64, Solved)>> = Mutex::new(vec![]);

fn hole_i(h: &str) -> usize {
    REF_HOLES.iter().position(|r| r.0 == h).unwrap()
}

/// Each hole's size, and the vents', that sound the reference's notes, the
/// holes where the reference's are, through a strip wall: the round hole's
/// diameter, whatever its shape. The same for every polygon.
fn solve() -> Solved {
    let t = plate();
    if let Some((_, v)) = SOLVED.lock().unwrap().iter().find(|e| e.0 == t.to_bits()) {
        return v.clone();
    }
    let r = reference();
    let mut holes: Vec<Hole> = REF_HOLES.iter().map(|&(h, z, d, _)| (h, z, d, t)).collect();
    let mut vent = REF_VENT.1;
    let every = |holes: &[Hole], vent: f64| -> Vec<Hole> {
        let mut v = holes.to_vec();
        v.push(("vent", REF_VENT.0, vent, t));
        v.push(("vent", REF_VENT.0, vent, t));
        v
    };
    for _ in 0..3 {
        let vmax = REF_CONE.0 + REF_CONE.1 * REF_VENT.0;
        if sounding(&every(&holes, vmax), "low G", r.get("low G")) < r.get("low G") {
            die(format!("error: low G is flat even with {vmax:.2}mm side holes."));
        }
        vent = bisect(&mut |d| sounding(&every(&holes, d), "low G", r.get("low G")), r.get("low G"), m::D_MIN, vmax, true);
        for h in HOLES {
            let nt = m::note_of(h);
            let i = hole_i(h);
            let d_max = REF_CONE.0 + REF_CONE.1 * holes[i].1;
            holes[i].2 = d_max;
            if sounding(&every(&holes, vent), nt, r.get(nt)) < r.get(nt) {
                die(format!("error: {h} is flat even {d_max:.2}mm across."));
            }
            let mut hc = holes.clone();
            holes[i].2 = bisect(
                &mut |d| {
                    hc[i].2 = d;
                    sounding(&every(&hc, vent), nt, r.get(nt))
                },
                r.get(nt),
                m::D_MIN,
                d_max,
                true,
            );
        }
    }
    let got = Notes(m::NOTES.map(|nt| sounding(&every(&holes, vent), nt, r.get(nt))));
    let v = (r, got, holes, vent);
    SOLVED.lock().unwrap().push((t.to_bits(), v.clone()));
    v
}

// --- the strips --------------------------------------------------------------
// Strips are numbered round the bore from the front, toward the player's
// right; corner i is between strip i and strip i + 1. In a strip, u runs
// across its face from its centre line, toward strip i + 1.

/// Each hole on the strips: (strip, x down from the seat's top, round diameter).
fn holes_of(holes: &[Hole], vent: f64, n: usize) -> Vec<(usize, f64, f64)> {
    let mut out: Vec<(usize, f64, f64)> = holes.iter().map(|&(h, z, d, _)| (if h == "hA" { n / 2 } else { 0 }, z, d)).collect();
    let (a, b) = vents(n);
    out.push((a, REF_VENT.0, vent));
    out.push((b, REF_VENT.0, vent));
    out
}

/// A hole's (width, length) on its face: round if it fits, else the stadium
/// of its area as wide as the face allows at the hole's top.
fn shape(z: f64, d: f64, n: usize) -> (f64, f64) {
    let (mut w, mut ln) = (d, d);
    for _ in 0..60 {
        let face = side(z - ln / 2.0, n) - WEB_FACE;
        if d <= face {
            return (d, d);
        }
        w = face;
        ln = w + (PI * d * d / 4.0 - PI * w * w / 4.0) / w;
    }
    (w, ln)
}

/// A stadium centred (x, 0) in a strip's frame, its length along x.
fn stadium(x: f64, w: f64, ln: f64, steps: usize) -> Vec<P2> {
    let (r, h) = (w / 2.0, (ln - w) / 2.0);
    let mut pts = vec![];
    for i in 0..=steps {
        let a = PI * i as f64 / steps as f64;
        pts.push((x + h + r * py::msin(a), r * py::mcos(a)));
    }
    for i in 0..=steps {
        let a = PI * i as f64 / steps as f64;
        pts.push((x - h - r * py::msin(a), -r * py::mcos(a)));
    }
    pts
}

/// Each corner's owner along the strips, from the strips' top: each strip
/// owns one of its corners, then the other, finger by finger; round each hole
/// its strip owns both, MIN_WEB past the hole, and the strips beside it keep
/// their other corners.
fn owners(ls: f64, x0: f64, holes: &[(usize, f64, f64)], n: usize) -> Vec<Vec<Run<usize>>> {
    let nf = py::round(ls / sc::FINGER).max(1) as usize;
    let mut cuts: Vec<f64> = (0..=nf).map(|i| ls * i as f64 / nf as f64).collect();
    let guard: Vec<(f64, f64, usize)> = holes
        .iter()
        .map(|&(st, z, d)| {
            let (_, ln) = shape(z, d, n);
            (z - x0 - ln / 2.0 - pc::MIN_WEB, z - x0 + ln / 2.0 + pc::MIN_WEB, st)
        })
        .collect();
    for &(a, b, _) in &guard {
        cuts.push(py::min(py::max(a, 0.0), ls));
        cuts.push(py::min(py::max(b, 0.0), ls));
    }
    cuts.sort_by(|a, b| a.partial_cmp(b).unwrap());
    cuts.dedup();
    let mut out: Vec<Vec<Run<usize>>> = vec![vec![]; n];
    for w in cuts.windows(2) {
        let (a, b) = (w[0], w[1]);
        let mid = (a + b) / 2.0;
        let k = ((mid / ls * nf as f64) as usize).min(nf - 1);
        let who = |c: usize| if k % 2 == 0 { c } else { (c + 1) % n };
        let gs: Vec<usize> = guard.iter().filter(|&&(g0, g1, _)| g0 <= mid && mid <= g1).map(|e| e.2).collect();
        let mut mine: Vec<Option<usize>> = vec![None; n];
        for &s in &gs {
            mine[(s + n - 1) % n] = Some(s);
            mine[s] = Some(s);
        }
        for &s in &gs {
            let c = (s + 1) % n;
            mine[c].get_or_insert(c);
            mine[(s + 2 * n - 2) % n].get_or_insert((s + n - 1) % n);
        }
        for c in 0..n {
            out[c].push((a, b, mine[c].unwrap_or(who(c)), mine[c].is_some()));
        }
    }
    out.into_iter().map(sc::tidy).collect()
}

/// Each corner's runs with how far its owner's finger reaches, as
/// [(x0, x1, owner, reach)].
fn reaches(own: &[Vec<Run<usize>>], x0: f64, t: f64, n: usize) -> Vec<Vec<(f64, f64, usize, f64)>> {
    let (full, back) = corner(t, n);
    let k = back / full;
    let mut cuts: Vec<f64> = own.iter().flatten().flat_map(|r| [r.0, r.1]).collect();
    cuts.sort_by(|a, b| a.partial_cmp(b).unwrap());
    cuts.dedup();
    let mut out: Vec<Vec<(f64, f64, usize, f64)>> = vec![vec![]; n];
    for w in cuts.windows(2) {
        let (a, b) = (w[0], w[1]);
        let mid = (a + b) / 2.0;
        let who: Vec<usize> = (0..n).map(|c| own[c].iter().find(|r| r.0 <= mid && mid <= r.1).unwrap().2).collect();
        let mut r = vec![full; n];
        for i in 0..n {
            if k != 0.0 && who[(i + n - 1) % n] != i && who[i] != i {
                let cap = (side(x0 + a, n) - MIN_STRIP - 0.05) / (2.0 * k);
                for c in [(i + n - 1) % n, i] {
                    r[c] = py::max(0.0, py::min(r[c], cap));
                }
            }
        }
        for c in 0..n {
            let iv = &mut out[c];
            match iv.last_mut() {
                Some(l) if l.2 == who[c] && (l.3 - r[c]).abs() < 1e-12 => l.1 = b,
                _ => iv.push((a, b, who[c], r[c])),
            }
        }
    }
    out
}

/// A strip from its top, x = 0, to the sole; u across it from its centre
/// line, a finger reaching past the face where the strip owns that corner,
/// standing back from it where it does not. lo, hi: its edges' runs,
/// [(x0, x1, owns, reach)].
fn strip_outline(lo: &[(f64, f64, bool, f64)], hi: &[(f64, f64, bool, f64)], x0: f64, t: f64, n: usize) -> Vec<P2> {
    let (full, back) = corner(t, n);
    let k = back / full;
    let edge = |runs: &[(f64, f64, bool, f64)], sign: f64| -> Vec<P2> {
        let mut pts = vec![];
        for &(a, b, owns, r) in runs {
            for x in [a, b] {
                pts.push((x, sign * (side(x0 + x, n) / 2.0 + if owns { r } else { -k * r })));
            }
        }
        pts
    };
    let mut pts = edge(lo, -1.0);
    pts.extend(edge(hi, 1.0).into_iter().rev());
    let mut out: Vec<P2> = vec![];
    for p in pts {
        if let Some(l) = out.last() {
            if (l.0 - p.0).abs() < 1e-9 && (l.1 - p.1).abs() < 1e-9 {
                continue;
            }
        }
        out.push(p);
    }
    let m_ = out.len();
    let mut clean = vec![];
    for i in 0..m_ {
        let (a, p, c) = (out[(i + m_ - 1) % m_], out[i], out[(i + 1) % m_]);
        if ((p.0 - a.0) * (c.1 - p.1) - (p.1 - a.1) * (c.0 - p.0)).abs() < 1e-9 {
            continue;
        }
        clean.push(p);
    }
    clean
}

/// Strip i's outward normal in the plate frame.
fn normal(i: usize, n: usize) -> P2 {
    let a = -2.0 * PI * i as f64 / n as f64;
    (-py::msin(a), -py::mcos(a))
}

/// The strips' outside at z down, in the plate frame: vertex i between strip
/// i and strip i + 1.
fn outer(z: f64, t: f64, n: usize) -> Vec<P2> {
    let r = (inr(z, n) + t) / py::mcos(PI / n as f64);
    (0..n)
        .map(|i| {
            let (a, b) = (normal(i, n), normal((i + 1) % n, n));
            let mm = py::hypot(a.0 + b.0, a.1 + b.1);
            (r * (a.0 + b.0) / mm, r * (a.1 + b.1) / mm)
        })
        .collect()
}

struct Built {
    r: Notes,
    got: Notes,
    holes: Vec<Hole>,
    vent: f64,
    t: f64,
    ls: f64,
    x0: f64,
    own: Vec<Vec<Run<usize>>>,
    on: Vec<(usize, f64, f64)>,
    rr: f64,
    tube: Vec<P2>,
}

fn build(n: usize) -> (Vec<CPart>, Built) {
    let (r, got, holes, vent) = solve();
    let t = plate();
    let mut parts = Parts::default();
    let x0 = (N_TENON + N_COLLAR) as f64 * t;
    let ls = REF_SOLE - x0;
    let on = holes_of(&holes, vent, n);
    let own = owners(ls, x0, &on, n);
    let reach = reaches(&own, x0, t, n);
    for i in 0..n {
        let runs = |e: usize| -> Vec<(f64, f64, bool, f64)> { reach[e].iter().map(|&(a, c, w, r)| (a, c, w == i, r)).collect() };
        let o = strip_outline(&runs((i + n - 1) % n), &runs(i), x0, t, n);
        let mut cuts = vec![];
        for &(st, z, d) in &on {
            if st != i {
                continue;
            }
            let (w, ln) = shape(z, d, n);
            cuts.push(if w == d { cc(z - x0, 0.0, d / 2.0) } else { C::Poly(stadium(z - x0, w, ln, 16)) });
        }
        let nm = names(n)[i];
        let q = parts.add(&format!("{nm} strip"), &o, cuts, x0, "strip", &format!("the {nm} strip"), None);
        q.strip_k = i;
    }
    // the foot: a pinwheel of flat strips round the strips' outside each
    // layer, the other hand each time, cut to the reference's sole
    let rr = FOOT_D / 2.0;
    let to_circle = |p: P2, d: P2| -> P2 {
        let b = p.0 * d.0 + p.1 * d.1;
        let s = -b + (b * b - (py::powf(p.0, 2.0) + py::powf(p.1, 2.0) - rr * rr)).sqrt();
        (p.0 + s * d.0, p.1 + s * d.1)
    };
    for k in 0..N_FOOT {
        let w = outer(REF_SOLE - (k as f64 + 0.5) * t, t, n);
        for j in 0..n {
            let (a, b, c) = (w[(j + n - 1) % n], w[j], w[(j + 1) % n]);
            let dab = (a.0 - b.0, a.1 - b.1);
            let dab = (dab.0 / py::hypot(dab.0, dab.1), dab.1 / py::hypot(dab.0, dab.1));
            let dbc = (b.0 - c.0, b.1 - c.1);
            let dbc = (dbc.0 / py::hypot(dbc.0, dbc.1), dbc.1 / py::hypot(dbc.0, dbc.1));
            let (p0, q) = (to_circle(a, dab), to_circle(b, dbc));
            let (a0, mut a1) = (py::matan2(q.1, q.0), py::matan2(p0.1, p0.0));
            while a1 < a0 {
                a1 += 2.0 * PI;
            }
            if a1 - a0 > PI {
                a1 -= 2.0 * PI;
            }
            let mut pts = vec![p0, a, b, q];
            pts.extend((1..40).map(|i| {
                let ang = a0 + (a1 - a0) * i as f64 / 40.0;
                (rr * py::mcos(ang), rr * py::msin(ang))
            }));
            if k % 2 == 1 {
                pts = pts.iter().rev().map(|&(x, y)| (-x, y)).collect();
            }
            let p = parts.add("foot", &pts, vec![], REF_SOLE - (k + 1) as f64 * t, "foot", &format!("foot layer {}", k + 1), None);
            p.w_ring = w.clone();
        }
    }
    s::jig(&mut parts, rr, N_FOOT, JIG, JIG_CLEAR);
    // the tenon and the collar, round, pinned together midway between the
    // seat's widest and the tenon's edge
    let pin_d = pc::PIN - pc::PIN_INTERF;
    let tenon = circle(0.0, 0.0, REF_TENON_D / 2.0, 72);
    let r_pin = (REF_SEAT.0 / 2.0 + REF_TENON_D / 2.0) / 2.0;
    let pins = [pc::polar(45.0, r_pin), pc::polar(225.0, r_pin)];
    let pin_cut = || pins.iter().map(|&(x, y)| cc(x, y, pin_d / 2.0)).collect::<Vec<_>>();
    for k in 0..N_TENON {
        let d = ref_d((k as f64 + 0.5) * t);
        let mut cuts = vec![cc(0.0, 0.0, d / 2.0)];
        cuts.extend(pin_cut());
        parts.add("tenon", &tenon, cuts, k as f64 * t, "seat", &format!("tenon plate {}, the seat {d:.2}mm", k + 1), Some(REF_TENON_D / 2.0));
    }
    for k in 0..N_COLLAR {
        let x = (N_TENON + k) as f64 * t;
        let (dd, d) = (bead_d(x + t / 2.0), ref_d(x + t / 2.0));
        let mut cuts = vec![cc(0.0, 0.0, d / 2.0)];
        cuts.extend(pin_cut());
        parts.add("collar", &circle(0.0, 0.0, dd / 2.0, 72), cuts, x, "collar", &format!("collar plate {}, {dd:.1}mm", k + 1), Some(dd / 2.0));
    }
    let dd = bead_d(x0 + t / 2.0);
    let tube = outer(x0, t, n);
    parts.add(
        "collar socket",
        &circle(0.0, 0.0, dd / 2.0, 72),
        vec![C::Poly(tube.clone())],
        x0,
        "sock",
        &format!("the collar's socket plate, {dd:.1}mm"),
        Some(dd / 2.0),
    );
    (parts.v, Built { r, got, holes, vent, t, ls, x0, own, on, rr, tube })
}

/// A strip's narrowest, across, and where.
fn widths(p: &CPart) -> (f64, f64) {
    let o = p.of();
    let mut xs: Vec<f64> = o.iter().map(|v| py::round_dp(v.0, 9)).collect();
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    xs.dedup();
    let mut best = (f64::INFINITY, 0.0);
    for w in xs.windows(2) {
        let x = (w[0] + w[1]) / 2.0;
        let mut us = vec![];
        for j in 0..o.len() {
            let ((x1, u1), (x2, u2)) = (o[j], o[(j + 1) % o.len()]);
            if (x1 - x) * (x2 - x) < 0.0 {
                us.push(u1 + (u2 - u1) * (x - x1) / (x2 - x1));
            }
        }
        if us.len() >= 2 {
            let c = (sf::fmax(&us) - sf::fmin(&us), x);
            if c.partial_cmp(&best) == Some(std::cmp::Ordering::Less) {
                best = c;
            }
        }
    }
    best
}

fn checks(n: usize, parts: &[CPart], info: &Built, sheets: &[Sheet]) -> Checks {
    let mut out: Checks = vec![];
    let mut note = |ok: bool, what: &str, detail: String| out.push((ok, what.to_string(), detail));
    let (t, ls, x0) = (info.t, info.ls, info.x0);
    let (reach, back) = corner(t, n);
    let worst = s::worst_cents(&info.got, &info.r);
    note(
        worst <= 1.0,
        "every note sounds where the reference's does, in the model",
        format!("within {worst:.2} cents; the reference's low A {:.0}Hz in the model", info.r.get("low A")),
    );
    let moved = sf::fmax(&HOLES.iter().map(|h| (REF_HOLES[hole_i(h)].1 - info.holes[hole_i(h)].1).abs()).collect::<Vec<_>>());
    let (v0, v1) = vents(n);
    let nv = info.on.iter().filter(|&&(st, z, _)| (st == v0 || st == v1) && (z - REF_VENT.0).abs() < 1e-9).count();
    note(
        moved < 1e-9 && nv == 2,
        "every hole is where the reference's is",
        HOLES.iter().map(|h| format!("{h} {:.1}", REF_HOLES[hole_i(h)].1)).collect::<Vec<_>>().join(", ")
            + &format!(", the side holes {:.1}mm down from the seat's top", REF_VENT.0),
    );
    let (mut fit, mut stad) = (true, vec![]);
    for &(st, z, d) in &info.on {
        let (w, ln) = shape(z, d, n);
        if w > side(z - ln / 2.0, n) - WEB_FACE + 1e-9 || w < m::D_MIN {
            fit = false;
        }
        if (PI * d * d / 4.0 - (PI * w * w / 4.0 + w * (ln - w))).abs() > 1e-6 {
            fit = false;
        }
        if w != d {
            stad.push((st, z, w, ln));
        }
    }
    let name_at = |z: f64| REF_HOLES.iter().find(|r| r.1 == z).map(|r| r.0).unwrap_or("a side hole");
    note(
        fit,
        "every hole cuts inside its face, at its area",
        if stad.is_empty() {
            "all round".into()
        } else {
            "round but for ".to_string()
                + &stad.iter().filter(|q| q.0 != v1).map(|&(_, z, w, ln)| format!("{} {w:.2} x {ln:.2}mm", name_at(z))).collect::<Vec<_>>().join(", ")
                + ", stadiums"
        },
    );
    let mut ok = true;
    for &(st, z, d) in &info.on {
        let (_, ln) = shape(z, d, n);
        let x = z - x0;
        for c in [(st + n - 1) % n, st] {
            for &(a, e, who, _) in &info.own[c] {
                if a < x + ln / 2.0 + pc::MIN_WEB && e > x - ln / 2.0 - pc::MIN_WEB && who != st {
                    ok = false;
                }
            }
        }
    }
    note(ok, "every hole keeps ply all round it", format!("its strip fingered both sides, {}mm past it along", g(pc::MIN_WEB)));
    let own = &info.own;
    let runs: Vec<f64> = own.iter().flatten().filter(|r| !r.3).map(|r| r.1 - r.0).collect();
    note(
        s::covered(own, ls) && (runs.is_empty() || sf::fmin(&runs) >= sc::MIN_FINGER - 1e-9),
        "every corner is one strip's, all the way",
        format!("finger joints; the shortest finger {:.1}mm; a finger reaches {reach:.2}mm past its face", sf::fmin(&runs))
            + &if back != 0.0 { format!(", the strip beside it standing back {back:.2}mm") } else { String::new() },
    );
    let strips: Vec<&CPart> = parts.iter().filter(|p| p.kind == "strip").collect();
    let mut narrow = (widths(strips[0]), strips[0]);
    for &p in &strips[1..] {
        let w = widths(p);
        if w.0 < narrow.0 .0 {
            narrow = (w, p);
        }
    }
    note(
        narrow.0 .0 >= MIN_STRIP,
        "no strip is too narrow anywhere",
        format!("the narrowest {:.2}mm, the {} strip {:.0}mm down, against {}", narrow.0 .0, names(n)[narrow.1.strip_k], narrow.0 .1 + x0, g(MIN_STRIP)),
    );
    let lean = py::degrees(matan(inr(1.0, n) - inr(0.0, n)));
    note(
        strips.len() == n,
        &format!("the strips make the reference's cone, {n}-sided"),
        format!(
            "{n} strips; the faces {:.2}mm at the collar to {:.2}mm at the sole, the same area as the reference's {:.2} to {:.2}mm round; each face leans {lean:.2} degrees",
            side(x0, n),
            side(REF_SOLE, n),
            ref_d(x0),
            ref_d(REF_SOLE)
        ),
    );
    let socket: Vec<&CPart> = parts.iter().filter(|p| p.kind == "sock").collect();
    note(
        socket.len() == 1 && (socket[0].x - x0).abs() < 1e-9 && (x0 + ls - REF_SOLE).abs() < 1e-9,
        "the strips run from the sole into the collar",
        format!("{ls:.1}mm, their top {}mm into its socket plate; the bore {}mm from the seat's top to the sole, the reference's", g(t), g(REF_SOLE)),
    );
    let sweb = socket[0].round.unwrap() - sf::fmax(&info.tube.iter().map(|v| py::hypot(v.0, v.1)).collect::<Vec<_>>());
    let mut collar: Vec<&CPart> = parts.iter().filter(|p| p.kind == "collar" || p.kind == "sock").collect();
    let cok = collar.iter().all(|p| (2.0 * p.round.unwrap() - bead_d(p.x + t / 2.0)).abs() < 1e-9);
    collar.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
    note(
        sweb >= pc::MIN_WEB && cok,
        "the collar is the reference's bead, plate by plate",
        collar.iter().map(|p| format!("{:.1}", 2.0 * p.round.unwrap())).collect::<Vec<_>>().join(", ")
            + &format!("mm round; {sweb:.1}mm of ply past the strips' corners in the socket plate"),
    );
    let rr = info.rr;
    let feet: Vec<&CPart> = parts.iter().filter(|p| p.kind == "foot").collect();
    let mut bad = 0;
    for k in 0..N_FOOT {
        let layer: Vec<&&CPart> = feet.iter().filter(|p| (p.x - (REF_SOLE - (k + 1) as f64 * t)).abs() < 1e-9).collect();
        let w = &layer[0].w_ring;
        let outs: Vec<Vec<P2>> = layer.iter().map(|p| p.of()).collect();
        for i in 0..60 {
            for j in 0..60 {
                let q = (-rr + (i as f64 + 0.5) * 2.0 * rr / 60.0, -rr + (j as f64 + 0.5) * 2.0 * rr / 60.0);
                let r = py::hypot(q.0, q.1);
                if (r - rr).abs() < 0.05 || py::min(pc::poly_dist(q, w), 9.0) < 0.02 {
                    continue;
                }
                let want = if pc::inside(q, w) || r > rr { 0 } else { 1 };
                bad += (outs.iter().filter(|o| pc::inside(q, o)).count() != want) as usize;
            }
        }
    }
    let far = sf::fmax(&feet.iter().flat_map(|p| p.w_ring.iter()).map(|v| py::hypot(v.0, v.1)).collect::<Vec<_>>());
    let lowest = sf::fmax(&info.on.iter().map(|&(_, z, d)| z + shape(z, d, n).1 / 2.0).collect::<Vec<_>>());
    let clear = (REF_SOLE - N_FOOT as f64 * t) - lowest;
    note(
        bad == 0 && feet.len() == n * N_FOOT && clear >= pc::MIN_WEB && rr - far >= 0.5,
        "the foot is the reference's sole, and clear of the side holes",
        format!(
            "{}mm round and {}mm tall, the reference's {}mm and {}mm; {:.2}mm of it past the strips' corners; {clear:.1}mm under the side holes",
            g(2.0 * rr),
            g(N_FOOT as f64 * t),
            g(REF_FOOT_D),
            g(REF_FOOT_H),
            rr - far
        ),
    );
    s::jig_check(&mut note, parts, rr, N_FOOT, t, JIG_CLEAR);
    let web = pc::pin_web(parts, Some(&["seat", "collar", "sock", "jig"]), &[(pc::PIN - pc::PIN_INTERF) / 2.0], false);
    note(web >= pc::MIN_WEB, "every pin keeps ply between it and every cut", format!("{web:.2}mm at the least, against {}", g(pc::MIN_WEB)));
    let mut seat: Vec<&CPart> = parts.iter().filter(|p| p.kind == "seat").collect();
    seat.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
    let r0 = |p: &CPart| match p.p.cuts[0] {
        square_flute::Cut::Circle(_, _, r) => r,
        _ => f64::NAN,
    };
    note(
        seat.iter().all(|p| (2.0 * r0(p) - ref_d(p.x + t / 2.0)).abs() < 1e-9),
        "the seat is the reference's taper, and the tenon its size",
        format!(
            "{:.2} to {:.2}mm over {}mm (the reference's {:.2} to {:.2}mm over {}); the tenon {}mm round, for hemp",
            2.0 * r0(seat[0]),
            2.0 * r0(seat[seat.len() - 1]),
            g(N_TENON as f64 * t),
            ref_d(0.0),
            ref_d(REF_SEAT_DEEP),
            g(REF_SEAT_DEEP),
            g(REF_TENON_D)
        ),
    );
    pc::sheet_checks(&mut note, sheets, parts.len(), "");
    out
}

fn label(n: usize) -> String {
    format!("ghb-chanter-{n}-sided")
}

fn svg_sheet(k: usize, nsh: usize, sh: &Sheet, n: usize) -> String {
    let txt = pc::svg_sheet(k, nsh, sh, n);
    let what = format!("Bagpipe chanter, a tapering {n}-sided bore of finger-jointed strips");
    txt.replace(&format!("Practice chanter, {n}-sided bore"), &what)
        .replace("cuts the bores, seats and pin holes first; then black", "cuts the bores, seats, finger and pin holes first; then black")
}

/// Where a part sits: a plate as practice_chanters lays them; a strip's x
/// along it, u across from its centre line, w through from the bore out, its
/// face leaning with the cone.
fn placed(p: &CPart, info: &Built, n: usize) -> Mat {
    if p.kind != "strip" {
        return plate_m(p.x);
    }
    let x = info.x0;
    let (nx, ny) = normal(p.strip_k, n);
    let (tx, ty) = (-ny, nx);
    let (r0, kr) = (inr(x, n), inr(x + 1.0, n) - inr(x, n));
    vec![
        vec![I(1), I(0), I(0), F(x)],
        vec![F(kr * ny), F(ty), F(ny), F(r0 * ny)],
        vec![F(-kr * nx), F(-tx), F(-nx), F(-r0 * nx)],
        vec![I(0), I(0), I(0), I(1)],
    ]
}

/// The chanter for the 3D page.
fn design3d(n: usize) -> J {
    let (parts, info) = build(n);
    let mid = REF_SOLE / 2.0;
    let out: Vec<J> = parts
        .iter()
        .filter(|p| p.kind != "jig")
        .map(|p| {
            let ex = if p.kind == "strip" {
                let (nx, ny) = normal(p.strip_k, n);
                [I(0), F(ny * 0.15), F(-nx * 0.15)]
            } else {
                [F((p.x - mid) / 60.0), I(0), I(0)]
            };
            page_part(p, &placed(p, &info, n), ex, if p.kind == "strip" { "dark" } else { "ply" })
        })
        .collect();
    pc::design_j(out)
}

fn page(path_out: &str) {
    let mut designs: Vec<(String, J)> = [4, 6, 8].iter().map(|&n| (format!("{n} sides"), design3d(n))).collect();
    turn3d::write(
        path_out,
        "Bagpipe chanter in 3D",
        &mut designs,
        "Drag to turn; right-drag, shift-drag or move two fingers to slide the view; \
         scroll or pinch to zoom in on whatever is under the pointer; double-click a part \
         to turn round it, and Reset view to go back. Draw Explode to pull the strips outward and \
         the plates apart along the chanter. Every part is the cut file\u{2019}s own outline. \
         The strips, darker, taper with the cone: wider toward the sole. The finger holes \
         face you, high A\u{2019}s on the back, and a side hole on each side near the sole; \
         where a hole is wider than its face it is a stadium. Above the strips, the collar \
         and the tenon with the reed\u{2019}s seat; at the sole, two layers of flat strips \
         make the foot. The buttons switch between four, six and eight strips.",
        BY,
        false,
    );
}

/// The chanters' bores as bore-tmm reads them, on stdout as one JSON
/// object: "cut", each side count as drawn (bore-tmm's
/// tools/read_cut_files.py writes them to bores/cut/), and "models", solve()'s
/// design (tools/export_models.py writes it to bores/models/).
fn bores() {
    use chanters::bores::{by_place, n as num, st, strip_holes, strs};
    let r = reference();
    let mut down: Vec<&'static str> = REF_HOLES.iter().map(|h| h.0).collect();
    down.sort_by(|a, b| REF_HOLES[hole_i(a)].1.partial_cmp(&REF_HOLES[hole_i(b)].1).unwrap());
    let (d0, slope) = REF_CONE;
    let t = plate();
    let hole = |h: &str, at: f64, d: f64| J::obj(vec![("name", st(h)), ("at_mm", num(at)), ("d_mm", num(d)), ("chimney_mm", num(t))]);
    let vent = |i: usize, at: f64, d: f64| {
        J::obj(vec![("name", st(&format!("vent{i}"))), ("at_mm", num(at)), ("d_mm", num(d)), ("chimney_mm", num(t)), ("always_open", J::B(true))])
    };
    let notes = |hz: &Notes, model: Option<&Notes>| -> J {
        J::A(
            m::NOTES
                .iter()
                .map(|&nt| {
                    let mut kv = vec![("name", st(nt)), ("hz", num(hz.get(nt))), ("open", strs(m::fingering(nt))), ("span", num(1.25))];
                    if let Some(g) = model {
                        kv.push(("model_hz", num(g.get(nt))));
                    }
                    J::obj(kv)
                })
                .collect(),
        )
    };
    let drive = || J::obj(vec![("kind", st("reed")), ("apex", J::B(true))]);
    let cone = |l: f64| J::A(vec![J::obj(vec![("len_mm", num(l)), ("d1_mm", num(d0)), ("d2_mm", num(d0 + slope * l)), ("loss_ratio", num(1.0))])]);
    let mut cut = vec![];
    for n in [4, 6, 8] {
        let (parts, info) = build(n);
        let found = strip_holes(&parts, 0.0);
        let (vents, fingers): (Vec<_>, Vec<_>) = found.into_iter().partition(|f| (f.0 - REF_VENT.0).abs() < 1.0);
        if vents.len() != 2 {
            die(format!("UNREADABLE ghb_chanter.rs {n} sides: {} side holes, not 2", vents.len()));
        }
        let holes = by_place(fingers, &down, &format!("ghb_chanter.rs {n} sides"));
        let sole = parts.iter().filter(|p| p.kind == "strip").map(|p| p.x).fold(f64::NEG_INFINITY, f64::max) + info.ls;
        let mut hs: Vec<J> = holes.iter().map(|&(h, z, d)| hole(h, z, d)).collect();
        hs.extend(vents.iter().enumerate().map(|(i, v)| vent(i + 1, v.0, v.1)));
        cut.push((
            format!("ghb-chanter-{n}-sided"),
            J::obj(vec![
                ("name", st(&format!("GHB chanter, {n}-sided strips, as drawn"))),
                (
                    "source",
                    st(&format!(
                        "ghb-chanter/ghb_chanter.rs build() at {n} sides: strip holes, side holes and the sole; the cone, its loss radius (the radius) and the reference's notes from its model"
                    )),
                ),
                ("temp_c", num(m::T_C)),
                ("drive", drive()),
                ("bore", cone(sole)),
                ("end", st("unflanged")),
                ("holes", J::A(hs)),
                ("notes", notes(&r, None)),
            ]),
        ));
    }
    let (r, got, holes, vd) = solve();
    let mut hs: Vec<J> = holes.iter().map(|&(h, z, d, w)| J::obj(vec![("name", st(h)), ("at_mm", num(z)), ("d_mm", num(d)), ("chimney_mm", num(w))])).collect();
    hs.extend((1..=2).map(|i| vent(i, REF_VENT.0, vd)));
    let model = J::obj(vec![
        ("name", st("GHB chanter, ghb_chanter.rs solve(): holes sized to sound the reference's notes")),
        ("source", st("ghb-chanter/ghb_chanter.rs solve(); hz = the reference chanter in the same model")),
        ("temp_c", num(m::T_C)),
        ("drive", drive()),
        ("bore", cone(REF_SOLE)),
        ("end", st("unflanged")),
        ("holes", J::A(hs)),
        ("notes", notes(&r, Some(&got))),
    ]);
    let mut s = String::new();
    J::O(vec![("cut".into(), J::O(cut)), ("models".into(), J::O(vec![("ghb-chanter".into(), model)]))]).dump(&mut s);
    println!("{s}");
}

fn main() {
    if std::env::args().nth(1).as_deref() == Some("--bores") {
        return bores();
    }
    let (mut out, mut write, mut n, mut page_to, mut scd) = (None::<String>, true, 4usize, None::<String>, None::<String>);
    for x in std::env::args().skip(1) {
        let val = || x.split_once('=').unwrap().1.to_string();
        if x == "--no-write" {
            write = false;
        } else if x.starts_with("--out=") {
            out = Some(val());
        } else if x.starts_with("--sides=") {
            n = val().trim().parse().unwrap_or_else(|_| die(format!("error: {x}: not a number.")));
            if ![4, 6, 8].contains(&n) {
                die(format!("error: --sides={n}: 4, 6 or 8."));
            }
        } else if x.starts_with("--plate=") {
            m::PLATE.set(pc::num_arg(&x));
        } else if x.starts_with("--kerf=") {
            sf::set_kerf(&x);
        } else if x.starts_with("--page-to=") {
            page_to = Some(val());
        } else if x.starts_with("--solidcraft=") {
            scd = Some(val());
        } else {
            die(format!("error: {x} is not a flag this generator reads. It takes --sides=, --out=, --plate=, --kerf=, --page-to=, --solidcraft= and --no-write."));
        }
    }
    if let Some(dir) = scd {
        // every polygon's design, whatever --sides= says
        std::fs::create_dir_all(&dir).unwrap_or_else(|e| die(format!("{dir}: {e}")));
        for k in [4, 6, 8] {
            let f = format!("ghb-chanter-{k}-sides.solidcraft.json");
            std::fs::write(format!("{dir}/{f}"), ghb_solidcraft::script(k)).unwrap_or_else(|e| die(format!("{dir}/{f}: {e}")));
            println!("  wrote {f}");
        }
        return;
    }
    if let Some(pt) = page_to {
        page(&pt);
        println!("  wrote {}", pc::basename(&pt));
        return;
    }
    if write && out.is_none() {
        die("error: --out=DIR, or --no-write to run the checks only.");
    }
    let (parts, info) = build(n);
    let sheets = pc::pack(&parts);
    let res = checks(n, &parts, &info, &sheets);
    println!(
        "bagpipe chanter, the Dis-Enchanter's holes on a tapering {n}-sided bore of finger-jointed strips: strips {:.1}mm long, faces {:.2} to {:.2}mm; {}mm from the seat's top to the sole",
        info.ls,
        side(info.x0, n),
        side(REF_SOLE, n),
        g(REF_SOLE)
    );
    for h in HOLES {
        let (_, z, d_ref, _) = REF_HOLES[hole_i(h)];
        let d = info.holes[hole_i(h)].2;
        let (w, ln) = shape(z, d, n);
        let how = if w == d { format!("{d:.2}mm round") } else { format!("a {w:.2} x {ln:.2}mm stadium, {d:.2}mm round's area") };
        println!(
            "  {h:<3} {:<5} strip, {z:6.1}mm down, {how} (the reference {d_ref:.2}), the face {:.2}mm",
            names(n)[if h == "hA" { n / 2 } else { 0 }],
            side(z, n)
        );
    }
    let (w, ln) = shape(REF_VENT.0, info.vent, n);
    let how = if w == info.vent { format!("{:.2}mm round", info.vent) } else { format!("{w:.2} x {ln:.2}mm stadiums, {:.2}mm round's area", info.vent) };
    let (v0, v1) = vents(n);
    println!(
        "  side holes, {} and {} strips, {:.1}mm down, {how} (the reference {:.2}), the face {:.2}mm",
        names(n)[v0],
        names(n)[v1],
        REF_VENT.0,
        REF_VENT.1,
        side(REF_VENT.0, n)
    );
    println!("  top down: {N_TENON} tenon, {N_COLLAR} collar, 1 collar socket over the strips; at the sole the round foot, {N_FOOT} layers of {n} strips");
    if pc::print_checks(&res) {
        std::process::exit(1);
    }
    if write {
        pc::write_sheets(&out.unwrap(), &sheets, &|k| format!("{}-sheet{k}-cut-files.svg", label(n)), &|k, nsh, sh| svg_sheet(k, nsh, sh, n));
    }
}

