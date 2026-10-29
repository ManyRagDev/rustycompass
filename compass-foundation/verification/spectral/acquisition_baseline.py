#!/usr/bin/env python3
"""Public-CLI baseline on frozen sources; no production imports or hidden query.

Default split is development. Reserved measurement is an explicit final-phase
operation. Only source fixtures are copied into the temporary indexed root.
"""
import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import selectors
import shutil
import subprocess
import tempfile
import time

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]


def digest(raw): return hashlib.sha256(raw).hexdigest()
def canonical(value): return json.dumps(value, ensure_ascii=False, separators=(',', ':'), sort_keys=True).encode()
def key(path): return base64.urlsafe_b64encode(path.encode()).rstrip(b'=').decode()
def decode(path): return base64.urlsafe_b64decode(path + '=' * (-len(path) % 4)).decode()


class MCP:
    def __init__(self, binary, config):
        self.process = subprocess.Popen([str(binary), 'mcp', '--config', str(config)],
                                        stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                        stderr=subprocess.PIPE, bufsize=0)
        self.selector = selectors.DefaultSelector()
        self.selector.register(self.process.stdout, selectors.EVENT_READ)
        self.buffer = b''
        self.identity = 0
        self.measurements = []
        self.request('initialize', dict(protocolVersion='2025-11-25', capabilities={},
                                       clientInfo=dict(name='spectral-independent-controls', version='1')))
        self.send(dict(jsonrpc='2.0', method='notifications/initialized'))

    def send(self, value):
        self.process.stdin.write(canonical(value) + b'\n')
        self.process.stdin.flush()

    def request(self, method, params):
        self.identity += 1
        begin = time.perf_counter_ns()
        self.send(dict(jsonrpc='2.0', id=self.identity, method=method, params=params))
        while b'\n' not in self.buffer:
            assert self.selector.select(120), 'MCP timeout'
            raw = os.read(self.process.stdout.fileno(), 65536)
            assert raw, 'MCP EOF'
            self.buffer += raw
        raw, self.buffer = self.buffer.split(b'\n', 1)
        result = json.loads(raw)
        assert result.get('id') == self.identity and 'error' not in result, result
        self.measurements.append(dict(method=method, response_envelope_bytes=len(raw),
                                      wall_ns=time.perf_counter_ns() - begin))
        return result['result']

    def retrieve(self, arguments):
        result = self.request('tools/call', dict(name='compass_retrieve', arguments=arguments))
        assert not result['isError'], result
        assert json.loads(result['content'][0]['text']) == result['structuredContent']
        return result['structuredContent']

    def close(self):
        self.process.stdin.close()
        assert self.process.wait(timeout=10) == 0
        assert not self.process.stderr.read()
        self.selector.close()


def covers(intervals, start, end):
    cursor = start
    for a, b in sorted(intervals):
        if a <= cursor:
            cursor = max(cursor, b)
    return cursor >= end


def line_span(raw, spec):
    lines = raw.splitlines(keepends=True)
    assert 1 <= spec['start_line'] <= spec['end_line'] <= len(lines), spec
    a = sum(len(x) for x in lines[:spec['start_line'] - 1])
    b = sum(len(x) for x in lines[:spec['end_line']])
    return a, len(raw[:b].rstrip(b'\r\n'))


def support(case, sources, intervals):
    units = []
    for unit in case['required']:
        hit = any(all(covers(intervals.get(span['path'], []), *line_span(sources[span['path']], span))
                      for span in alternative) for alternative in unit['alternatives'])
        units.append(dict(id=unit['id'], delivered=hit))
    return dict(required_units=len(units), delivered_units=sum(x['delivered'] for x in units),
                support_complete=all(x['delivered'] for x in units) if units else None,
                answerable=case['answerable'], units=units)


