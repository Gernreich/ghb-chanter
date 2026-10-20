#!/usr/bin/env python3
"""Cut files for a Great Highland bagpipe chanter whose conical bore is a
tapering square of four long strips of ply, glued, from the sole up into a
collar of stacked plates, with a stacked tenon and reed seat above it.

    python3 ghb_chanter.py --no-write                  # the checks
    python3 ghb_chanter.py --out=DIR                   # the sheet
    python3 ghb_chanter.py --plate=2.9 ...             # for ply as measured
    python3 ghb_chanter.py --kerf=0.18 ...             # for that kerf
    python3 ghb_chanter.py --page-to=turn.html         # the 3D view

THE REFERENCE is Jeremy Starcher's Dis-Enchanter (2017, Thingiverse thing
2373962), a 3D-printed chanter copied from a 1970s Hardie, measured from its
STL: a round conical bore from a 4.76mm throat to 21.2mm at the sole, eight
finger holes and two side holes. It was printed in two pieces; the joint is
ignored. Its designer calls it mostly working, with F and G low on the first
print, so it is a model to follow, not a proven scale.

The bore here is the reference's cone made square, the same area at every
height, so the strips taper: each strip's face is the square's side, wider
toward the sole. The holes stay where the reference's are. They go through a
3mm strip instead of the reference's 2mm of wall, so each hole's size is
solved for its note to sound where the reference's does, in chanter_model.py's
terms, and the side holes' size for low G. The model treats the bore as an
exact cone, closed at its apex for the reed; it is used only for the
difference between the two chanters, not for the pitch. (The cone closed at
its apex stands in for the reed: a conical bore sounds as the whole cone.)

From the sole up:

  * the STRIPS, front, back and two sides, tapering, from the sole into the
    collar, finger-jointed and glued; round each hole its strip owns both
    corners. There is no butt-jointed version: the holes are too wide for
    their face, high A's and the side holes within a millimetre of its edge,
    to keep ply round them unless their strip owns the corners;
  * the FOOT, N_FOOT layers of strips lying flat round the strips' bottom,
    four to a layer, each layer turned the other way to the one below, each
    strip's outer edge an arc of the reference's sole, FOOT_D round;
  * the COLLAR: a plate with a square hole the strips' top stands in, glued
    to the strips; then N_COLLAR plates with the reference's round throat,
    each as wide as the reference's bead at its height;
  * the TENON: N_TENON round plates with the reed's tapered seat through
    them, which goes into the chanter stock with hemp. The collar's round
    plates and the tenon press together on two pins.

And a JIG, not part of the chanter: a base plate and N_FOOT rings with the
foot's circle cut through them, pinned into a cup the foot is glued up in.
"""
import cmath
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, '..', 'practice-chanters'))
import practice_chanters as P                                  # noqa: E402
import strip_chanter as SC                                     # noqa: E402
from practice_chanters import M, SF, Part, circle              # noqa: E402

KERF = P.KERF
# --- the reference chanter, measured (mm, down from the seat's top) -----------
REF_SEAT = (7.155, -0.0991)        # the reed's seat, tapered: across at 0, per mm down
REF_SEAT_DEEP = 22.0
REF_THROAT = 25.25                 # the cone's narrowest, 4.76mm
REF_CONE = (3.463, 0.05132)        # the bore below: across at 0, per mm down
REF_SOLE = 345.0                   # the bore's end; the print's base past it is ignored
REF_WALL = 2.0                     # the wall at every hole
REF_TENON_D = 17.0                 # hemped into the stock
# the bead the stock's top meets: (down, across) on its outside
REF_BEAD = [(22.25, 34.12), (24.25, 33.82), (26.25, 33.04), (28.25, 31.73), (30.25, 29.83),
            (32.25, 27.23), (34.25, 23.75), (36.25, 18.69), (37.25, 15.21), (38.25, 10.34)]
# each hole: down from the seat's top, across, and the strip it is in
REF_HOLES = {'hA': (64.30, 4.80, 'back'), 'G': (79.60, 4.80, 'front'),
             'F': (104.10, 6.00, 'front'), 'E': (128.57, 7.15, 'front'),
             'D': (153.05, 7.20, 'front'), 'C': (175.30, 7.90, 'front'),
             'B': (203.63, 7.85, 'front'), 'A': (234.23, 7.85, 'front')}
