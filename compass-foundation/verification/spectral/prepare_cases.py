#!/usr/bin/env python3
"""Author synthetic acquisition sources and freeze evidence before policy runs.

Only fixtures/<split> is indexed. controls/<split>.json is evaluator material,
never copied into a collection. Reserved cases must not tune the policy.
"""
import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def build(split, root_name, middle_name, leaf_name, value):
    source = {
        'entry.py': f'from bridge import {middle_name}\n\ndef {root_name}(amount):\n    return {middle_name}(amount)\n\ndef local_tag(amount):\n    return "local:" + str(amount)\n',
        'bridge.py': f'from policy import {leaf_name}\n\ndef {middle_name}(amount):\n    adjusted = {leaf_name}(amount)\n    return adjusted + {value}\n',
        'policy.py': f'OFFSET = {value * 2}\n\ndef {leaf_name}(amount):\n    return amount * 3 + OFFSET\n',
        'external.py': 'import os\n\ndef external_choice(amount, callback):\n    return callback(amount) + int(os.environ["REMOTE_FEE"])\n',
        'decoys.py': '\n'.join(f'def nearby_{i}(amount):\n    return amount + {i}\n' for i in range(12)) + '\n',
        'notes.txt': 'Synthetic repository. Runtime callback and environment are supplied outside this repository.\n',
    }
    # A coherent definition longer than the short budget; no expectation is
    # placed in source comments or filenames consumed by ranking.
    source['bulk.py'] = 'def bulk_transform(amount):\n' + ''.join(f'    stage_{i} = amount + {i}\n' for i in range(350)) + '    return stage_349\n'
    def span(path, start, end):
        return dict(path=path, start_line=start, end_line=end)
    def unit(name, *alternatives):
        return dict(id=name, alternatives=[list(a) for a in alternatives])
    direct = unit('entry-body', [span('entry.py', 3, 4)])
    middle = unit('bridge-body', [span('bridge.py', 3, 5)])
    leaf = unit('leaf-rule', [span('policy.py', 1, 1), span('policy.py', 3, 4)],
                [span('policy.py', 1, 1), span('policy.py', 4, 4)])
    c = [
        dict(id='local', query='local_tag', scope='local', required=[unit('local-body', [span('entry.py', 6, 7)])], answerable=True),
        dict(id='same-seed-local', query=root_name, scope='local', required=[direct], answerable=True),
        dict(id='same-seed-dependencies', query=root_name, scope='dependencies', required=[direct, middle, leaf], answerable=True),
        dict(id='cross-module', query=middle_name, scope='dependencies', required=[middle, leaf], answerable=True),
        dict(id='negative-missing', query='never_defined_symbol_817', scope='local', required=[], answerable=False, expected_gap='no_evidence'),
        dict(id='dynamic-external', query='external_choice', scope='dependencies', required=[unit('dynamic-body', [span('external.py', 1, 4)])], answerable=False, expected_gap='dynamic_or_external'),
        dict(id='budget-coherent-unit', query='bulk_transform', scope='local', required=[unit('whole-definition', [span('bulk.py', 1, 352)])], answerable=True),
    ]
    if split == 'reserved':
        # A branch with two dependencies, shared policy state and a caller in
        # another module. This is a different topology, not renamed dev cases.
        source['entry.py'] = ('from branch import route_adjustment\nfrom policy import capacity_rule\n\n'
                              'def dispatch_value(amount):\n    if amount < 0:\n        return capacity_rule(0)\n'
                              '    return route_adjustment(amount)\n\n'
                              'def local_tag(amount):\n    return amount // 2\n')
        source.pop('bridge.py')
        source['branch.py'] = ('from policy import capacity_rule\n\n'
                               'def route_adjustment(amount):\n    left = capacity_rule(amount)\n'
                               '    right = capacity_rule(amount + 2)\n    return max(left, right)\n')
        source['policy.py'] = ('CAPACITY = 37\n\ndef capacity_rule(amount):\n'
                               '    if amount > CAPACITY:\n        return CAPACITY\n    return amount\n')
        source['caller.py'] = ('from entry import dispatch_value\n\ndef run_batch(amounts):\n'
                               '    return [dispatch_value(amount) for amount in amounts]\n')
        direct = unit('entry-body', [span('entry.py', 4, 7)])
        middle = unit('branch-body', [span('branch.py', 3, 6)])
        leaf = unit('capacity-rule', [span('policy.py', 1, 6)],
                    [span('policy.py', 1, 1), span('policy.py', 4, 6)])
        c[0].update(required=[unit('local-body', [span('entry.py', 9, 10)])])
        c[1].update(required=[direct])
        c[2].update(required=[direct, middle, leaf])
        c[3].update(query='capacity_rule', scope='callers', required=[leaf, direct, middle])
    control = dict(split=split, case_version=1, budgets=[16384, 3000], cases=c,
                   obligation_symbols=[dict(name=root_name, path='entry.py'), dict(name=middle_name, path='branch.py' if split == 'reserved' else 'bridge.py'), dict(name=leaf_name, path='policy.py')],
                   file_sha256={p:digest(s.encode()) for p,s in source.items()})
    directory = HERE / 'fixtures' / split
    directory.mkdir(parents=True, exist_ok=True)
    for path, text in source.items():
        (directory / path).write_text(text)
    controls = HERE / 'controls'
    controls.mkdir(exist_ok=True)
    (controls / f'{split}.json').write_text(json.dumps(control, indent=2) + '\n')
    return dict(control_sha256=digest((controls / f'{split}.json').read_bytes()), file_sha256=control['file_sha256'], cases=len(c))


def main():
    frozen = {split:build(split, *args) for split,args in [
        ('dev', ('invoice_total', 'apply_fee', 'base_rule', 7)),
        ('reserved', ('dispatch_value', 'route_adjustment', 'capacity_rule', 11)),
    ]}
    (HERE / 'frozen-cases.json').write_text(json.dumps(dict(version=1, frozen_before_policy_evaluation=True, splits=frozen), indent=2) + '\n')
    print(json.dumps(dict(status='frozen', splits={k:v['cases'] for k,v in frozen.items()})))


if __name__ == '__main__':
    main()