def verify_pack(pack, sources, generation, raw):
    assert pack['generation'] == generation
    assert pack['payload_bytes'] == len(raw.rstrip(b'\n')) <= pack['budget_bytes']
    intervals = {}
    source_bytes = redundant_bytes = 0
    for item in pack['items']:
        path = decode(item['path_key'])
        original = sources[path]
        a, b = int(item['start_byte']), int(item['end_byte'])
        assert 0 <= a <= b <= len(original)
        assert original[a:b].decode() == item['text']
        assert digest(original) == item['content_sha256']
        assert item['verification'] == 'full_object_sha256'
        prior = intervals.setdefault(path, [])
        points = sorted({a, b, *[max(a, min(b, p)) for pair in prior for p in pair]})
        redundant_bytes += sum(y - x for x, y in zip(points, points[1:]) if covers(prior, x, y))
        prior.append((a, b))
        source_bytes += b - a
    for obligation in pack.get('completion', {}).get('obligations', []):
        if obligation['status'] == 'satisfied':
            assert obligation['witnesses']
            by_span = {item['span_id']:item for item in pack['items']}
            for witness in obligation['witnesses']:
                item = by_span[witness['span_id']]
                assert int(item['start_byte']) <= int(witness['start_byte']) <= int(witness['end_byte']) <= int(item['end_byte'])
    return intervals, dict(source_bytes=source_bytes, redundant_source_bytes=redundant_bytes,
                           payload_bytes=pack['payload_bytes'], items=len(pack['items']),
                           paths=sorted(intervals))