REF_VENT = (286.20, 9.60)          # two side holes, always open: they set low G
REF_FOOT_D, REF_FOOT_H = 36.5, 7.25  # the sole's bead
# --- this chanter ---------------------------------------------------------------
SQ = math.sqrt(math.pi) / 2        # a square's side for a circle's area, per diameter
N_TENON = 7                        # 21mm of seat, the reference's 22
N_COLLAR = 4                       # round plates over the socket plate
FOOT_D = REF_FOOT_D
N_FOOT = 2                         # 6mm, the reference's sole 7.25mm
JIG = FOOT_D + 10.0
JIG_CLEAR = 0.3
VENT_STRIPS = ('left', 'right')
OPPOSITE = {'front': 'back', 'back': 'front', 'left': 'right', 'right': 'left'}


def ref_d(z):
    """The reference's bore across, z down from the seat's top."""
    if z < REF_SEAT_DEEP:
        return REF_SEAT[0] + REF_SEAT[1] * z
    cone = REF_CONE[0] + REF_CONE[1] * z
    if z < REF_THROAT:
        top = REF_SEAT[0] + REF_SEAT[1] * REF_SEAT_DEEP
        return top + (cone - top) * (z - REF_SEAT_DEEP) / (REF_THROAT - REF_SEAT_DEEP)
    return cone


def bead_d(z):
    """The reference's bead across, z down, straight between the measures."""
    for (z0, d0), (z1, d1) in zip(REF_BEAD, REF_BEAD[1:]):
        if z <= z1:
            return d0 + (d1 - d0) * (max(z, z0) - z0) / (z1 - z0)
    return REF_BEAD[-1][1]


def side(z):
    """This chanter's square bore across, z down: the reference's cone's area."""
    return SQ * (REF_CONE[0] + REF_CONE[1] * z)


# --- the model: an exact cone, closed near its apex, holes along it ------------
APEX = -REF_CONE[0] / REF_CONE[1]  # the cone's apex, z down (above the seat)


class Hole:
    """What chanter_model.hole wants of the bore at a hole."""

    def __init__(self, z):
        self.a = (REF_CONE[0] + REF_CONE[1] * z) / 2000

    def hole_r(self, w):
        return w / 2000


def phi(k, w, z):
    """The cone's two spherical waves at z, cos(kx)/x and sin(kx)/x, x from
    the apex: (p, U) of each as columns."""
    x = (z - APEX) / 1000
    S = math.pi * (REF_CONE[1] / 2) ** 2 * x * x       # the slope has no unit
    c, s = cmath.cos(k * x), cmath.sin(k * x)
    da, db = -k * s / x - c / (x * x), k * c / x - s / (x * x)
    q = -S / (1j * w * M.RHO)
    return ((c / x, s / x), (q * da, q * db))


def cone(f, z1, z2):
    """The cone's transfer matrix from z2 up to z1, (p, U), spherical waves."""
    w = 2 * math.pi * f
    rh = (REF_CONE[0] + REF_CONE[1] * (z1 + z2) / 2) / 4000
    k = w / M.C - 1j * 3.0e-5 * math.sqrt(f) / rh

    a, b = phi(k, w, z1), phi(k, w, z2)
    det = b[0][0] * b[1][1] - b[0][1] * b[1][0]
    inv = ((b[1][1] / det, -b[0][1] / det), (-b[1][0] / det, b[0][0] / det))
    return M.mul(a, inv)


def admittance(f, holes, opened):
    """Im(Y) at the seat's top, the bore below it and the cone above it to
    its closed apex together; it crosses zero, rising, at each resonance.
    holes (name, z down, across, wall): the vents always open, the named
    holes open."""
    z = 0.0
    m = ((1, 0), (0, 1))
    for name, zh, d, wall in sorted(holes, key=lambda h: h[1]):
        m = M.mul(m, cone(f, z, zh))
        m = M.mul(m, M.hole(Hole(zh), f, d, wall, name == 'vent' or name in opened))
        z = zh
    m = M.mul(m, cone(f, z, REF_SOLE))
    a = (REF_CONE[0] + REF_CONE[1] * REF_SOLE) / 2000
    z0 = M.RHO * M.C / (math.pi * a * a)
    ka = 2 * math.pi * f / M.C * a
    zr = z0 * (0.25 * ka ** 2 + 1j * 0.6133 * ka)
    zin = (m[0][0] * zr + m[0][1]) / (m[1][0] * zr + m[1][1])
    # above the seat: the cone closed at its apex carries sin(kx)/x alone
    w = 2 * math.pi * f
    rh = (REF_CONE[0] / 2) / 2000
    k = w / M.C - 1j * 3.0e-5 * math.sqrt(f) / rh
    (pa, pb), (ua, ub) = phi(k, w, 0.0)
    return (1 / zin - ub / pb).imag


