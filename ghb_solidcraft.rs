//! The chanter as a parametric SolidCraft design (2026-11-08), one per number
//! of sides: `ghb_chanter --solidcraft=DIR` writes them, and SolidCraft builds
//! the STEP files in step/ from them.
//!
//! The frame is the 3D view's: X down the chanter from the seat's top, the
//! finger holes to -Y. Each strip leans with the cone, so it is sketched in
//! the plane through the axis along its face, drawn with its fingers, cells,
//! guards and holes as the generator draws its outline, extruded out through
//! the whole wall's reach, and then intersected with the slab between the
//! bore's face and ply outside it, which leans as the cone does. The
//! generator's x runs along the face, so the sketch lays it k_lean of that
//! down the axis, the holes' ends too: else a strip came out 0.15 mm longer
//! than its cut outline (2026-11-09). The cone,
//! the ply, the holes and their sizes are parameters; the joints follow them
//! as the generator works them out. Where the generator's tidy or its cap on
//! a finger's reach gives a different joint from the rule, the design says
//! so with bands whose ends are the same cells and guards, so they follow.
use super::*;
use lasermade_tools::solidcraft::{n as nn, Script};

/// sum of coef * (expr), the zero terms dropped.
fn lin(terms: &[(f64, &str)]) -> String {
    let t: Vec<String> = terms.iter().filter(|(c, _)| c.abs() > 1e-12).map(|(c, e)| format!("{}*({e})", nn(*c))).collect();
    if t.is_empty() { "0 mm".into() } else { t.join(" + ") }
}

/// The polygon's side at z down from the seat's top.
fn sd(z: &str) -> String {
    format!("k_side*(cone_a + cone_b*({z}))")
}
/// Its inradius.
fn ir(z: &str) -> String {
    format!("k_in*(cone_a + cone_b*({z}))")
}
/// The reference's bore across at z, as ref_d: the seat, then straight to
/// the throat, then the cone.
fn refd(z: &str) -> String {
    format!(
        "seat_a + seat_b*min({z}, seat_deep) + ((cone_a + cone_b*throat) - (seat_a + seat_b*seat_deep))/(throat - seat_deep)*min(throat - seat_deep, max(0, ({z}) - seat_deep)) + cone_b*max(0, ({z}) - throat)"
    )
}
/// The reference's bead across at z, as bead_d.
fn bead(z: &str) -> String {
    let mut e = format!("{} mm", nn(REF_BEAD[0].1));
    for w in REF_BEAD.windows(2) {
        let ((z0, d0), (z1, d1)) = (w[0], w[1]);
        e += &format!(" + {} mm*min(1, max(0, (({z}) - {} mm)/{} mm))", nn(d1 - d0), nn(z0), nn(z1 - z0));
    }
    e
}

/// A circle in the active sketch.
fn circ(s: &mut Script, c: [&str; 2], d: &str) {
    s.cmd("sketch.circle.center", format!("{{\"center\":[\"{}\",\"{}\"],\"diameter\":\"{d}\"}}", c[0], c[1]));
}

/// A sketch across the chanter at X = x, (Y, Z).
fn across(s: &mut Script, x: &str, name: &str) -> String {
    s.sketch_on([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0], Some(x), name)
}

/// Plate frame (p0, p1) to the across sketch's (Y, Z).
fn yz(p: (f64, f64)) -> (f64, f64) {
    (p.1, -p.0)
}

