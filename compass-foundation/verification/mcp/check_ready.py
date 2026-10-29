#!/usr/bin/env python3
"""Verify the installed launcher and fixed demo over real stdio, from any cwd."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import selectors
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]
DEMO = "Compass MCP pronto.\nmarcador-integracao-compass\nLeitura verificável.\nContinuação na mesma revisão.\n"


def check(command):
    process = subprocess.Popen([command], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE, cwd="/tmp", bufsize=0)
    selector = selectors.DefaultSelector()
    selector.register(process.stdout, selectors.EVENT_READ)
    buffer = b""
    next_id = 0

    def send(message):
        process.stdin.write(json.dumps(message).encode() + b"\n")
        process.stdin.flush()

    def request(method, params):
        nonlocal buffer, next_id
        next_id += 1
        send(dict(jsonrpc="2.0", id=next_id, method=method, params=params))
        deadline = time.monotonic() + 35
        while b"\n" not in buffer:
            remaining = deadline - time.monotonic()
            if remaining <= 0 or not selector.select(remaining):
                raise RuntimeError("MCP response timeout")
            chunk = os.read(process.stdout.fileno(), 65536)
            if not chunk:
                raise RuntimeError("Unexpected MCP EOF")
            buffer += chunk
        line, buffer = buffer.split(b"\n", 1)
        response = json.loads(line)
        assert response["id"] == next_id and "error" not in response, response
        return response["result"]

    def call(name, **arguments):
        result = request("tools/call", dict(name=name, arguments=arguments))
        assert not result["isError"], result
        value = result["structuredContent"]
        assert json.loads(result["content"][0]["text"]) == value
        return value

    try:
        initialized = request("initialize", dict(protocolVersion="2025-11-25", capabilities={},
                              clientInfo=dict(name="compass-ready-check", version="1")))
        assert initialized["protocolVersion"] == "2025-11-25"
        send(dict(jsonrpc="2.0", method="notifications/initialized"))
        tools = request("tools/list", {})["tools"]
        assert {t["name"] for t in tools} == {"compass_search", "compass_read", "compass_files", "compass_query", "compass_neighbors", "compass_retrieve"}
        retrieve_tool = next(t for t in tools if t["name"] == "compass_retrieve")
        assert "contract" in retrieve_tool["inputSchema"]["properties"]
        assert "acquisition_policy" in retrieve_tool["inputSchema"]["properties"]
        found = call("compass_search", source="demo", query="marcador-integracao-compass")
        assert len(found["matches"]) == 1 and found["matches"][0]["line"] == "2"
        key = found["matches"][0]["path_key"]
        first = call("compass_read", source="demo", path_key=key, start_line="1", max_lines=2)
        assert first["excerpt"]["next_start_line"] == "3"
        second = call("compass_read", source="demo", path_key=key, start_line="3", max_lines=2)
        assert second["excerpt"]["next_start_line"] is None
        assert first["snapshot"] == second["snapshot"] == found["snapshot"]
        assert first["excerpt"]["text"] + second["excerpt"]["text"] == DEMO
        assert first["file"]["expected_revision"]["content_sha256"] == hashlib.sha256(DEMO.encode()).hexdigest()
        source = call("compass_search", source="compass-source", query="pub fn read(")
        assert source["matches"], source
        hit = source["matches"][0]
        opened = call("compass_read", source="compass-source", path_key=hit["path_key"],
                      start_line=hit["line"], max_lines=5)
        assert "pub fn read(" in opened["excerpt"]["text"]
        pack = call("compass_retrieve", source="compass-demo", query="checkoutSignal", mode="literal", expansion="heat", budget_bytes=16384)
        assert pack["payload_bytes"] <= 16384 and pack["expansion"]["status"] == "complete"
        assert any("checkoutSignal" in item["text"] for item in pack["items"])
        assert any(item["expanded"] and "def total_amount" in item["text"] for item in pack["items"])
        semantic = call("compass_query", source="compass-demo", mode="semantic", query="calcular o preço total de uma compra", limit=3)
        assert any("def total_amount" in item["text"] for item in semantic["results"])
        managed = call("compass_read", source="compass-demo", generation=pack["generation"], path_key=pack["items"][0]["path_key"], max_lines=2)
        assert managed["status"] == "available" and managed["snapshot"]["snapshot_id"] == pack["generation"]
        contract = json.loads((ROOT / "docs/examples/checkout-contract.json").read_text())
        acquired = call("compass_retrieve", source="compass-demo", generation=pack["generation"],
                        query="checkoutSignal", mode="literal", budget_bytes=16384, contract=contract)
        assert acquired["schema_version"] == "2" and acquired["payload_bytes"] <= 16384
        assert acquired["completion"]["scope"] == "current_pack"
        assert acquired["completion"]["status"] == "SUFFICIENT_EVIDENCE_ACQUIRED"
        assert all(o["status"] == "satisfied" and o["witnesses"] for o in acquired["completion"]["obligations"])
        pending = json.loads((ROOT / "docs/examples/checkout-insufficient-contract.json").read_text())
        incomplete = call("compass_retrieve", source="compass-demo", generation=pack["generation"],
                          query="checkoutSignal", mode="literal", budget_bytes=16384, contract=pending)
        assert incomplete["completion"]["status"] == "INSUFFICIENT_EVIDENCE"
        assert incomplete["completion"]["obligations"][0]["status"] == "satisfied"
        spectral = call("compass_retrieve", source="compass-demo", generation=pack["generation"],
                        query="checkoutSignal", mode="literal", budget_bytes=16384,
                        acquisition_policy="spectral_window_v1", scope="dependencies")
        assert spectral["schema_version"] == "3" and spectral["next_cursor"] is None
        assert spectral["generation"] == pack["generation"] and spectral["payload_bytes"] <= 16384
        assert spectral["acquisition"]["policy"] == "spectral_window_v1"
        assert spectral["acquisition"]["state"] == "terminal" and spectral["acquisition"]["reason"]
        assert spectral["scope"] == {"kind": "dependencies", "origin": "provided"}
        assert "gaps" in spectral["coverage"] and "status" in spectral["structural"]
        assert spectral.get("completion", {}).get("status") != "SUFFICIENT_EVIDENCE_ACQUIRED"
        assert any("checkoutSignal" in item["text"] for item in spectral["items"])
        process.stdin.close()
        process.wait(timeout=5)
        assert process.returncode == 0 and not process.stderr.read()
        return dict(status="passed", command=command, protocol="2025-11-25",
                    checks=["initialize", "tools/list", "demo search/read/continuation",
                            "original bytes/hash", "same revision", "compass source search/read",
                            "managed heat pack", "managed semantic query", "managed verified read",
                            "contract discovery", "one-call current-pack sufficient evidence",
                            "interpretation pending blocks sufficiency", "spectral policy discovery",
                            "spectral terminal pack with explicit scope and fixed generation",
                            "foreign cwd", "clean EOF shutdown"],
                    snapshots={"demo": first["snapshot"], "compass-source": opened["snapshot"]})
    finally:
        if process.poll() is None:
            process.kill()
            process.wait(timeout=5)
        selector.close()
        for stream in (process.stdin, process.stdout, process.stderr):
            stream.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--command", default=str(ROOT / "scripts/compass-mcp.sh"))
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = check(args.command)
    encoded = json.dumps(result, ensure_ascii=False, indent=2) + "\n"
    if args.output:
        args.output.write_text(encoded)
    print(encoded, end="")