def sounding(holes, note, near):
    """The fingering's resonance nearest `near`."""
    fing = M.FINGERING[note]
    lo, hi = near / 1.25, near * 1.25
    Y = lambda f: admittance(f, holes, fing)
    best, pf, py = None, lo, Y(lo)
    for i in range(1, 61):
        f = lo + (hi - lo) * i / 60
        y = Y(f)
        if py < 0 <= y:
            a, c = pf, f
            for _ in range(30):
                mid = (a + c) / 2
                if Y(mid) < 0:
                    a = mid
                else:
                    c = mid
            r = (a + c) / 2
            if best is None or abs(r - near) < abs(best - near):
                best = r
        pf, py = f, y
    if best is None:
        raise ValueError(f'{note}: no resonance near {near:.0f}Hz')
    return best


def ref_holes():
    return ([(h, z, d, REF_WALL) for h, (z, d, st) in REF_HOLES.items()]
            + [('vent', REF_VENT[0], REF_VENT[1], REF_WALL)] * 2)


def reference():
    """The reference's notes, in the model."""
    holes = ref_holes()
    low_a = sounding(holes, 'low A', 480.0)
    # each note is the reference's own nearest resonance to its scale step
    return {nt: sounding(holes, nt, low_a * r) for nt, r in zip(M.NOTES, M.RATIOS)}


def bisect(fn, goal, lo, hi, rising):
    for _ in range(30):
        mid = (lo + hi) / 2
        if (fn(mid) < goal) == rising:
            lo = mid
        else:
            hi = mid
    return (lo + hi) / 2


def solve():
    """Each hole's size, and the vents', that sound the reference's notes,
    the holes where the reference's are, through a strip wall."""
    ref = reference()
    t = M.PLATE
    note_of = dict(zip(M.HOLES, M.SETS))
    holes = {h: [h, z, d, t] for h, (z, d, st) in REF_HOLES.items()}
    vent = [REF_VENT[1]]
    every = lambda: list(holes.values()) + [('vent', REF_VENT[0], vent[0], t)] * 2
    for _ in range(3):
        def fv(d):
            vent[0] = d
            return sounding(every(), 'low G', ref['low G'])
        vmax = side(REF_VENT[0]) - 0.35
        if fv(vmax) < ref['low G']:
            raise SystemExit(f'error: low G is flat even with {vmax:.2f}mm side holes.')
        vent[0] = bisect(fv, ref['low G'], M.D_MIN, vmax, rising=True)
        for h in M.HOLES:
            nt = note_of[h]
            d_max = side(holes[h][1]) - 0.35

            def f(d):
                holes[h][2] = d
                return sounding(every(), nt, ref[nt])
            if f(d_max) < ref[nt]:
                raise SystemExit(f'error: {h} is flat even {d_max:.2f}mm across.')
            holes[h][2] = bisect(f, ref[nt], M.D_MIN, d_max, rising=True)
    got = {nt: sounding(every(), nt, ref[nt]) for nt in M.NOTES}
    return ref, got, holes, vent[0]


# --- the strips ----------------------------------------------------------------
def owners(Ls, holes):
    """Each corner's owner along the strips, as [(x0, x1, strip, guarded)]
    from the strips' top: fingers along one edge and the other in turn, and
    round each hole its strip owns both its corners, MIN_WEB past the hole."""
    n = max(1, int(round(Ls / SC.FINGER)))
    edges = [Ls * i / n for i in range(n + 1)]
    base = {'FL': ('front', 'left'), 'FR': ('right', 'front'),
            'KL': ('left', 'back'), 'KR': ('back', 'right')}
    guard = [(x - d / 2 - P.MIN_WEB, x + d / 2 + P.MIN_WEB, st) for st, x, d in holes]
    cuts = sorted(set(edges + [g for a, b, s in guard for g in (a, b)]))
    out = {}
    for c in SC.CORNERS:
        iv = []
        for x0, x1 in zip(cuts, cuts[1:]):
            mid = (x0 + x1) / 2
            i = min(int(mid / Ls * n), n - 1)
            who = base[c][i % 2]
            gs = [s for a, b, s in guard if a <= mid <= b]
            mine = [s for s in gs if c in SC.STRIPS[s]]
            if mine:
                who = mine[0]
            elif gs:
                # the corner beyond a guarded strip goes to the side strip,
                # not the one opposite the hole
                who = next(s for s in base[c] if s != OPPOSITE[gs[0]])
            iv.append([x0, x1, who, bool(gs)])
        out[c] = SC.tidy(iv)
    return out