def run(binary, split):
    control_path = HERE / 'controls' / f'{split}.json'
    control = json.loads(control_path.read_text())
    frozen = json.loads((HERE / 'frozen-cases.json').read_text())['splits'][split]
    assert digest(control_path.read_bytes()) == frozen['control_sha256']
    fixture_dir = HERE / 'fixtures' / split
    sources = {p.name:p.read_bytes() for p in fixture_dir.iterdir() if p.is_file()}
    assert {name:digest(raw) for name,raw in sources.items()} == frozen['file_sha256']
    measurements = []
    def cli(args, allow_error=False):
        with tempfile.NamedTemporaryFile() as metric:
            begin = time.perf_counter_ns()
            proc = subprocess.run(['/usr/bin/time', '-f', '%M', '-o', metric.name,
                                   str(binary), *map(str, args), '--json'], capture_output=True, timeout=120)
            elapsed = time.perf_counter_ns() - begin
            rss = Path(metric.name).read_text().strip().splitlines()[-1]
        value = json.loads(proc.stdout)
        measure = dict(operation=list(map(str,args[:2])), wall_ns=elapsed,
                       peak_rss_kib=int(rss), stdout_bytes=len(proc.stdout), exit_code=proc.returncode)
        measurements.append(measure)
        if proc.returncode:
            assert allow_error and value['error']['code'] == 'invalid_input', (value, proc.stderr)
        return value, proc.stdout, measure
    results = []
    with tempfile.TemporaryDirectory(prefix='compass-spectral-dev-') as temporary:
        tmp = Path(temporary)
        tree, store = tmp / 'source', tmp / 'store'
        shutil.copytree(fixture_dir, tree)
        prepared, _, _ = cli(['collection', 'prepare', '--root', tree, '--store', store])
        generation = prepared['generation']
        config_path = tmp / 'mcp.json'
        config_path.write_bytes(canonical(dict(sources=dict(control=dict(collection=str(store))))))
        client = MCP(binary, config_path)
        capabilities = client.request('tools/list', {})['tools']
        assert any(x['name'] == 'compass_retrieve' for x in capabilities)
        mcp_checks = []
        for case in control['cases']:
            for budget in control['budgets']:
                for arm in ['none', 'bfs', 'heat']:
                    args = ['retrieve', '--store', store, '--generation', generation,
                            '--mode', 'literal', '--query', case['query'], '--expansion', arm,
                            '--budget-bytes', budget]
                    pack, raw, measure = cli(args, allow_error=True)
                    record = dict(case_id=case['id'], scope_expected=case['scope'], arm=arm,
                                  budget_bytes=budget, public_acquisition_calls=1,
                                  measurement=measure)
                    if 'error' in pack:
                        record.update(status='budget_refusal', error=pack['error'],
                                      support_complete=False if case['required'] else None,
                                      required_units=len(case['required']), delivered_units=0)
                    else:
                        intervals, metrics = verify_pack(pack, sources, generation, raw)
                        record.update(status='pack', **metrics, **support(case, sources, intervals),
                                      next_cursor=pack.get('next_cursor'),
                                      reported_diagnostics=pack.get('diagnostics'),
                                      reported_completion=pack.get('completion'))
                        if case['id'] == 'same-seed-dependencies' and budget == 16384:
                            actual = client.retrieve(dict(source='control', generation=generation,
                                                          query=case['query'], mode='literal',
                                                          expansion=arm, budget_bytes=budget))
                            assert actual == pack, 'CLI/MCP pack disagreement'
                            mcp_checks.append(dict(case_id=case['id'], arm=arm, equality='passed'))
                    results.append(record)
        obligations = [dict(id=f'definition-{i}', kind='symbol_definition',
                            symbol=dict(name=s['name'], path_key=key(s['path'])))
                       for i,s in enumerate(control['obligation_symbols'])]
        obligation_results = []
        for complete in [True, False]:
            contract = dict(schema_version='1', intent='Explicit mechanism control; never seeds the open arms.',
                            obligations=obligations + ([] if complete else [dict(id='missing', kind='symbol_definition', symbol=dict(name='never_present_941'))]),
                            interpretation_pending=[])
            contract_path = tmp / 'contract.json'
            contract_path.write_bytes(canonical(contract))
            for budget in control['budgets']:
                pack, raw, measure = cli(['retrieve', '--store', store, '--generation', generation,
                                          '--query', control['cases'][1]['query'], '--mode', 'literal',
                                          '--expansion', 'none', '--budget-bytes', budget,
                                          '--contract', contract_path], allow_error=True)
                result = dict(complete_contract=complete, budget_bytes=budget, measurement=measure)
                if 'error' in pack:
                    result.update(status='budget_refusal', error=pack['error'])
                else:
                    _, metrics = verify_pack(pack, sources, generation, raw)
                    completion = pack['completion']
                    result.update(status=completion['status'], completion=completion, **metrics)
                    if budget == 16384:
                        assert (completion['status'] == 'SUFFICIENT_EVIDENCE_ACQUIRED') == complete
                        actual = client.retrieve(dict(source='control', generation=generation,
                                                      query=control['cases'][1]['query'], mode='literal',
                                                      expansion='none', budget_bytes=budget, contract=contract))
                        assert actual == pack, 'CLI/MCP obligation disagreement'
                        mcp_checks.append(dict(complete_contract=complete, equality='passed'))
                obligation_results.append(result)
        client.close()
    return dict(status='passed', split=split, generation=generation,
                binary_sha256=digest(binary.read_bytes()), controls_sha256=digest(control_path.read_bytes()),
                arms_semantics='Legacy none/BFS/fixed heat; no scope parameter or shared new materializer. This baseline does not isolate scale selection.',
                budgets=control['budgets'], mode='literal', semantic_model=None,
                acquisition_results=results, obligation_results=obligation_results,
                mcp_checks=mcp_checks, mcp_measurements=client.measurements,
                mcp_tool_schemas=capabilities,
                instrumentation_calls=1, measurements=measurements,
                conditions='Separate CLI processes; uncontrolled OS page cache; RSS from /usr/bin/time. Support counts only verified delivered source intervals. Candidate counts unavailable unless exposed by the response.')


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--binary', type=Path, default=ROOT / 'target/release/compass')
    p.add_argument('--split', choices=['dev', 'reserved'], default='dev')
    p.add_argument('--output', type=Path, default=HERE / 'legacy-baseline-dev.json')
    a = p.parse_args()
    result = run(a.binary.resolve(), a.split)
    a.output.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(dict(status=result['status'], split=a.split,
                          acquisitions=len(result['acquisition_results']), obligations=len(result['obligation_results']),
                          output=str(a.output))))


if __name__ == '__main__':
    main()
