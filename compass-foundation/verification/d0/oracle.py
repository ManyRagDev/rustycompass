"""Independent exact mathematics. No production imports or expected-output capture."""
from itertools import combinations


def lines(raw):
    text = raw.decode('utf-8', errors='strict')
    if '\0' in text:
        raise ValueError('nul')
    if text.startswith('\ufeff'):
        text = text[1:]
    text = text.replace('\r\n', '\n')
    if not text:
        return []
    parts = text.split('\n')
    return parts[:-1] if text.endswith('\n') else parts


def lcs(left, right):
    if left == right:
        return len(left)  # Identity, including E36: no quadratic allocation/work.
    row = [0] * (len(right) + 1)
    for x in left:
        diagonal = 0
        for j, y in enumerate(right, 1):
            old = row[j]
            row[j] = diagonal + 1 if x == y else max(row[j], row[j-1])
            diagonal = old
    return row[-1]


def exhaustive_lcs(left, right):
    """Second method for small examples: enumerate subsequences, no DP recurrence."""
    if len(left) > 12 or len(right) > 12:
        raise ValueError('small examples only')
    a = {tuple(left[i] for i in indices) for n in range(len(left)+1)
         for indices in combinations(range(len(left)), n)}
    b = {tuple(right[i] for i in indices) for n in range(len(right)+1)
         for indices in combinations(range(len(right)), n)}
    return max(map(len, a & b))


def ratio(c, m):
    d = m or 1
    q, r = divmod(c * 1_000_000, d)
    q += 2*r > d or (2*r == d and q % 2 == 1)
    return {'numerator': str(c), 'denominator': str(d),
            'decimal': f'{q//1_000_000}.{q%1_000_000:06d}'}


def calculation(left, right):
    n, t, common = len(left), len(right), lcs(left, right)
    a, r = t-common, n-common
    return dict(n0=str(n), nt=str(t), l=str(common), a=str(a), r=str(r),
                c=str(a+r), m=str(n+t), delta=ratio(a+r, n+t),
                zero_basis=None if n+t else 'zero_text_mass')


def cell_cost(left, right):
    return 0 if left == right else len(left)*len(right)


def aggregate(units, partial=False):
    available = [u['calculation'] for u in units if u['status'] == 'available']
    unresolved = any(u['status'] == 'unavailable' for u in units)
    blocked = partial or unresolved
    c = sum(int(x['c']) for x in available)
    m = sum(int(x['m']) for x in available)
    status = ('partial' if available else 'unavailable') if blocked else (
        'complete' if available or not units else 'unavailable')
    reasons = sorted((['foundation_partial'] if partial else []) +
                     (['unresolved_units'] if unresolved else []) +
                     (['no_text_units'] if not available and status != 'complete' else []))
    subset = None
    if available:
        subset = dict(scope='resolved_text_units_only', value=ratio(c,m),
                      line_mass=str(m), zero_basis=None if m else 'zero_text_mass',
                      resolved_text_units=str(len(available)))
    return dict(status=status, value=ratio(c,m) if status == 'complete' else None,
                line_mass=str(m) if status == 'complete' else None,
                zero_basis=(None if m else ('zero_text_mass' if available else 'empty_universe'))
                if status == 'complete' else None, reasons=reasons, known_subset=subset)
