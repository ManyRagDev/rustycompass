#!/usr/bin/env python3
"""Check that the black-box suite rejects three controlled response mutations.

These are public-protocol mutations, not a claim of exhaustive source mutation
coverage. They leave disk operations untouched and inject only the stated defect
into compare output. No generated wrapper escapes its temporary directory.
"""

import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile


WRAPPER = '''#!{python}
import base64, json, subprocess, sys
p = subprocess.run([{binary}] + sys.argv[1:], capture_output=True)
out = p.stdout
if "compare" in sys.argv and p.returncode in (0, 3):
    r = json.loads(out)
    mode = {mode}
    if mode == "always_equal":
        r["changes"] = []
        r["relation"] = "equal"
    elif mode == "omit_empty":
        omitted = {{"Yi50eHQ", "Yy50eHQ"}}  # b.txt and c.txt in fixed S0/S1
        r["changes"] = [c for c in r["changes"] if c["path_key"] not in omitted]
    elif mode == "unreadable_as_removed":
        for c in r["changes"]:
            if c["classification"] == "unknown":
                c["classification"] = "removed"
    out = (json.dumps(r) + "\\n").encode()
sys.stdout.buffer.write(out)
sys.stderr.buffer.write(p.stderr)
sys.exit(p.returncode)
'''


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    args = parser.parse_args()
    binary = args.binary.resolve()
    if not binary.is_file():
        parser.error("build Compass before mutation checks")
    suite = Path(__file__).with_name("acceptance.py")
    cases = [
        ("always_equal", "Acceptance.test_F02_F03_F05_manual_fixture_and_hashes"),
        ("omit_empty", "Acceptance.test_F02_F03_F05_manual_fixture_and_hashes"),
        ("unreadable_as_removed", "Acceptance.test_F08_unreadable_file_is_unknown"),
    ]
    skipped = 0
    with tempfile.TemporaryDirectory(prefix="compass-mutants-") as directory:
        for mode, case in cases:
            if mode == "unreadable_as_removed" and os.geteuid() == 0:
                print("SKIP unreadable_as_removed: root bypasses permission denial")
                skipped += 1
                continue
            wrapper = Path(directory) / mode
            wrapper.write_text(WRAPPER.format(python=sys.executable,
                                             binary=repr(str(binary)), mode=repr(mode)))
            wrapper.chmod(0o700)
            process = subprocess.run([sys.executable, str(suite), "--binary", str(wrapper), case],
                                     capture_output=True, text=True, timeout=60)
            if process.returncode != 1 or "AssertionError" not in process.stderr:
                print(process.stdout + process.stderr, file=sys.stderr)
                raise SystemExit(f"FAIL: mutation {mode} was not rejected by an assertion")
            print(f"REJECTED {mode}: {case}")
    print(json.dumps({"mutations_rejected": len(cases) - skipped, "skipped": skipped}))


if __name__ == "__main__":
    main()
