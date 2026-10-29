#!/usr/bin/env python3
"""Reproducible local CLI workload; measurements are descriptive, not targets."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import tempfile
import time
from datetime import datetime, timezone


def measured(binary, *args):
    # GNU time starts the CLI from a small native process. Measuring a child
    # directly from Python can include Python's inherited RSS before exec.
    with tempfile.TemporaryDirectory(prefix="compass-time-") as temporary:
        metrics_path = Path(temporary) / "metrics.txt"
        start = time.perf_counter_ns()
        child = subprocess.run(["/usr/bin/time", "-f", "%U %S %M", "-o", str(metrics_path),
                                str(binary), *args, "--json"], capture_output=True, timeout=60)
        wall_ns = time.perf_counter_ns() - start
        raw, error = child.stdout, child.stderr
        if child.returncode != 0:
            raise RuntimeError((args, child.returncode, raw, error))
        user_seconds, system_seconds, rss_kib = metrics_path.read_text().split()
        result = json.loads(raw)
        return result, {
            "wall_ms": round(wall_ns / 1_000_000, 3),
            "user_cpu_ms": round(float(user_seconds) * 1000, 3),
            "system_cpu_ms": round(float(system_seconds) * 1000, 3),
            "max_rss_kib": int(rss_kib),
            "stdout_bytes": len(raw),
            "exit_code": child.returncode,
        }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    binary = args.binary.resolve(strict=True)
    measurements = []
    cpu_model = next((line.split(":", 1)[1].strip() for line in Path("/proc/cpuinfo").read_text().splitlines()
                      if line.startswith("model name")), "unavailable")
    memory_kib = next((int(line.split()[1]) for line in Path("/proc/meminfo").read_text().splitlines()
                       if line.startswith("MemTotal:")), None)

    def run(label, *arguments):
        result, metrics = measured(binary, *arguments)
        measurements.append({"operation": label, **metrics})
        return result

    with tempfile.TemporaryDirectory(prefix="compass-profile-") as temporary:
        root = Path(temporary)
        corpus_digest = hashlib.sha256(b"compass-profile-corpus-v1\0")
        for index in range(1000):
            relative = f"dir-{index // 10:03d}/file-{index:04d}.bin"
            content = index.to_bytes(8, "little") + (bytes(range(256)) * 16)[8:]
            path = root / relative
            path.parent.mkdir(exist_ok=True)
            path.write_bytes(content)
            corpus_digest.update(len(relative.encode()).to_bytes(8, "little"))
            corpus_digest.update(relative.encode())
            corpus_digest.update(len(content).to_bytes(8, "little"))
            corpus_digest.update(content)
        run("init", "init", "--root", str(root))
        first = run("snapshot_initial", "snapshot", "--root", str(root))
        repeat = run("snapshot_unchanged", "snapshot", "--root", str(root))
        equal = run("compare_equal", "compare", "--baseline", first["snapshot_path"],
                    "--current", repeat["snapshot_path"])
        assert first["state_digest"] == repeat["state_digest"]
        assert equal["relation"] == "equal" and equal["counts"]["unchanged"] == "1000"
        changed_path = root / "dir-000/file-0000.bin"
        data = bytearray(changed_path.read_bytes())
        data[0] ^= 0xff
        changed_path.write_bytes(data)
        (root / "dir-099/file-0999.bin").unlink()
        (root / "new-empty.bin").write_bytes(b"")
        current = run("snapshot_changed", "snapshot", "--root", str(root))
        changes = run("compare_changed", "compare", "--baseline", first["snapshot_path"],
                      "--current", current["snapshot_path"])
        expected = {"added": "1", "removed": "1", "modified": "1", "unchanged": "998",
                    "metadata_changed": "0", "left_scope": "0", "entered_scope": "0", "unknown": "0"}
        assert changes["relation"] == "different" and changes["counts"] == expected
        snapshots = [json.loads(Path(r["snapshot_path"]).read_bytes())
                     for r in [first, repeat, current]]
        assert all(s["quality"] == "complete" for s in snapshots)
        assert [s["stats"]["bytes_read"] for s in snapshots] == ["4096000", "4096000", "4091904"]
        records = [p for p in (root / ".compass").rglob("*") if p.is_file()]
        report = {
            "recorded_at_utc": datetime.now(timezone.utc).isoformat(),
            "scope": "One local run; no cold-cache, scale or product-efficacy claim.",
            "measurement": "wall: perf_counter_ns including GNU time launch; CPU/RSS: GNU time for CLI child, CPU rounded to 10ms, RSS in KiB; filesystem cache not flushed; files freshly generated",
            "environment": {"platform": platform.platform(), "architecture": platform.machine(),
                            "logical_cpus": os.cpu_count(), "cpu_model": cpu_model,
                            "physical_memory_kib": memory_kib, "python": platform.python_version()},
            "build": {"profile": "release", "producer_build": snapshots[0]["producer_build"],
                      "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
                      "binary_bytes": binary.stat().st_size},
            "workload": {"generator": "verification/reference/profile.py", "initial_files": 1000,
                         "initial_directories": 100, "bytes_per_file": 4096, "initial_bytes": 4096000,
                         "corpus_sha256": corpus_digest.hexdigest(),
                         "edits": "one byte modified; one 4096-byte file removed; one empty file added"},
            "operations": measurements,
            "snapshot_stats": [s["stats"] for s in snapshots],
            "comparison_counts": changes["counts"],
            "disk": {"snapshot_logical_bytes": [Path(r["snapshot_path"]).stat().st_size for r in [first, repeat, current]],
                     "state_regular_file_count": len(records),
                     "state_regular_files_logical_bytes": sum(p.stat().st_size for p in records),
                     "state_regular_files_allocated_bytes": sum(p.stat().st_blocks * 512 for p in records),
                     "excludes": "directory metadata; compiler/dependency artifacts; source corpus"},
            "assertions": "equal repeat; exact manual changes; complete observations; expected bytes read",
        }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"output": str(args.output), "operations": measurements}, indent=2))


if __name__ == "__main__":
    main()