def strip_outline(Ls, lo, hi, x0, t):
    """A strip from its top, x = 0, to the sole, x = Ls; u across it from its
    centre line, the bore's face out to side/2 each way, a finger out t more
    where the strip owns that corner. The face widens with the cone, so every
    edge slopes. lo, hi: its edges' runs, [(x0, x1, owns)]."""
    def edge(runs, sign):
        pts = []
        for a, b, owns in runs:
            for x in (a, b):
                pts.append((x, sign * (side(x0 + x) / 2 + (t if owns else 0.0))))
        return pts
    pts = edge(lo, -1) + edge(hi, 1)[::-1]
    out = []
    for p in pts:
        if out and abs(out[-1][0] - p[0]) < 1e-9 and abs(out[-1][1] - p[1]) < 1e-9:
            continue
        out.append(p)
    clean = []
    for i, p in enumerate(out):
        a, c = out[i - 1], out[(i + 1) % len(out)]
        if abs((p[0] - a[0]) * (c[1] - p[1]) - (p[1] - a[1]) * (c[0] - p[0])) < 1e-9:
            continue
        clean.append(p)
    return clean


def build():
    ref, got, holes, vent = solve()
    t = M.PLATE
    num = iter(range(1, 9999))
    parts = []

    def part(name, outline, cuts, x, kind, note='', rnd=None):
        p = Part(name, next(num), outline, note)
        p.cuts = list(cuts)
        p.x, p.kind, p.round = x, kind, rnd
        parts.append(p)
        return p
    # the plate frame is the reference's: x down from the seat's top
    x0 = (N_TENON + N_COLLAR) * t              # the strips' top, in the socket plate
    Ls = REF_SOLE - x0
    strip_holes = [(REF_HOLES[h][2], z - x0, d) for h, z, d, w in holes.values()]
    strip_holes += [(st, REF_VENT[0] - x0, vent) for st in VENT_STRIPS]
    own = owners(Ls, strip_holes)
    for name, (lo, hi) in SC.STRIPS.items():
        runs = [[(a, c, w == name) for a, c, w, g in own[e]] for e in (lo, hi)]
        o = strip_outline(Ls, runs[0], runs[1], x0, t)
        cuts = [('circle', x, 0.0, d / 2) for st, x, d in strip_holes if st == name]
        p = part(f'{name} strip', o, cuts, x0, 'strip', f'the {name} strip')
        p.strip = name
    # the foot: a pinwheel of flat strips each layer, the other hand each
    # time, cut to the reference's sole; each layer's inner square is the
    # strips' outside at its own height
    R = FOOT_D / 2
    for k in range(N_FOOT):
        xm = REF_SOLE - (k + 0.5) * t
        a = side(xm) / 2 + t
        e = math.sqrt(R * R - a * a)
        t0, t1 = math.atan2(-a, -e), math.atan2(-e, a)
        arc = [(R * math.cos(t0 + (t1 - t0) * i / 60), R * math.sin(t0 + (t1 - t0) * i / 60))
               for i in range(61)]
        sg = 1 if k % 2 == 0 else -1
        for i in range(4):
            pts = [(a * sg, -a), (-e * sg, -a)] + [(x * sg, y) for x, y in arc[1:-1]] + [(a * sg, -e)]
            c, sn = [(1, 0), (0, 1), (-1, 0), (0, -1)][i]
            pts = [(c * x - sn * y, sn * x + c * y) for x, y in pts]
            if sg < 0:
                pts = pts[::-1]
            p = part('foot', pts, [], REF_SOLE - (k + 1) * t, 'foot', f'foot layer {k + 1}')
            p.a = a
    # the jig: a cup the foot is glued up in
    jo = P.ngon(4, JIG / 2)
    pin_d = P.PIN - P.PIN_INTERF
    jr = (R + JIG_CLEAR / 2 + pin_d / 2 + P.MIN_WEB
          + (JIG / 2 - pin_d / 2 - P.MIN_WEB) * math.sqrt(2)) / 2
    jig_pins = [('circle', x, y, pin_d / 2) for x, y in (P.polar(45, jr), P.polar(225, jr))]
    part('jig base', jo, list(jig_pins), 0.0, 'jig', 'the foot jig\'s base')
    for k in range(N_FOOT):
        part('jig ring', jo, [('circle', 0.0, 0.0, R + JIG_CLEAR / 2)] + jig_pins, 0.0, 'jig',
             f'the foot jig, ring {k + 1}')
    # the tenon and the collar, round, pinned together midway between the
    # seat's widest and the tenon's edge
    tenon = circle(0, 0, REF_TENON_D / 2, 72)
    r_pin = (REF_SEAT[0] / 2 + REF_TENON_D / 2) / 2
    pins = [P.polar(45, r_pin), P.polar(225, r_pin)]
    pin_cut = [('circle', x, y, pin_d / 2) for x, y in pins]
    for k in range(N_TENON):
        d = ref_d((k + 0.5) * t)
        part('tenon', tenon, [('circle', 0.0, 0.0, d / 2)] + pin_cut, k * t, 'seat',
             f'tenon plate {k + 1}, the seat {d:.2f}mm', rnd=REF_TENON_D / 2)
    for k in range(N_COLLAR):
        x = (N_TENON + k) * t
        D, d = bead_d(x + t / 2), ref_d(x + t / 2)
        part('collar', circle(0, 0, D / 2, 72), [('circle', 0.0, 0.0, d / 2)] + pin_cut, x,
             'collar', f'collar plate {k + 1}, {D:.1f}mm', rnd=D / 2)
    D = bead_d(x0 + t / 2)
    tube = P.ngon(4, side(x0) / 2 + t)
    part('collar socket', circle(0, 0, D / 2, 72), [('poly', tube)], x0, 'sock',
         f'the collar\'s socket plate, {D:.1f}mm', rnd=D / 2)
    info = dict(ref=ref, got=got, holes=holes, vent=vent, t=t, Ls=Ls, x0=x0, own=own,
                strip_holes=strip_holes, pins=pins, R=R, tube=tube)
    return parts, info