pub fn script(n: usize) -> String {
    let (_, _, holes, vent) = solve();
    let t = plate();
    let tn = (PI / n as f64).tan();
    let th = PI - 2.0 * PI / n as f64;
    let kb = (-th.cos()).max(0.0);
    let mut s = Script::new();
    s.len("ply", format!("{} mm", nn(t)), "plate thickness: the strips, every plate, and the joints' depth, as the generator's --plate=");
    s.len("cone_a", format!("{} mm", nn(REF_CONE.0)), "the reference's cone: across at the seat's top, as if it ran up so far");
    s.num("cone_b", nn(REF_CONE.1), "and its widening per mm down");
    s.num("k_side", nn((PI * tn / n as f64).sqrt()), &format!("read only: the {n}-sided polygon's side per mm of the round bore's diameter, the same area"));
    s.num("k_in", nn((PI * tn / n as f64).sqrt() / (2.0 * tn)), "read only: and its inradius");
    s.len("seat_a", format!("{} mm", nn(REF_SEAT.0)), "the reed's tapered seat: across at the top");
    s.num("seat_b", nn(REF_SEAT.1), "and per mm down");
    s.len("seat_deep", format!("{} mm", nn(REF_SEAT_DEEP)), "the seat's depth");
    s.len("throat", format!("{} mm", nn(REF_THROAT)), "where the cone starts, its narrowest");
    s.len("sole", format!("{} mm", nn(REF_SOLE)), "the bore's end, down from the seat's top");
    s.len("tenon_d", format!("{} mm", nn(REF_TENON_D)), "the tenon, hemped into the stock");
    s.len("x0", format!("{}*ply", N_TENON + N_COLLAR), &format!("read only: the strips' top, under {N_TENON} tenon and {N_COLLAR} collar plates"));
    s.len("ls", "sole - x0", "read only: the strips' length");
    s.num("k_lean", "1/sqrt(1 + (k_in*cone_b)*(k_in*cone_b))", "read only: the faces lean with the cone, and a strip drawn ls long along its face reaches this much of ls down the axis");
    s.len("reach", format!("ply*{}", nn(1.0 / (th / 2.0).tan())), "read only: how far a finger reaches past its face's edge");
    s.num("k_back", nn(kb), "read only: how far the strip beside it stands back from the edge, per mm of reach");
    s.len("finger", format!("{} mm", nn(sc::FINGER)), "a joint cell's length, about");
    s.len("min_finger", format!("{} mm", nn(sc::MIN_FINGER)), "no finger or gap shorter than this: a scrap beside a hole's guard goes to the guard");
    s.len("min_web", format!("{} mm", nn(pc::MIN_WEB)), "the guard round each hole, where its strip owns both corners");
    s.len("web_face", format!("{} mm", nn(WEB_FACE)), "a round hole at least this much narrower than its face; past that a stadium");
    s.num("ncell", "max(1, round(ls/finger))", "read only: joint cells along each corner");
    s.len("cell", "ls/ncell", "read only: each cell's length");
    // the holes: hA ... A, then the side holes
    let mut hs: Vec<(String, &str, f64, f64, Vec<usize>)> = holes.iter().map(|&(h, z, d, _)| (h.to_string(), h, z, d, vec![if h == "hA" { n / 2 } else { 0 }])).collect();
    let (va, vb) = vents(n);
    hs.push(("v".into(), "side", REF_VENT.0, vent, vec![va, vb]));
    for (k, (h, what, z, d, _)) in hs.iter().enumerate() {
        let k = if h == "v" { "v".to_string() } else { (k + 1).to_string() };
        let what = if *what == "side" { "the side holes".to_string() } else { format!("the {what} hole") };
        s.len(&format!("z_{k}"), format!("{} mm", nn(*z)), &format!("{what}, down from the seat's top: the reference's"));
        s.len(&format!("d_{k}"), format!("{} mm", nn(*d)), "the round hole's diameter, the solver's for the reference's note");
        // shape(): as wide as the face allows at the hole's top, long enough for the area
        let mut l = format!("d_{k}");
        for it in 1..=3 {
            let w = format!("w{it}_{k}");
            s.len(&w, format!("min(d_{k}, {} - web_face)", sd(&format!("z_{k} - ({l})/2"))), "read only: its width, worked toward the face's at its top");
            let nl = format!("l{it}_{k}");
            s.len(&nl, format!("{w} + max(0, pi*(d_{k}^2 - {w}^2)/4/{w})"), "read only: and its length, for the round hole's area");
            l = nl;
        }
        s.len(&format!("xh_{k}"), format!("z_{k} - x0"), "read only: its centre down the strip");
    }
    // each guard, raw: MIN_WEB past its hole; grown over a short scrap to the
    // cell's edge, but never into the next guard
    let keys: Vec<String> = hs.iter().enumerate().map(|(k, e)| if e.0 == "v" { "v".to_string() } else { (k + 1).to_string() }).collect();
    let ra = |k: &str| format!("(xh_{k} - l3_{k}/2 - min_web)");
    let rb = |k: &str| format!("(xh_{k} + l3_{k}/2 + min_web)");
    for (q, k) in keys.iter().enumerate() {
        let (a, b) = (ra(k), rb(k));
        let fl = format!("({a} - floor({a}/cell)*cell)");
        let fr = format!("(ceil({b}/cell)*cell - {b})");
        let mut g0 = format!("{a} - (1 - min(1, floor({fl}/min_finger)))*{fl}");
        let mut g1 = format!("{b} + (1 - min(1, floor({fr}/min_finger)))*{fr}");
        if q > 0 {
            g0 = format!("max({g0}, min({a}, {}))", rb(&keys[q - 1]));
        }
        if q + 1 < keys.len() {
            g1 = format!("min({g1}, max({b}, {}))", ra(&keys[q + 1]));
        }
        s.len(&format!("g0_{k}"), g0, "read only: where its guard starts down the strip");
        s.len(&format!("g1_{k}"), g1, "read only: and ends");
    }
    // between two guards one cell's edge apart with both scraps short, the
    // generator's tidy gives both to one guard, the shorter scrap's first;
    // with no cell's edge between them the scrap keeps its cell's owner
    for w in keys.windows(2) {
        let (i, j) = (&w[0], &w[1]);
        let (b, a) = (rb(i), ra(j));
        let p = format!("{i}_{j}");
        s.num(&format!("jl_{p}"), format!("floor({b}/cell)"), "read only: the cell the scrap after one guard starts in");
        s.num(&format!("jr_{p}"), format!("floor({a}/cell)"), "read only: and the cell the next guard starts in");
        s.len(&format!("c_{p}"), format!("(jl_{p} + 1)*cell"), "read only: the cell's edge between them");
        s.num(&format!("one_{p}"), format!("1 - min(1, abs(jr_{p} - jl_{p} - 1))"), "read only: 1 if one cell's edge is between them");
        s.num(&format!("none_{p}"), format!("1 - min(1, abs(jr_{p} - jl_{p}))"), "read only: 1 if none is");
        s.num(&format!("both_{p}"), format!("(1 - min(1, floor(max(0, c_{p} - {b})/min_finger)))*(1 - min(1, floor(max(0, {a} - c_{p})/min_finger)))"), "read only: 1 if both scraps are short");
        s.num(&format!("lf_{p}"), format!("1 - min(1, ceil(max(0, (c_{p} - {b}) - ({a} - c_{p}))/(0.000001 mm)))"), "read only: 1 if the first scrap is the shorter, tidied first");
        s.num(&format!("par_{p}"), format!("mod(jl_{p}, 2)"), "read only: that cell's parity");
    }
    s.len("hole_room", format!("{} - web_face - d_1", sd("z_1 - l3_1/2")), "read only: hA's face past a round hole, below 0 a stadium");
    s.len("foot_d", format!("{} mm", nn(FOOT_D)), "the round foot, the reference's sole");
    s.len("pin", format!("{} mm", nn(pc::PIN)), "the tenon's and collar's brass pins");
    s.len("pin_interf", format!("{} mm", nn(pc::PIN_INTERF)), "their holes this much under the pin, a press-fit");
    s.len("r_pin", "(seat_a/2 + tenon_d/2)/2", "read only: the pins' distance from the axis, midway between the seat and the tenon's edge");
    let ncell0 = py::round((REF_SOLE - (N_TENON + N_COLLAR) as f64 * t) / sc::FINGER).max(1) as usize;
    // each guard's corners: owners() for each hole, its strips own both their corners and their neighbours keep their others
    let mine = |gs: &[usize]| -> Vec<Option<usize>> {
        let mut m: Vec<Option<usize>> = vec![None; n];
        for &st in gs {
            m[(st + n - 1) % n] = Some(st);
            m[st] = Some(st);
        }
        for &st in gs {
            let c = (st + 1) % n;
            m[c].get_or_insert(c);
            m[(st + 2 * n - 2) % n].get_or_insert((st + n - 1) % n);
        }
        m
    };
    // The rule above, in numbers, against the generator's own runs: its tidy
    // takes the shortest scrap first, which the guards' growth does not
    // always match (two short scraps between two guards). Where they differ,
    // a band gives the corner to the generator's owner, its ends as
    // expressions of the same parameters.
    let x0n = (N_TENON + N_COLLAR) as f64 * t;
    let lsn = REF_SOLE - x0n;
    let ncn = py::round(lsn / sc::FINGER).max(1) as usize;
    let celln = lsn / ncn as f64;
    let mut cand: Vec<(f64, String)> = (0..=ncn).map(|j| (j as f64 * celln, format!("{j}*cell"))).collect();
    let mut raw: Vec<(f64, f64)> = vec![];
    for (k, (h, _, z, d, _)) in hs.iter().enumerate() {
        let k = if h == "v" { "v".to_string() } else { (k + 1).to_string() };
        let (_, l) = shape(*z, *d, n);
        let (a, b) = (z - x0n - l / 2.0 - pc::MIN_WEB, z - x0n + l / 2.0 + pc::MIN_WEB);
        cand.push((a, format!("(xh_{k} - l3_{k}/2 - min_web)")));
        cand.push((b, format!("(xh_{k} + l3_{k}/2 + min_web)")));
        raw.push((a, b));
    }
    let mut gnum: Vec<(f64, f64, Vec<Option<usize>>)> = vec![];
    for (q, (h, _, _, _, gs)) in hs.iter().enumerate() {
        let k = if h == "v" { "v".to_string() } else { (q + 1).to_string() };
        let (a, b) = raw[q];
        let fl = a - (a / celln).floor() * celln;
        let fr = (b / celln).ceil() * celln - b;
        let mut g0 = if fl < sc::MIN_FINGER { a - fl } else { a };
        let mut g1 = if fr < sc::MIN_FINGER { b + fr } else { b };
        if q > 0 {
            g0 = g0.max(a.min(raw[q - 1].1));
        }
        if q + 1 < raw.len() {
            g1 = g1.min(b.max(raw[q + 1].0));
        }
        cand.push((g0, format!("g0_{k}")));
        cand.push((g1, format!("g1_{k}")));
        gnum.push((g0, g1, mine(gs)));
    }
    // the between-guards rule, in numbers: (corner, pair, owner, on, lo, hi)
    let mut betw: Vec<(usize, usize, usize, f64, f64, f64)> = vec![];
    for q in 0..raw.len() - 1 {
        let (b, a) = (raw[q].1, raw[q + 1].0);
        let (jl, jr) = ((b / celln).floor(), (a / celln).floor());
        let c = (jl + 1.0) * celln;
        let one = if (jr - jl - 1.0).abs() < 0.5 { 1.0 } else { 0.0 };
        let none = if (jr - jl).abs() < 0.5 { 1.0 } else { 0.0 };
        let both = if (c - b).max(0.0) < sc::MIN_FINGER && (a - c).max(0.0) < sc::MIN_FINGER { 1.0 } else { 0.0 };
        let lf = if (c - b) - (a - c) <= 0.0 { 1.0 } else { 0.0 };
        let par = jl.rem_euclid(2.0);
        for cn in 0..n {
            let (Some(oi), Some(oj)) = (gnum[q].2[cn], gnum[q + 1].2[cn]) else { continue };
            if oi == oj {
                continue;
            }
            let pb = if oj == cn { 0.0 } else { 1.0 };
            let case_b = 1.0 - (par - pb).abs();
            let base_is = |o: usize| if o == cn { 1.0 - par } else { par };
            betw.push((cn, q, oi, one * both * case_b * lf + none * base_is(oi), b, a));
            betw.push((cn, q, oj, one * both * case_b * (1.0 - lf) + none * base_is(oj), b, a));
        }
    }
    let own = owners(lsn, x0n, &holes_of(&holes, vent, n), n);
    let sym = |v: f64| -> String {
        cand.iter().find(|c| (c.0 - v).abs() < 1e-6).map(|c| c.1.clone()).unwrap_or_else(|| die(format!("error: a run's end at {v} is no cell's or guard's.")))
    };
    let mut fixes: Vec<(usize, String, String, usize)> = vec![]; // corner, from, to, owner
    for c in 0..n {
        let mut xs: Vec<f64> = own[c].iter().flat_map(|r| [r.0, r.1]).chain(cand.iter().map(|q| q.0)).filter(|&x| (0.0..=lsn).contains(&x)).collect();
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        xs.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
        for w in xs.windows(2) {
            let mid = (w[0] + w[1]) / 2.0;
            let gen = own[c].iter().find(|r| r.0 <= mid && mid <= r.1).unwrap().2;
            let j = (mid / celln).floor() as usize;
            let mut rule = if j % 2 == 0 { c } else { (c + 1) % n };
            if let Some(o) = gnum.iter().find(|g| g.0 <= mid && mid <= g.1 && g.2[c].is_some()).and_then(|g| g.2[c]) {
                rule = o;
            }
            if let Some(e) = betw.iter().find(|e| e.0 == c && e.3 > 0.5 && e.4 <= mid && mid <= e.5) {
                rule = e.2;
            }
            if rule != gen {
                match fixes.last_mut() {
                    Some(f) if f.0 == c && f.3 == gen && (cand.iter().find(|q| q.1 == f.2).map(|q| q.0).unwrap_or(-1.0) - w[0]).abs() < 1e-9 => f.2 = sym(w[1]),
                    _ => fixes.push((c, sym(w[0]), sym(w[1]), gen)),
                }
            }
        }
    }
    // and where the generator caps a reach: a strip owning neither of its
    // corners, too narrow for both its neighbours' stand-back
    let mut caps: Vec<(usize, String, String, usize, String)> = vec![];
    let (full, _) = corner(t, n);
    for (c, runs) in reaches(&own, x0n, t, n).iter().enumerate() {
        for &(a, b, o, r) in runs {
            if r < full - 1e-9 {
                let (sa, sb) = (sym(a), sym(b));
                let cap = format!("min(reach, ({} - {} mm - 0.05 mm)/(2*k_back))", sd(&format!("x0 + {sa}")), nn(MIN_STRIP));
                caps.push((c, sa, sb, o, cap));
            }
        }
    }
    let s0 = format!("{} - 1 mm", ir("x0"));
    let depth = format!("{} - {} + ply + 2 mm", ir("sole"), ir("x0"));
    for i in 0..n {
        let body = format!("{} strip", names(n)[i]);
        let (nx, ny) = normal(i, n);
        let nw = [0.0, ny, -nx];
        let yd = [0.0, -nx, -ny];
        // a polygon in the strip's (x, u) frame, x down from its top
        let sk = |s: &mut Script, name: &str| s.sketch_on([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], yd, Some(&s0), name);
        // (x along the strip's face, as the generator draws it: down the
        // axis, k_lean of that)
        let pt = |x: &str, u: &str| [format!("x0 + ({x})*k_lean"), format!("-({u})")];
        // the band along an edge from x a to b, between the face's edge less k_back reach and `out` past it
        let band = |x0: &str, x1: &str, sg: f64, out: &str| -> Vec<[String; 2]> {
            let e = |x: &str, extra: &str| format!("{}*({}/2 + {extra})", nn(sg), sd(&format!("x0 + ({x})")));
            vec![pt(x0, &e(x0, "-k_back*reach")), pt(x1, &e(x1, "-k_back*reach")), pt(x1, &e(x1, out)), pt(x0, &e(x0, out))]
        };
        let k0 = sk(&mut s, &format!("{body} sketch"));
        let full = |x: &str, sg: f64| format!("{}*({}/2 + reach)", nn(sg), sd(&format!("x0 + ({x})")));
        s.poly(&[pt("0 mm", &full("0 mm", -1.0)), pt("ls", &full("ls", -1.0)), pt("ls", &full("ls", 1.0)), pt("0 mm", &full("0 mm", 1.0))]);
        s.finish();
        s.extrude(&k0, &depth, false, "new", &[], &body);
        // the cells it gives up: on its low edge (corner i - 1) the even ones, on its high (corner i) the odd
        let kc = sk(&mut s, &format!("{body}, cells sketch"));
        for j in 0..ncell0 + 3 {
            let lift = if j < ncell0 - 2 { String::new() } else { format!(" + (1 - min(1, max(0, ncell - {j})))*1000 mm") };
            let (a, b) = (format!("{j}*cell{lift}"), format!("{}*cell{lift}", j + 1));
            s.poly(&band(&a, &b, if j % 2 == 0 { -1.0 } else { 1.0 }, "reach + 1 mm"));
        }
        s.finish();
        s.extrude(&kc, &depth, false, "cut", &[&body], &format!("{body}, cells"));
        // the guards: given back, or taken
        let mut joins = vec![];
        let mut cuts = vec![];
        for (k, (h, _, _, _, gs)) in hs.iter().enumerate() {
            let k = if h == "v" { "v".to_string() } else { (k + 1).to_string() };
            let m = mine(gs);
            for (c, sg) in [((i + n - 1) % n, -1.0), (i, 1.0)] {
                if let Some(o) = m[c] {
                    let (g0, g1) = (format!("g0_{k}"), format!("g1_{k}"));
                    if o == i {
                        joins.push(band(&g0, &g1, sg, "reach"));
                    } else {
                        cuts.push(band(&g0, &g1, sg, "reach + 1 mm"));
                    }
                }
            }
        }
        // between close guards, as the tidy has it: each band active or parked
        // on its owner's own guard, where it changes nothing
        let (mut bj, mut bc) = (vec![], vec![]);
        for q in 0..keys.len() - 1 {
            let (ki, kj) = (&keys[q], &keys[q + 1]);
            let p = format!("{ki}_{kj}");
            let (mi, mj) = (mine(&hs[q].4), mine(&hs[q + 1].4));
            for (cn, sg) in [((i + n - 1) % n, -1.0), (i, 1.0)] {
                let (Some(oi), Some(oj)) = (mi[cn], mj[cn]) else { continue };
                if oi == oj {
                    continue;
                }
                let pb = if oj == cn { 0 } else { 1 };
                let case_b = format!("(1 - abs(par_{p} - {pb}))");
                for (o, kk, lfx) in [(oi, ki, format!("lf_{p}")), (oj, kj, format!("(1 - lf_{p})"))] {
                    let base_is = if o == cn { format!("(1 - par_{p})") } else { format!("par_{p}") };
                    let on = format!("(one_{p}*both_{p}*{case_b}*{lfx} + none_{p}*{base_is})");
                    let lo = format!("{on}*{} + (1 - {on})*{}", rb(ki), ra(kk));
                    let hi = format!("{on}*{} + (1 - {on})*{}", ra(kj), rb(kk));
                    if o == i {
                        bj.push(band(&lo, &hi, sg, "reach"));
                    } else {
                        bc.push(band(&lo, &hi, sg, "reach + 1 mm"));
                    }
                }
            }
        }
        // then where the generator's tidy differs, after the guards
        let (mut fj, mut fc) = (vec![], vec![]);
        for (c, a, b, o) in &fixes {
            for (cc, sg) in [((i + n - 1) % n, -1.0), (i, 1.0)] {
                if cc == *c {
                    if *o == i {
                        fj.push(band(a, b, sg, "reach"));
                    } else {
                        fc.push(band(a, b, sg, "reach + 1 mm"));
                    }
                }
            }
        }
        // the capped reaches: the owner's finger shorter, the other strip standing back less
        let (mut cj, mut cc_) = (vec![], vec![]);
        for (c, a, b, o, cap) in &caps {
            for (cc, sg) in [((i + n - 1) % n, -1.0), (i, 1.0)] {
                if cc != *c {
                    continue;
                }
                let e = |x: &str, extra: &str| format!("{}*({}/2 + {extra})", nn(sg), sd(&format!("x0 + ({x})")));
                if *o == i {
                    let (lo, hi) = (format!("{cap}"), "reach + 1 mm".to_string());
                    cc_.push(vec![pt(a, &e(a, &lo)), pt(b, &e(b, &lo)), pt(b, &e(b, &hi)), pt(a, &e(a, &hi))]);
                } else {
                    let (lo, hi) = ("-k_back*reach - 0.5 mm".to_string(), format!("-k_back*{cap}"));
                    cj.push(vec![pt(a, &e(a, &lo)), pt(b, &e(b, &lo)), pt(b, &e(b, &hi)), pt(a, &e(a, &hi))]);
                }
            }
        }
        for (polys, op, what) in [(joins, "join", "guards"), (cuts, "cut", "guards"), (bj, "join", "between"), (bc, "cut", "between"), (fj, "join", "tidy"), (fc, "cut", "tidy"), (cj, "join", "capped"), (cc_, "cut", "capped")] {
            if polys.is_empty() {
                continue;
            }
            let kg = sk(&mut s, &format!("{body}, {what} {op} sketch"));
            for p in &polys {
                s.poly(p);
            }
            s.finish();
            s.extrude(&kg, &depth, false, op, &[&body], &format!("{body}, {what} {op}"));
        }
        // the holes: two round ends along the wall, and between them a stadium's middle
        let mine_h: Vec<String> = hs.iter().enumerate().filter(|(_, e)| e.4.contains(&i)).map(|(k, e)| if e.0 == "v" { "v".to_string() } else { (k + 1).to_string() }).collect();
        for k in &mine_h {
            let half = format!("(l3_{k} - w3_{k})/2");
            for (e, sg) in [(1, "-"), (2, "+")] {
                let x = format!("x0 + (xh_{k} {sg} {half})*k_lean");
                s.cylinder(&format!("{body}, hole {k} end {e}"), [x, lin(&[(nw[1], &s0)]), lin(&[(nw[2], &s0)])], Some(nw), &format!("w3_{k}"), &depth, "cut", &[&body]);
            }
        }
        if !mine_h.is_empty() {
            let kh = sk(&mut s, &format!("{body}, stadia sketch"));
            for k in &mine_h {
                let half = format!("(l3_{k} - w3_{k})/2");
                let on = format!("min(1, floor((l3_{k} - w3_{k})/(0.001 mm)))");
                let lift = format!("(1 - {on})*1000 mm");
                let (a, b) = (format!("xh_{k} - {half} + {lift}"), format!("xh_{k} - {half} + max(2*{half}, 0.01 mm) + {lift}"));
                let w = format!("w3_{k}/2");
                s.poly(&[pt(&a, &format!("-{w}")), pt(&b, &format!("-{w}")), pt(&b, &w), pt(&a, &w)]);
            }
            s.finish();
            s.extrude(&kh, &depth, false, "cut", &[&body], &format!("{body}, stadia"));
        }
        // the wall: from the bore's face out ply, leaning with the cone
        let kw = s.sketch_on([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], nw, None, &format!("{body}, wall sketch"));
        let (xa, xb) = ("x0 - 1 mm", "sole + 1 mm");
        s.poly(&[[xa.to_string(), ir(xa)], [xb.to_string(), ir(xb)], [xb.to_string(), format!("{} + ply", ir(xb))], [xa.to_string(), format!("{} + ply", ir(xa))]]);
        s.finish();
        s.cmd(
            "solid.extrude",
            format!("{{\"sketch\":\"{kw}\",\"distance\":\"4*foot_d\",\"direction\":\"symmetric\",\"operation\":\"intersect\",\"targets\":[\"{body}\"],\"name\":\"{body}, wall\"}}"),
        );
    }
    // the foot: each layer a pinwheel, each piece the foot's disc past its
    // strip's face and short of the next one's, the other hand each layer
    for k in 0..N_FOOT {
        let x = format!("sole - {}*ply", k + 1);
        let r = format!("({} + ply)", ir(&format!("sole - {}*ply", k as f64 + 0.5)));
        for j in 0..n {
            let name = format!("foot {}, piece {}", k + 1, j + 1);
            s.cylinder(&name, [x.clone(), "0 mm".into(), "0 mm".into()], Some([1.0, 0.0, 0.0]), "foot_d", "ply", "new", &[]);
            let mirror = |p: (f64, f64)| if k % 2 == 1 { (-p.0, p.1) } else { p };
            for (e, f, keep_out) in [(1, j, true), (2, (j + 1) % n, false)] {
                let nv = yz(mirror(normal(f, n)));
                let tv = (-nv.1, nv.0);
                // the half-plane to cut away: inside face f's line (keep_out) or outside it
                let far = if keep_out { -2.0 } else { 2.0 };
                let p = |a: f64, b: f64| -> [String; 2] {
                    [lin(&[(nv.0, &r), (a * tv.0 + b * nv.0, "foot_d")]), lin(&[(nv.1, &r), (a * tv.1 + b * nv.1, "foot_d")])]
                };
                let kk = across(&mut s, &format!("{x} - 1 mm"), &format!("{name}, edge {e} sketch"));
                s.poly(&[p(1.0, 0.0), p(-1.0, 0.0), p(-1.0, far), p(1.0, far)]);
                s.finish();
                s.extrude(&kk, "ply + 2 mm", false, "cut", &[&name], &format!("{name}, edge {e}"));
            }
        }
    }
    // the tenon, the collar's round plates and its socket plate
    let mut pinned = vec![];
    for k in 0..N_TENON {
        let name = format!("tenon plate {}", k + 1);
        let x = format!("{k}*ply");
        let kk = across(&mut s, &x, &format!("{name} sketch"));
        circ(&mut s, ["0 mm", "0 mm"], "tenon_d");
        circ(&mut s, ["0 mm", "0 mm"], &refd(&format!("{}*ply", nn(k as f64 + 0.5))));
        s.finish();
        s.extrude(&kk, "ply", false, "new", &[], &name);
        pinned.push(name);
    }
    for k in 0..N_COLLAR {
        let name = format!("collar plate {}", k + 1);
        let x = format!("{}*ply", N_TENON + k);
        let mid = format!("{}*ply", nn((N_TENON + k) as f64 + 0.5));
        let kk = across(&mut s, &x, &format!("{name} sketch"));
        circ(&mut s, ["0 mm", "0 mm"], &bead(&mid));
        circ(&mut s, ["0 mm", "0 mm"], &refd(&mid));
        s.finish();
        s.extrude(&kk, "ply", false, "new", &[], &name);
        pinned.push(name);
    }
    let kk = across(&mut s, "x0", "collar socket sketch");
    circ(&mut s, ["0 mm", "0 mm"], &bead("x0 + ply/2"));
    let rv = format!("({} + ply)/{}", ir("x0"), nn((PI / n as f64).cos()));
    let pts: Vec<[String; 2]> = (0..n)
        .map(|i| {
            let (a, b) = (normal(i, n), normal((i + 1) % n, n));
            let mm = (a.0 + b.0).hypot(a.1 + b.1);
            let v = yz(((a.0 + b.0) / mm, (a.1 + b.1) / mm));
            [lin(&[(v.0, &rv)]), lin(&[(v.1, &rv)])]
        })
        .collect();
    s.poly(&pts);
    s.finish();
    s.extrude(&kk, "ply", false, "new", &[], "collar socket");
    let tg: Vec<&str> = pinned.iter().map(|x| x.as_str()).collect();
    for (k, ang) in [(1, 45.0f64), (2, 225.0)] {
        // polar(ang, r) lands at Y = -r cos, Z = -r sin
        let a = ang.to_radians();
        s.cylinder(&format!("pin {k}"), ["-1 mm".to_string(), lin(&[(-a.cos(), "r_pin")]), lin(&[(-a.sin(), "r_pin")])], Some([1.0, 0.0, 0.0]), "pin - pin_interf", "x0 + 1 mm", "cut", &tg);
    }
    s.measure();
    s.text()
}
