#!/usr/bin/env python3
"""Cut files for a Great Highland bagpipe chanter whose conical bore is a
tapering polygon of long strips of ply, four, six or eight, glued, from the
sole up into a collar of stacked plates, with a stacked tenon and reed seat
above it.

    python3 ghb_chanter.py --no-write                  # the checks, four sides
    python3 ghb_chanter.py --sides=6 --no-write        # six sides, or 8
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

The bore here is the reference's cone made a regular polygon, the same area
at every height, so the strips taper: each strip's face is the polygon's
side, wider toward the sole. Its front and back are faces, for the finger
holes and the thumb's. The holes stay where the reference's are. They go through a
3mm strip instead of the reference's 2mm of wall, so each hole's size is
solved for its note to sound where the reference's does, in chanter_model.py's
terms, and the side holes' size for low G. The model treats the bore as an
exact cone, closed at its apex for the reed; it is used only for the
difference between the two chanters, not for the pitch. (The cone closed at
its apex stands in for the reed: a conical bore sounds as the whole cone.)

A hole wider than its face, less WEB_FACE, is a stadium instead: as wide as
that allows and long enough for the round hole's area. The model takes it as
that round hole.

From the sole up:

  * the STRIPS, tapering, from the sole into the collar, finger-jointed and
    glued; round each hole its strip owns both corners. Where the faces meet
    at more than a right angle (six and eight sides), a finger reaches only
    t / tan(half that angle) past the face's edge and the strip beside it
    stands back from the edge between fingers, as boxes.py's angled finger
    joints do: the strips meet along the bore's corner, with a small groove
    inside and a notch outside at every finger. There is no butt-jointed
    version: the holes are too wide for their face to keep ply round them
    unless their strip owns the corners;
  * the FOOT, N_FOOT layers of strips lying flat round the strips' bottom,
    one to a side, each layer turned the other way to the one below, each
    strip's outer edge an arc of the reference's sole, FOOT_D round;
  * the COLLAR: a plate with a hole the strips' top stands in, glued
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
SIDES = 4                          # the bore's polygon: 4, 6 or 8
WEB_FACE = 0.35                    # a round hole this much narrower than its face, at least
MIN_STRIP = 1.0                    # no strip narrower than this anywhere
N_TENON = 7                        # 21mm of seat, the reference's 22
N_COLLAR = 4                       # round plates over the socket plate
FOOT_D = REF_FOOT_D
N_FOOT = 2                         # 6mm, the reference's sole 7.25mm
JIG = FOOT_D + 10.0
JIG_CLEAR = 0.3
NAMES = {4: ['front', 'right', 'back', 'left'],
         6: ['front', 'front right', 'back right', 'back', 'back left', 'front left'],
         8: ['front', 'front right', 'right', 'back right', 'back', 'back left', 'left',
             'front left']}
VENTS = {4: (1, 3), 6: (2, 4), 8: (2, 6)}   # the side holes' strips


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


def side(z, n=None):
    """The polygon's side, z down: the reference's cone's area."""
    n = n or SIDES
    k = math.sqrt(math.pi * math.tan(math.pi / n) / n)
    return k * (REF_CONE[0] + REF_CONE[1] * z)


def inr(z, n=None):
    """The polygon's inradius, z down."""
    n = n or SIDES
    return side(z, n) / (2 * math.tan(math.pi / n))


def corner(t, n=None):
    """At a corner of the polygon: how far a finger reaches past the face's
    edge, and how far the strip beside it stands back from the edge."""
    n = n or SIDES
    th = math.pi - 2 * math.pi / n                 # the faces' inside angle
    reach = t / math.tan(th / 2)
    return reach, max(0.0, -reach * math.cos(th))


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


_SOLVED = {}


def solve():
    """Each hole's size, and the vents', that sound the reference's notes,
    the holes where the reference's are, through a strip wall: the round
    hole's diameter, whatever its shape. The same for every polygon, which
    the model knows only by its area."""
    if M.PLATE in _SOLVED:
        return _SOLVED[M.PLATE]
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
        vmax = REF_CONE[0] + REF_CONE[1] * REF_VENT[0]
        if fv(vmax) < ref['low G']:
            raise SystemExit(f'error: low G is flat even with {vmax:.2f}mm side holes.')
        vent[0] = bisect(fv, ref['low G'], M.D_MIN, vmax, rising=True)
        for h in M.HOLES:
            nt = note_of[h]
            d_max = REF_CONE[0] + REF_CONE[1] * holes[h][1]

            def f(d):
                holes[h][2] = d
                return sounding(every(), nt, ref[nt])
            if f(d_max) < ref[nt]:
                raise SystemExit(f'error: {h} is flat even {d_max:.2f}mm across.')
            holes[h][2] = bisect(f, ref[nt], M.D_MIN, d_max, rising=True)
    got = {nt: sounding(every(), nt, ref[nt]) for nt in M.NOTES}
    _SOLVED[M.PLATE] = ref, got, holes, vent[0]
    return _SOLVED[M.PLATE]


