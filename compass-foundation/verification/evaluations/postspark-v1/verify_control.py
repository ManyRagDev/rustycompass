#!/usr/bin/env python3
"""Check the independent control against frozen bytes; does not call Compass."""
import argparse
import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def verify(root=None):
    manifest = json.loads((HERE / "manifest.json").read_text())
    gold = json.loads((HERE / "gabarito.json").read_text())
    identity = sha(json.dumps(manifest["files"], sort_keys=True, separators=(",", ":")).encode())
    assert identity == manifest["corpus_id"] == gold["corpus_id"]
    root = Path(root or manifest["corpus_root"])
    data = {}
    for file in manifest["files"]:
        raw = (root / file["path"]).read_bytes()
        assert sha(raw) == file["sha256"] and len(raw) == file["size_bytes"], file["path"]
        data[file["path"]] = raw
    count = 0
    for case in gold["cases"]:
        for evidence in case["evidence"]:
            raw = data[evidence["path"]]
            parts = raw.split(b"\n")
            lines = [p + b"\n" for p in parts[:-1]] + ([parts[-1]] if parts[-1] else [])
            start, end = evidence["start_line"], evidence["end_line"]
            begin = sum(len(line) for line in lines[:start - 1])
            stop = sum(len(line) for line in lines[:end])
            excerpt = b"".join(lines[start - 1:end])
            assert (begin, stop) == (evidence["start_byte"], evidence["end_byte"])
            assert excerpt == raw[begin:stop] == evidence["text"].encode()
            assert sha(excerpt) == evidence["excerpt_sha256"]
            assert sha(raw) == evidence["file_sha256"]
            count += 1
    return dict(status="passed", files=len(data), cases=len(gold["cases"]),
                evidence_ranges=count, corpus_id=identity, checked_root=str(root),
                compass_queried=False)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", help="Optional original root to detect source drift")
    args = parser.parse_args()
    print(json.dumps(verify(args.root), ensure_ascii=False, indent=2))