def checks(parts, info, sheets):
    out = []

    def note(ok, what, detail):
        out.append((ok, what, detail))
    ref, got, t, Ls, x0 = info['ref'], info['got'], info['t'], info['Ls'], info['x0']
    off = {nt: 1200 * math.log2(got[nt] / ref[nt]) for nt in M.NOTES}
    worst = max(abs(c) for c in off.values())
    note(worst <= 1.0, 'every note sounds where the reference\'s does, in the model',
         f'within {worst:.2f} cents; the reference\'s low A {ref["low A"]:.0f}Hz in the model')
    moved = max(abs(REF_HOLES[h][0] - info['holes'][h][1]) for h in M.HOLES)
    vents = [x + x0 for st, x, d in info['strip_holes'] if st in VENT_STRIPS
             and abs(x + x0 - REF_VENT[0]) < 1e-9]
    note(moved < 1e-9 and len(vents) == 2, 'every hole is where the reference\'s is',
         ', '.join(f'{h} {REF_HOLES[h][0]:.1f}' for h in M.HOLES)
         + f', the side holes {REF_VENT[0]:.1f}mm down from the seat\'s top')
    fits = [(st, x, d, side(x + x0)) for st, x, d in info['strip_holes']]
    note(all(M.D_MIN <= d <= s - 0.35 + 1e-9 for st, x, d, s in fits),
         'every hole cuts round, and fits the bore\'s face',
         ', '.join(f'{h} {info["holes"][h][2]:.2f}' for h in M.HOLES)
         + f', side holes {info["vent"]:.2f}mm; the tightest '
         f'{min(s - d for st, x, d, s in fits):.2f}mm under its face')
    # ply round each hole, and every corner one strip's
    ok = True
    for st, x, d in info['strip_holes']:
        for c in SC.STRIPS[st]:
            for a, e, w, g in info['own'][c]:
                if a < x + d / 2 + P.MIN_WEB and e > x - d / 2 - P.MIN_WEB and w != st:
                    ok = False
    note(ok, 'every hole keeps ply all round it',
         f'its strip fingered both sides, {P.MIN_WEB:g}mm past it along')
    own = info['own']
    cover = all(abs(own[c][0][0]) < 1e-9 and abs(own[c][-1][1] - Ls) < 1e-9
                and all(abs(p[1] - q[0]) < 1e-9 for p, q in zip(own[c], own[c][1:]))
                for c in SC.CORNERS)
    runs = [e - a for c in SC.CORNERS for a, e, w, g in own[c] if not g]
    note(cover and (not runs or min(runs) >= SC.MIN_FINGER - 1e-9), 'every corner is one strip\'s, all the way',
         f'finger joints; the shortest finger {min(runs):.1f}mm')
    # the strips taper with the cone: their face the square of the
    # reference's area at both ends, every edge straight between
    strips = [p for p in parts if p.kind == 'strip']
    top_w = min(max(v[1] for v in p.outline if v[0] < 1e-9) for p in strips)
    bot_w = min(max(v[1] for v in p.outline if v[0] > Ls - 1e-9) for p in strips)
    lean = math.degrees(math.atan(SQ * REF_CONE[1] / 2))
    note(abs(top_w - side(x0) / 2) < 1e-9 and abs(bot_w - side(REF_SOLE) / 2) < 1e-9,
         'the strips make the reference\'s cone, square',
         f'{side(x0):.2f}mm square at the collar to {side(REF_SOLE):.2f}mm at the sole, the '
         f'reference\'s {ref_d(x0):.2f} to {ref_d(REF_SOLE):.2f}mm round; each face leans '
         f'{lean:.2f} degrees')
    socket = [p for p in parts if p.kind == 'sock']
    note(len(socket) == 1 and abs(socket[0].x - x0) < 1e-9 and abs(x0 + Ls - REF_SOLE) < 1e-9,
         'the strips run from the sole into the collar',
         f'{Ls:.1f}mm, their top {t:g}mm into its socket plate; the bore {REF_SOLE:g}mm from the '
         f'seat\'s top to the sole, the reference\'s')
    # the collar is the reference's bead, and its socket plate holds the
    # strips' top with ply round them
    sock = socket[0]
    sweb = sock.round - max(math.hypot(*v) for v in info['tube'])
    collar = [p for p in parts if p.kind in ('collar', 'sock')]
    note(sweb >= P.MIN_WEB and all(abs(2 * p.round - bead_d(p.x + t / 2)) < 1e-9 for p in collar),
         'the collar is the reference\'s bead, plate by plate',
         ', '.join(f'{2 * p.round:.1f}' for p in sorted(collar, key=lambda p: p.x))
         + f'mm round; {sweb:.1f}mm of ply past the strips\' corners in the socket plate')
    # the foot: each layer fills the ring round the strips once, and stops
    # short of the side holes
    R = info['R']
    feet = [p for p in parts if p.kind == 'foot']
    bad = 0
    for k in range(N_FOOT):
        layer = [p for p in feet if abs(p.x - (REF_SOLE - (k + 1) * t)) < 1e-9]
        a = layer[0].a
        for i in range(50):
            for j in range(50):
                q = (-R + (i + 0.5) * 2 * R / 50, -R + (j + 0.5) * 2 * R / 50)
                r = math.hypot(*q)
                if abs(r - R) < 0.05:
                    continue
                want = 0 if max(abs(q[0]), abs(q[1])) < a or r > R else 1
                bad += sum(P.inside(q, p.outline) for p in layer) != want
    corner = max(math.hypot(p.a, p.a) for p in feet)
    lowest = max(x + d / 2 for st, x, d in info['strip_holes'])
    clear = (Ls - N_FOOT * t) - lowest
    note(bad == 0 and len(feet) == 4 * N_FOOT and clear >= P.MIN_WEB and R - corner >= 0.5,
         'the foot is the reference\'s sole, and clear of the side holes',
         f'{2 * R:g}mm round and {N_FOOT * t:g}mm tall, the reference\'s {REF_FOOT_D:g}mm and '
         f'{REF_FOOT_H:g}mm; {R - corner:.2f}mm of it past the strips\' corners; {clear:.1f}mm '
         f'under the side holes')
    jig = [p for p in parts if p.kind == 'jig']
    rings = [c[3] for p in jig for c in p.cuts if c[0] == 'circle' and c[3] > 5]
    note(len(jig) == N_FOOT + 1 and len(rings) == N_FOOT
         and all(abs(r - (R + JIG_CLEAR / 2)) < 1e-9 for r in rings),
         'the foot jig holds every layer on the foot\'s circle',
         f'a base and {N_FOOT} rings, {N_FOOT * t:g}mm deep, the circle {2 * R + JIG_CLEAR:.1f}mm '
         f'for the foot\'s {2 * R:g}mm')
    web = math.inf
    for p in parts:
        if p.kind not in ('seat', 'collar', 'sock', 'jig'):
            continue
        edges = [('circle', 0.0, 0.0, p.round) if p.round else ('poly', p.outline)] + p.cuts
        for c in p.cuts:
            if c[0] != 'circle' or abs(c[3] - (P.PIN - P.PIN_INTERF) / 2) > 1e-9:
                continue
            for e in edges:
                if e is c:
                    continue
                d = (P.poly_dist((c[1], c[2]), e[1]) if e[0] == 'poly'
                     else abs(math.hypot(c[1] - e[1], c[2] - e[2]) - e[3]))
                web = min(web, d - c[3])
    note(web >= P.MIN_WEB, 'every pin keeps ply between it and every cut',
         f'{web:.2f}mm at the least, against {P.MIN_WEB:g}')
    seat = sorted((p for p in parts if p.kind == 'seat'), key=lambda p: p.x)
    note(all(abs(2 * p.cuts[0][3] - ref_d(p.x + t / 2)) < 1e-9 for p in seat),
         'the seat is the reference\'s taper, and the tenon its size',
         f'{2 * seat[0].cuts[0][3]:.2f} to {2 * seat[-1].cuts[0][3]:.2f}mm over '
         f'{N_TENON * t:g}mm (the reference\'s {ref_d(0):.2f} to {ref_d(REF_SEAT_DEEP):.2f}mm over '
         f'{REF_SEAT_DEEP:g}); the tenon {REF_TENON_D:g}mm round, for hemp')
    note(all(sh['w'] <= P.BED[0] and sh['h'] <= P.BED[1] for sh in sheets), 'every sheet fits the bed',
         f'{len(sheets)} sheet(s), largest {max(sh["w"] for sh in sheets):.0f} x '
         f'{max(sh["h"] for sh in sheets):.0f}mm')
    least = math.inf
    for sh in sheets:
        its = sh['parts']
        for i, (a, ax, ay) in enumerate(its):
            for c, cx, cy in its[i + 1:]:
                if (cx > ax + a.w + SF.GAP + KERF or ax > cx + c.w + SF.GAP + KERF
                        or cy > ay + a.h + SF.GAP + KERF or ay > cy + c.h + SF.GAP + KERF):
                    continue
                least = min(least, P.apart(a, ax, ay, c, cx, cy))
    note(least >= SF.GAP + KERF - 1e-3, 'no two parts on a sheet come closer than the gap',
         f'{least - KERF:.2f}mm of ply between cuts at the least, against {SF.GAP:g}')
    placed_n = sum(len(sh['parts']) for sh in sheets)
    note(placed_n == len(parts), 'every part is on a sheet', f'{placed_n} of {len(parts)}')
    return out