# --- the strips --------------------------------------------------------------
# Strips are numbered round the bore from the front, toward the player's
# right; corner i is between strip i and strip i + 1. In a strip, u runs
# across its face from its centre line, toward strip i + 1.

def holes_of(info_holes, vent):
    """Each hole on the strips: (strip, x down from the seat's top, round
    diameter)."""
    n = SIDES
    out = [((n // 2) if h == 'hA' else 0, z, d) for h, z, d, w in info_holes.values()]
    out += [(i, REF_VENT[0], vent) for i in VENTS[n]]
    return out


def shape(z, d):
    """A hole's (width, length) on its face: round if it fits, else the
    stadium of its area as wide as the face allows at the hole's top."""
    w, ln = d, d
    for _ in range(60):
        face = side(z - ln / 2) - WEB_FACE
        if d <= face:
            return d, d
        w = face
        ln = w + (math.pi * d * d / 4 - math.pi * w * w / 4) / w
    return w, ln


def stadium(x, w, ln, steps=16):
    """A stadium centred (x, 0) in a strip's frame, its length along x."""
    r, h = w / 2, (ln - w) / 2
    pts = []
    for i in range(steps + 1):
        a = math.pi * i / steps
        pts.append((x + h + r * math.sin(a), r * math.cos(a)))
    for i in range(steps + 1):
        a = math.pi * i / steps
        pts.append((x - h - r * math.sin(a), -r * math.cos(a)))
    return pts


def owners(Ls, x0, holes):
    """Each corner's owner along the strips, as [(x0, x1, strip, guarded)]
    from the strips' top: each strip owns one of its corners, then the
    other, finger by finger; round each hole its strip owns both, MIN_WEB
    past the hole, and the strips beside it keep their other corners."""
    n = SIDES
    nf = max(1, int(round(Ls / SC.FINGER)))
    edges = [Ls * i / nf for i in range(nf + 1)]
    guard = [(z - x0 - ln / 2 - P.MIN_WEB, z - x0 + ln / 2 + P.MIN_WEB, st)
             for st, z, d in holes for w, ln in [shape(z, d)]]
    cuts = sorted(set(edges + [min(max(g, 0.0), Ls) for a, b, s in guard for g in (a, b)]))
    out = {c: [] for c in range(n)}
    for a, b in zip(cuts, cuts[1:]):
        mid = (a + b) / 2
        k = min(int(mid / Ls * nf), nf - 1)
        who = {c: (c, (c + 1) % n)[k % 2] for c in range(n)}
        gs = [s for g0, g1, s in guard if g0 <= mid <= g1]
        mine = {}
        for s in gs:
            mine[(s - 1) % n] = mine[s] = s
        for s in gs:
            mine.setdefault((s + 1) % n, (s + 1) % n)
            mine.setdefault((s - 2) % n, (s - 1) % n)
        for c in range(n):
            out[c].append([a, b, mine.get(c, who[c]), c in mine])
    return {c: SC.tidy(iv) for c, iv in out.items()}


def reaches(own, x0, t):
    """Each corner's runs with how far its owner's finger reaches, as
    [(x0, x1, owner, reach)]. Where a hole's strip owns both its corners,
    some strip owns neither and stands back at both; where that would leave
    it narrower than MIN_STRIP, the fingers either side of it reach less, and
    it stands back less: a shorter finger meets the strip beside it in the
    same way, its end on that strip's end, with a deeper notch outside."""
    n = SIDES
    full, back = corner(t)
    k = back / full
    cuts = sorted(set(v for c in range(n) for a, b, w, g in own[c] for v in (a, b)))
    out = {c: [] for c in range(n)}
    for a, b in zip(cuts, cuts[1:]):
        mid = (a + b) / 2
        who = {c: next(w for p, q, w, g in own[c] if p <= mid <= q) for c in range(n)}
        r = {c: full for c in range(n)}
        for i in range(n):
            if k and who[(i - 1) % n] != i and who[i] != i:
                cap = (side(x0 + a) - MIN_STRIP - 0.05) / (2 * k)
                for c in ((i - 1) % n, i):
                    r[c] = max(0.0, min(r[c], cap))
        for c in range(n):
            iv = out[c]
            if iv and iv[-1][2] == who[c] and abs(iv[-1][3] - r[c]) < 1e-12:
                iv[-1][1] = b
            else:
                iv.append([a, b, who[c], r[c]])
    return out


def strip_outline(Ls, lo, hi, x0, t):
    """A strip from its top, x = 0, to the sole, x = Ls; u across it from its
    centre line, the bore's face out to side/2 each way, a finger reaching
    past it where the strip owns that corner, standing back from it where
    it does not. The face widens with the cone, so every edge slopes. lo, hi:
    its edges' runs, [(x0, x1, owns, reach)]."""
    full, back = corner(t)
    k = back / full

    def edge(runs, sign):
        pts = []
        for a, b, owns, r in runs:
            for x in (a, b):
                pts.append((x, sign * (side(x0 + x) / 2 + (r if owns else -k * r))))
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


def normal(i):
    """Strip i's outward normal in the plate frame (x to the right, y to the
    front is negative)."""
    a = -2 * math.pi * i / SIDES
    return (-math.sin(a), -math.cos(a))


def outer(z, t):
    """The strips' outside at z down, in the plate frame: vertex i between
    strip i and strip i + 1."""
    n = SIDES
    R = (inr(z) + t) / math.cos(math.pi / n)
    pts = []
    for i in range(n):
        a, b = normal(i), normal((i + 1) % n)
        m = math.hypot(a[0] + b[0], a[1] + b[1])
        pts.append((R * (a[0] + b[0]) / m, R * (a[1] + b[1]) / m))
    return pts


def build():
    ref, got, holes, vent = solve()
    n, t = SIDES, M.PLATE
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
    on = holes_of(holes, vent)
    own = owners(Ls, x0, on)
    reach = reaches(own, x0, t)
    for i in range(n):
        runs = [[(a, c, w == i, r) for a, c, w, r in reach[e]] for e in ((i - 1) % n, i)]
        o = strip_outline(Ls, runs[0], runs[1], x0, t)
        cuts = []
        for st, z, d in on:
            if st != i:
                continue
            w, ln = shape(z, d)
            cuts.append(('circle', z - x0, 0.0, d / 2) if w == d
                        else ('poly', stadium(z - x0, w, ln)))
        p = part(f'{NAMES[n][i]} strip', o, cuts, x0, 'strip', f'the {NAMES[n][i]} strip')
        p.strip = i
    # the foot: a pinwheel of flat strips round the strips' outside each
    # layer, the other hand each time, cut to the reference's sole: each
    # piece runs along one side from the circle, past the corner before it,
    # to the corner after it
    R = FOOT_D / 2

    def to_circle(p, d):
        b = p[0] * d[0] + p[1] * d[1]
        s = -b + math.sqrt(b * b - (p[0] ** 2 + p[1] ** 2 - R * R))
        return (p[0] + s * d[0], p[1] + s * d[1])
    for k in range(N_FOOT):
        W = outer(REF_SOLE - (k + 0.5) * t, t)
        for j in range(n):
            A, B, C = W[(j - 1) % n], W[j], W[(j + 1) % n]
            dab = (A[0] - B[0], A[1] - B[1])
            dab = (dab[0] / math.hypot(*dab), dab[1] / math.hypot(*dab))
            dbc = (B[0] - C[0], B[1] - C[1])
            dbc = (dbc[0] / math.hypot(*dbc), dbc[1] / math.hypot(*dbc))
            p0, q = to_circle(A, dab), to_circle(B, dbc)
            a0, a1 = math.atan2(q[1], q[0]), math.atan2(p0[1], p0[0])
            while a1 < a0:
                a1 += 2 * math.pi
            if a1 - a0 > math.pi:
                a1 -= 2 * math.pi
            arc = [(R * math.cos(a0 + (a1 - a0) * i / 40), R * math.sin(a0 + (a1 - a0) * i / 40))
                   for i in range(1, 40)]
            pts = [p0, A, B, q] + arc
            if k % 2:
                pts = [(-x, y) for x, y in pts][::-1]
            p = part('foot', pts, [], REF_SOLE - (k + 1) * t, 'foot', f'foot layer {k + 1}')
            p.W = W
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
    tube = outer(x0, t)
    part('collar socket', circle(0, 0, D / 2, 72), [('poly', tube)], x0, 'sock',
         f'the collar\'s socket plate, {D:.1f}mm', rnd=D / 2)
    info = dict(ref=ref, got=got, holes=holes, vent=vent, t=t, Ls=Ls, x0=x0, own=own, reach=reach,
                on=on, pins=pins, R=R, tube=tube)
    return parts, info


def widths(p):
    """A strip's narrowest, across, and where."""
    xs = sorted(set(round(v[0], 9) for v in p.outline))
    best = (math.inf, 0.0)
    for a, b in zip(xs, xs[1:]):
        x = (a + b) / 2
        us = []
        o = p.outline
        for j in range(len(o)):
            (x1, u1), (x2, u2) = o[j], o[(j + 1) % len(o)]
            if (x1 - x) * (x2 - x) < 0:
                us.append(u1 + (u2 - u1) * (x - x1) / (x2 - x1))
        if len(us) >= 2:
            best = min(best, (max(us) - min(us), x))
    return best


def checks(parts, info, sheets):
    out = []

    def note(ok, what, detail):
        out.append((ok, what, detail))
    n = SIDES
    ref, got, t, Ls, x0 = info['ref'], info['got'], info['t'], info['Ls'], info['x0']
    reach, back = corner(t)
    off = {nt: 1200 * math.log2(got[nt] / ref[nt]) for nt in M.NOTES}
    worst = max(abs(c) for c in off.values())
    note(worst <= 1.0, 'every note sounds where the reference\'s does, in the model',
         f'within {worst:.2f} cents; the reference\'s low A {ref["low A"]:.0f}Hz in the model')
    moved = max(abs(REF_HOLES[h][0] - info['holes'][h][1]) for h in M.HOLES)
    vents = [z for st, z, d in info['on'] if st in VENTS[n] and abs(z - REF_VENT[0]) < 1e-9]
    note(moved < 1e-9 and len(vents) == 2, 'every hole is where the reference\'s is',
         ', '.join(f'{h} {REF_HOLES[h][0]:.1f}' for h in M.HOLES)
         + f', the side holes {REF_VENT[0]:.1f}mm down from the seat\'s top')
    # each hole: round, or a stadium of its area, inside its face
    fit, stad = True, []
    for st, z, d in info['on']:
        w, ln = shape(z, d)
        if w > side(z - ln / 2) - WEB_FACE + 1e-9 or w < M.D_MIN:
            fit = False
        if abs(math.pi * d * d / 4 - (math.pi * w * w / 4 + w * (ln - w))) > 1e-6:
            fit = False
        if w != d:
            stad.append((st, z, w, ln))
    names = {z: h for h, (z, d, s) in REF_HOLES.items()}
    note(fit, 'every hole cuts inside its face, at its area',
         'all round' if not stad else
         'round but for ' + ', '.join(f'{names.get(z, "a side hole")} {w:.2f} x {ln:.2f}mm'
                                      for st, z, w, ln in stad
                                      if st != VENTS[n][1]) + ', stadiums')
    # ply round each hole, and every corner one strip's
    ok = True
    for st, z, d in info['on']:
        w, ln = shape(z, d)
        x = z - x0
        for c in ((st - 1) % n, st):
            for a, e, who, g in info['own'][c]:
                if a < x + ln / 2 + P.MIN_WEB and e > x - ln / 2 - P.MIN_WEB and who != st:
                    ok = False
    note(ok, 'every hole keeps ply all round it',
         f'its strip fingered both sides, {P.MIN_WEB:g}mm past it along')
    own = info['own']
    cover = all(abs(own[c][0][0]) < 1e-9 and abs(own[c][-1][1] - Ls) < 1e-9
                and all(abs(p[1] - q[0]) < 1e-9 for p, q in zip(own[c], own[c][1:]))
                for c in range(n))
    runs = [e - a for c in range(n) for a, e, w, g in own[c] if not g]
    note(cover and (not runs or min(runs) >= SC.MIN_FINGER - 1e-9),
         'every corner is one strip\'s, all the way',
         f'finger joints; the shortest finger {min(runs):.1f}mm; a finger reaches {reach:.2f}mm '
         f'past its face' + (f', the strip beside it standing back {back:.2f}mm' if back else ''))
    strips = [p for p in parts if p.kind == 'strip']
    narrow = min((widths(p) + (p,) for p in strips), key=lambda v: v[0])
    note(narrow[0] >= MIN_STRIP, 'no strip is too narrow anywhere',
         f'the narrowest {narrow[0]:.2f}mm, the {NAMES[n][narrow[2].strip]} strip '
         f'{narrow[1] + x0:.0f}mm down, against {MIN_STRIP:g}')
    lean = math.degrees(math.atan(inr(1) - inr(0)))
    note(len(strips) == n, f'the strips make the reference\'s cone, {n}-sided',
         f'{n} strips; the faces {side(x0):.2f}mm at the collar to {side(REF_SOLE):.2f}mm at the '
         f'sole, the same area as the reference\'s {ref_d(x0):.2f} to {ref_d(REF_SOLE):.2f}mm '
         f'round; each face leans {lean:.2f} degrees')
    socket = [p for p in parts if p.kind == 'sock']
    note(len(socket) == 1 and abs(socket[0].x - x0) < 1e-9 and abs(x0 + Ls - REF_SOLE) < 1e-9,
         'the strips run from the sole into the collar',
         f'{Ls:.1f}mm, their top {t:g}mm into its socket plate; the bore {REF_SOLE:g}mm from the '
         f'seat\'s top to the sole, the reference\'s')
    sock = socket[0]
    sweb = sock.round - max(math.hypot(*v) for v in info['tube'])
    collar = [p for p in parts if p.kind in ('collar', 'sock')]
    note(sweb >= P.MIN_WEB and all(abs(2 * p.round - bead_d(p.x + t / 2)) < 1e-9 for p in collar),
         'the collar is the reference\'s bead, plate by plate',
         ', '.join(f'{2 * p.round:.1f}' for p in sorted(collar, key=lambda p: p.x))
         + f'mm round; {sweb:.1f}mm of ply past the strips\' corners in the socket plate')
    # the foot: each layer fills the ring round the strips once
    R = info['R']
    feet = [p for p in parts if p.kind == 'foot']
    bad = 0
    for k in range(N_FOOT):
        layer = [p for p in feet if abs(p.x - (REF_SOLE - (k + 1) * t)) < 1e-9]
        W = layer[0].W
        for i in range(60):
            for j in range(60):
                q = (-R + (i + 0.5) * 2 * R / 60, -R + (j + 0.5) * 2 * R / 60)
                r = math.hypot(*q)
                if abs(r - R) < 0.05 or min(P.poly_dist(q, W), 9) < 0.02:
                    continue
                want = 0 if P.inside(q, W) or r > R else 1
                bad += sum(P.inside(q, p.outline) for p in layer) != want
    far = max(math.hypot(*v) for p in feet for v in p.W)
    lowest = max(z + shape(z, d)[1] / 2 for st, z, d in info['on'])
    clear = (REF_SOLE - N_FOOT * t) - lowest
    note(bad == 0 and len(feet) == n * N_FOOT and clear >= P.MIN_WEB and R - far >= 0.5,
         'the foot is the reference\'s sole, and clear of the side holes',
         f'{2 * R:g}mm round and {N_FOOT * t:g}mm tall, the reference\'s {REF_FOOT_D:g}mm and '
         f'{REF_FOOT_H:g}mm; {R - far:.2f}mm of it past the strips\' corners; {clear:.1f}mm '
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
    return f'ghb-chanter-{SIDES}-sided'


def svg_sheet(k, nsh, sh):
    txt = P.svg_sheet(k, nsh, sh, SIDES)
    what = f'Bagpipe chanter, a tapering {SIDES}-sided bore of finger-jointed strips'
    return (txt.replace(f'Practice chanter, {SIDES}-sided bore', what)
            .replace('cuts the bores, seats and pin holes first; then black',
                     'cuts the bores, seats, finger and pin holes first; then black'))


def placed(p, info):
    """Where a part sits: the plate frame's x across, y down, z through, to
    the chanter's side, front and length; a strip's x along it, u across
    from its centre line, w through from the bore out, its face leaning with
    the cone."""
    if p.kind != 'strip':
        return [[0, 0, 1, p.x], [0, 1, 0, 0], [-1, 0, 0, 0], [0, 0, 0, 1]]
    X = info['x0']
    nx, ny = normal(p.strip)
    tx, ty = -ny, nx                               # toward strip i + 1
    r0, kr = inr(X), inr(X + 1) - inr(X)
    # plate frame (px, py) = (r + w) n + u t; the chanter's y = py, z = -px
    return [[1, 0, 0, X],
            [kr * ny, ty, ny, r0 * ny],
            [-kr * nx, -tx, -nx, -r0 * nx],
            [0, 0, 0, 1]]


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
            nx, ny = normal(p.strip)
            ex = (0, ny * 0.15, -nx * 0.15)
        else:
            ex = ((p.x - mid) / 60, 0, 0)
        wood = 'dark' if p.kind == 'strip' else 'ply'
        out.append(t3.placed({'o': p.outline, 'h': holes, 'e': []}, placed(p, info), ex, wood))
    return {'parts': out, 'ex': 60, 'view': [-0.12, 0.32, 3.0]}


def page(path_out):
    global SIDES
    sys.path.insert(0, os.path.join(HERE, '..', 'lasermade-tools'))
    import turn3d as t3
    designs, keep = {}, SIDES
    for n in (4, 6, 8):
        SIDES = n
        designs[f'{n} sides'] = design3d()
    SIDES = keep
    t3.write(path_out, 'Bagpipe chanter in 3D', designs,
             'Drag to turn; right-drag, shift-drag or move two fingers to slide the view; '
             'scroll or pinch to zoom in on whatever is under the pointer; double-click a part '
             'to turn round it, and Reset view to go back. Draw Explode to pull the strips outward and '
             'the plates apart along the chanter. Every part is the cut file’s own outline. '
             'The strips, darker, taper with the cone: wider toward the sole. The finger holes '
             'face you, high A’s on the back, and a side hole on each side near the sole; '
             'where a hole is wider than its face it is a stadium. Above the strips, the collar '
             'and the tenon with the reed’s seat; at the sole, two layers of flat strips '
             'make the foot. The buttons switch between four, six and eight strips.',
             'ghb_chanter.py')


def main():
    global KERF, SIDES
    out, write, page_to = None, True, None
    for x in sys.argv[1:]:
        if x == '--no-write':
            write = False
        elif x.startswith('--out='):
            out = x.split('=', 1)[1]
        elif x.startswith('--sides='):
            SIDES = int(x.split('=', 1)[1])
            if SIDES not in NAMES:
                raise SystemExit(f'error: --sides={SIDES}: 4, 6 or 8.')
        elif x.startswith('--plate='):
            M.PLATE = float(x.split('=', 1)[1])
        elif x.startswith('--kerf='):
            KERF = P.KERF = SF.set_kerf(x)
        elif x.startswith('--page-to='):
            page_to = x.split('=', 1)[1]
        else:
            raise SystemExit(f'error: {x} is not a flag this generator reads. It takes --sides=, '
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
    n = SIDES
    print(f'bagpipe chanter, the Dis-Enchanter\'s holes on a tapering {n}-sided bore of '
          f'finger-jointed strips: strips {info["Ls"]:.1f}mm long, faces {side(info["x0"]):.2f} '
          f'to {side(REF_SOLE):.2f}mm; {REF_SOLE:g}mm from the seat\'s top to the sole')
    for h in M.HOLES:
        z, d_ref, st = REF_HOLES[h]
        d = info['holes'][h][2]
        w, ln = shape(z, d)
        how = f'{d:.2f}mm round' if w == d else f'a {w:.2f} x {ln:.2f}mm stadium, {d:.2f}mm round\'s area'
        print(f'  {h:3} {NAMES[n][n // 2 if h == "hA" else 0]:5} strip, {z:6.1f}mm down, {how} '
              f'(the reference {d_ref:.2f}), the face {side(z):.2f}mm')
    w, ln = shape(REF_VENT[0], info['vent'])
    how = (f'{info["vent"]:.2f}mm round' if w == info['vent']
           else f'{w:.2f} x {ln:.2f}mm stadiums, {info["vent"]:.2f}mm round\'s area')
    print(f'  side holes, {" and ".join(NAMES[n][i] for i in VENTS[n])} strips, '
          f'{REF_VENT[0]:.1f}mm down, {how} (the reference {REF_VENT[1]:.2f}), the face '
          f'{side(REF_VENT[0]):.2f}mm')
    print(f'  top down: {N_TENON} tenon, {N_COLLAR} collar, 1 collar socket over the strips; at '
          f'the sole the round foot, {N_FOOT} layers of {n} strips')
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
