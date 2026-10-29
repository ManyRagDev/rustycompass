# Public obligation-pack controls

`controls.json` records manually authored expected source ranges and directed facts.
`frozen.json` preserves its SHA256 and every original fixture hash, before the new
implementation was inspected or the integrated campaign executed. DEV and heldout
are distinct synthetic collections. No private or displaced evaluation key is used.

Run the oracle alone:

```sh
python3 verification/acquisition/acceptance.py --self-test
```

After building the current binary, run CLI and real stdio MCP acceptance:

```sh
python3 verification/acquisition/acceptance.py --binary target/debug/compass --output verification/acquisition/results.json
```

The campaign checks full declarations/bodies (including >80-line definitions),
scoped homonyms, exact scoped literals, direct directed calls/imports, current-item witness
ranges, source bytes/hash/line offsets, budget refusal, ambiguous/missing/dynamic
relations, interpretation pending, request limits, transport equivalence, updates,
restarts, retained generations, and retrieval after removing the live source tree.
The original fixture/control freeze predates implementation inspection. Import direction
checks and hybrid-mode probes were added afterward from those same source bytes.
`development-results.json` records the development binary only; final evidence is
written to `results.json` after the release binary is ready.

Paired normal retrieval uses the identical source, query, generation and byte budget.
Source-byte overlap is reported separately from JSON payload overhead. CLI latency
is elapsed wall time and RSS is GNU time's per-process maximum, with uncontrolled
OS cache. MCP resource telemetry is not available. No token-cost estimate or general
quality claim is made. `results.json` is generated only by an actual campaign run.