def label():
    return 'ghb-chanter'


def svg_sheet(k, nsh, sh):
    txt = P.svg_sheet(k, nsh, sh, 4)
    what = 'Bagpipe chanter, a tapering square bore of four finger-jointed strips'
    return (txt.replace('Practice chanter, 4-sided bore', what)
            .replace('cuts the bores, seats and pin holes first; then black',
                     'cuts the bores, seats, finger and pin holes first; then black'))


def placed(p, info):
    """Where a part sits: the plate frame's x across, y down, z through, to
    the chanter's length, front and side; a strip's x along it, u across
    from its centre line, w through, its face leaning with the cone."""
    t, X = info['t'], info['x0']
    if p.kind != 'strip':
        return [[0, 0, 1, p.x], [0, 1, 0, 0], [-1, 0, 0, 0], [0, 0, 0, 1]]
    h = SQ * REF_CONE[1] / 2                       # the face's lean, per mm along
    o = side(X) / 2                                # the face at the strips' top
    return {
        # front: w out to in is y -(face + t) to -face; u to the left is -z
        'front': [[1, 0, 0, X], [-h, 0, 1, -o - t], [0, -1, 0, 0], [0, 0, 0, 1]],
        'back': [[1, 0, 0, X], [h, 0, -1, o + t], [0, 1, 0, 0], [0, 0, 0, 1]],
        # left: w in to out is z face to face + t; u to the back is y
        'left': [[1, 0, 0, X], [0, 1, 0, 0], [h, 0, 1, o], [0, 0, 0, 1]],
        'right': [[1, 0, 0, X], [0, -1, 0, 0], [-h, 0, -1, -o], [0, 0, 0, 1]],
    }[p.strip]


def design3d():
    """The chanter for the 3D page."""
    parts, info = build()
    import turn3d as t3
    mid = REF_SOLE / 2
    out = []
    for p in parts:
        if p.kind == 'jig':
            continue                                  # a tool, not the chanter
        holes = [c[1] if c[0] == 'poly' else circle(c[1], c[2], c[3], 20) for c in p.cuts]
        if p.kind == 'strip':
            d = {'front': (0, -1, 0), 'back': (0, 1, 0), 'left': (0, 0, 1),
                 'right': (0, 0, -1)}[p.strip]
            ex = (0, d[1] * 0.15, d[2] * 0.15)
        else:
            ex = ((p.x - mid) / 60, 0, 0)
        wood = 'dark' if p.kind == 'strip' else 'ply'
        out.append(t3.placed({'o': p.outline, 'h': holes, 'e': []}, placed(p, info), ex, wood))
    return {'parts': out, 'ex': 60, 'view': [-0.12, 0.32, 3.0]}


def page(path_out):
    sys.path.insert(0, os.path.join(HERE, '..', 'lasermade-tools'))
    import turn3d as t3
    t3.write(path_out, 'Bagpipe chanter in 3D', {'chanter': design3d()},
             'Drag to turn, scroll or pinch to zoom; draw Explode to pull the strips outward and '
             'the plates apart along the chanter. Every part is the cut file’s own outline. '
             'The four strips, darker, taper with the cone: wider toward the sole. The finger '
             'holes face you, high A’s on the back, and a side hole on each side near the '
             'sole. Above the strips, the collar and the tenon with the reed’s seat; at the '
             'sole, two layers of flat strips make the foot.',
             'ghb_chanter.py')


def main():
    global KERF
    out, write, page_to = None, True, None
    for x in sys.argv[1:]:
        if x == '--no-write':
            write = False
        elif x.startswith('--out='):
            out = x.split('=', 1)[1]
        elif x.startswith('--plate='):
            M.PLATE = float(x.split('=', 1)[1])
        elif x.startswith('--kerf='):
            KERF = P.KERF = SF.set_kerf(x)
        elif x.startswith('--page-to='):
            page_to = x.split('=', 1)[1]
        else:
            raise SystemExit(f'error: {x} is not a flag this generator reads. It takes '
                             f'--out=, --plate=, --kerf=, --page-to= and --no-write.')
    if page_to:
        page(page_to)
        print(f'  wrote {os.path.basename(page_to)}')
        return 0
    if write and not out:
        raise SystemExit('error: --out=DIR, or --no-write to run the checks only.')
    parts, info = build()
    sheets = P.pack(parts)
    res = checks(parts, info, sheets)
    t = info['t']
    print(f'bagpipe chanter, the Dis-Enchanter\'s holes on a tapering square bore of four '
          f'finger-jointed strips: strips {info["Ls"]:.1f}mm long, {side(info["x0"]):.2f} to '
          f'{side(REF_SOLE):.2f}mm square inside; {REF_SOLE:g}mm from the seat\'s top to the sole')
    for h in M.HOLES:
        z, d_ref, st = REF_HOLES[h]
        d = info['holes'][h][2]
        print(f'  {h:3} {st:5} strip, {z:6.1f}mm down from the seat\'s top, {d:.2f}mm round '
              f'(the reference {d_ref:.2f}), the face {side(z):.2f}mm')
    print(f'  side holes, left and right strips, {REF_VENT[0]:.1f}mm down, {info["vent"]:.2f}mm '
          f'round (the reference {REF_VENT[1]:.2f}), the face {side(REF_VENT[0]):.2f}mm')
    print(f'  top down: {N_TENON} tenon, {N_COLLAR} collar, 1 collar socket over the strips; at '
          f'the sole the round foot, {N_FOOT} layers of 4 strips')
    bad = 0
    for ok, what, detail in res:
        print(f'  {"pass" if ok else "FAIL"}  {what:52} {detail}')
        bad += not ok
    if bad:
        print(f'\n  {bad} check(s) failed. Nothing written.')
        return 1
    if write:
        os.makedirs(out, exist_ok=True)
        for k, sh in enumerate(sheets, 1):
            f = os.path.join(out, f'{label()}-sheet{k}-cut-files.svg')
            open(f, 'w').write(svg_sheet(k, len(sheets), sh))
            print(f'  wrote {os.path.basename(f)}  {sh["w"]:.0f} x {sh["h"]:.0f}mm, '
                  f'{len(sh["parts"])} parts')
    return 0


if __name__ == '__main__':
    sys.exit(main())
