#!/usr/bin/env python3
"""Black-box acceptance checks. Python is a laboratory, never the Compass engine.

Run: python3 verification/reference/acceptance.py --binary target/debug/compass
All file interventions happen under TemporaryDirectory. Expected classifications
come from manual_expectations.json and the contract, not a second diff algorithm.
"""

import argparse
import base64
import copy
import fcntl
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import unittest


HERE = Path(__file__).resolve().parent
FIXTURES = json.loads((HERE.parent / "fixtures/manual_expectations.json").read_text())
BINARY = None


def key(raw):
    if isinstance(raw, str):
        raw = raw.encode()
    return base64.urlsafe_b64encode(raw).rstrip(b"=").decode("ascii")


def raw_path(path_key):
    return base64.urlsafe_b64decode(path_key + "=" * (-len(path_key) % 4))


def unique_object(pairs):
    result = {}
    for name, value in pairs:
        if name in result:
            raise AssertionError(f"duplicate JSON output key {name!r}")
        result[name] = value
    return result


class Acceptance(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="compass-acceptance-")
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name)
        self.root = self.base / "workspace"
        self.root.mkdir()
        self.init = self.invoke("init", "--root", self.root)[0]

    def invoke(self, *args, exit_code=0):
        process = subprocess.run(
            [str(BINARY), *map(str, args), "--json"], capture_output=True,
            text=True, timeout=20,
        )
        self.assertEqual(process.returncode, exit_code, process.stdout + process.stderr)
        self.assertTrue(process.stdout.strip(), "--json stdout must contain one object")
        result = json.loads(process.stdout, object_pairs_hook=unique_object)
        self.assertIsInstance(result, dict)
        return result, process.stdout

    def snapshot(self, exit_code=0, root=None):
        root = root or self.root
        result, _ = self.invoke("snapshot", "--root", root, exit_code=exit_code)
        path = Path(os.fsdecode(raw_path(result["snapshot_path_base64"])))
        self.assertTrue(path.is_file(), result)
        snapshot = json.loads(path.read_text(), object_pairs_hook=unique_object)
        self.assertEqual(snapshot["snapshot_id"], result["snapshot_id"])
        self.assertEqual(snapshot["quality"], result["quality"])
        self.assertEqual(path.name, snapshot["snapshot_id"] + ".json")
        return path, snapshot

    def compare(self, baseline, current, exit_code=0):
        before = [baseline.read_bytes(), current.read_bytes()]
        result, raw = self.invoke(
            "compare", "--baseline", baseline, "--current", current, exit_code=exit_code
        )
        self.assertEqual([baseline.read_bytes(), current.read_bytes()], before,
                         "comparison must not advance or rewrite baseline")
        return result, raw

    def changes(self, report):
        return [(raw_path(c["path_key"]), c["classification"]) for c in report["changes"]]

    def entries(self, snapshot):
        entries = {raw_path(e["path_key"]): e for e in snapshot["entries"]}
        self.assertEqual(len(entries), len(snapshot["entries"]))
        return entries

    def write(self, name, content):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)
        path.chmod(0o600)
        return path

    def edit_policy(self, **limits):
        path = self.root / ".compass/policy.json"
        policy = json.loads(path.read_text())
        policy.update(limits)
        path.write_text(json.dumps(policy))

    def corrupted(self, original, mutate, name="tampered.json"):
        value = copy.deepcopy(original)
        mutate(value)
        path = self.base / name
        path.write_text(json.dumps(value))
        return path

    def test_F01_F04_F20_empty_then_same_state(self):
        first, s0 = self.snapshot()
        second, s1 = self.snapshot()
        self.assertEqual(s0["quality"], "complete")
        self.assertIsNotNone(s0["state_digest"])
        self.assertEqual(s0["state_digest"], s1["state_digest"])
        self.assertNotEqual(s0["snapshot_id"], s1["snapshot_id"])
        report, _ = self.compare(first, second)
        self.assertEqual(report["relation"], "equal")
        self.assertEqual(report["changes"], [])
        empty = self.write("empty", b"")
        added, manifest = self.snapshot()
        self.assertEqual(self.entries(manifest)[b"empty"]["size_bytes"], "0")
        self.assertEqual(self.changes(self.compare(first, added)[0]), [(b"empty", "added")])
        empty.unlink()
        removed, _ = self.snapshot()
        self.assertEqual(self.changes(self.compare(added, removed)[0]), [(b"empty", "removed")])

    def test_F02_F03_F05_manual_fixture_and_hashes(self):
        for name, content in FIXTURES["s0"].items():
            self.write(name, content.encode())
        first, initial = self.snapshot()
        observed = self.entries(initial)
        for name, vector in [("b.txt", FIXTURES["sha256_vectors"][0]),
                             ("a.txt", FIXTURES["sha256_vectors"][1])]:
            self.assertEqual(observed[name.encode()]["content_sha256"], vector["sha256"])
            self.assertEqual(observed[name.encode()]["size_bytes"], vector["size_bytes"])
        (self.root / "b.txt").unlink()
        for name, content in FIXTURES["s1"].items():
            self.write(name, content.encode())
        second, _ = self.snapshot()
        report, _ = self.compare(first, second)
        expected = [(c["path"].encode(), c["kind"]) for c in FIXTURES["s0_to_s1"]]
        self.assertEqual(self.changes(report), expected)
        self.assertEqual(report["relation"], "different")
        self.assertEqual(report["quality"], "complete")

    def test_F06_return_to_original_baseline(self):
        self.write("a.txt", b"abc")
        first, s0 = self.snapshot()
        self.write("a.txt", b"abd")
        middle, _ = self.snapshot()
        self.write("a.txt", b"abc")
        last, s2 = self.snapshot()
        self.assertEqual(self.compare(first, middle)[0]["relation"], "different")
        self.assertEqual(self.compare(first, last)[0]["relation"], "equal")
        self.assertEqual(s0["state_digest"], s2["state_digest"])
        self.assertNotEqual(s0["snapshot_id"], s2["snapshot_id"])

    def test_F07_rename_is_two_changes(self):
        path = self.write("a.txt", b"abc")
        first, _ = self.snapshot()
        path.rename(self.root / "renamed.txt")
        second, _ = self.snapshot()
        expected = [(c["path"].encode(), c["kind"]) for c in FIXTURES["rename"]]
        self.assertEqual(self.changes(self.compare(first, second)[0]), expected)

    @unittest.skipIf(os.geteuid() == 0, "root bypasses chmod permission denial")
    def test_F08_unreadable_file_is_unknown(self):
        path = self.write("private", b"abc")
        first, _ = self.snapshot()
        path.chmod(0)
        try:
            second, partial = self.snapshot(exit_code=3)
            self.assertEqual(self.entries(partial)[b"private"]["state"], "unreadable")
            self.assertIsNone(partial["state_digest"])
            report, _ = self.compare(first, second, exit_code=3)
            self.assertEqual(report["relation"], "undetermined")
            self.assertEqual(self.changes(report), [(b"private", "unknown")])
        finally:
            path.chmod(0o600)

    @unittest.skipIf(os.geteuid() == 0, "root bypasses chmod permission denial")
    def test_F09_directory_not_enumerated_does_not_prove_removal(self):
        self.write("private/child", b"abc")
        first, _ = self.snapshot()
        directory = self.root / "private"
        directory.chmod(0)
        try:
            second, partial = self.snapshot(exit_code=3)
            self.assertIsNone(partial["state_digest"])
            report, _ = self.compare(first, second, exit_code=3)
            self.assertEqual(report["relation"], "undetermined")
            self.assertEqual(self.changes(report), [(b"private/child", "unknown")])
            self.assertTrue(report["unknown_regions"])
        finally:
            directory.chmod(0o700)

    def test_partial_ancestor_overrides_even_successful_file_reads(self):
        self.write("a", b"abc")
        first, original = self.snapshot()

        def mark_partial(value):
            value["quality"] = "partial"
            value["state_digest"] = None
            value["directory_coverage"][0].update(state="unstable", reason="directory_changed")
            for entry in value["entries"]:
                if entry["path_key"] == key("a"):
                    entry["content_sha256"] = hashlib.sha256(b"abd").hexdigest()

        partial = self.corrupted(original, mark_partial, "partial-observation.json")
        report, _ = self.compare(first, partial, exit_code=3)
        self.assertEqual(report["relation"], "undetermined")
        self.assertIn((b"a", "unknown"), self.changes(report))
        self.assertEqual(report["counts"]["modified"], "0")
        self.assertEqual(report["counts"]["unchanged"], "0")
        self.assertTrue(report["unknown_regions"])

    def test_F11_links_special_files_and_policy_exclusions(self):
        self.write("real", b"abc")
        external = self.base / "external"
        external.mkdir()
        (external / "secret").write_bytes(b"must not be read")
        os.symlink("real", self.root / "internal")
        os.symlink(external, self.root / "external")
        os.symlink("cycle_b", self.root / "cycle_a")
        os.symlink("cycle_a", self.root / "cycle_b")
        os.mkfifo(self.root / "pipe")
        self.write(".env", b"secret")
        self.write(".env.production", b"secret")
        self.write("nested/.git/config", b"secret")
        self.write(".aws/credentials", b"secret")
        self.write(".ssh/id_example", b"secret")
        self.write("visible", b"normal")
        _, snapshot = self.snapshot()
        entries = self.entries(snapshot)
        self.assertEqual(snapshot["quality"], "complete")
        for name in [b"internal", b"external", b"cycle_a", b"cycle_b"]:
            self.assertEqual(entries[name]["state"], "excluded")
            self.assertEqual(entries[name]["reason"], "symlink_not_followed")
            self.assertNotIn("content_sha256", entries[name])
        self.assertEqual(entries[b"pipe"]["reason"], "unsupported_kind")
        for name in [b".env", b".env.production", b"nested/.git", b".aws", b".ssh"]:
            self.assertEqual(entries[name]["state"], "excluded")
        self.assertNotIn(b"external/secret", entries)
        self.assertNotIn(b"nested/.git/config", entries)
        self.assertEqual(entries[b"visible"]["state"], "observed")

    def test_F12_native_bytes_case_and_hardlinks(self):
        a = self.write("Case", b"abc")
        self.write("case", b"abd")
        os.link(a, self.root / "hardlink")
        raw_name = b"\xff-native"
        fd = os.open(os.fsencode(self.root) + b"/" + raw_name, os.O_CREAT | os.O_WRONLY, 0o600)
        with os.fdopen(fd, "wb") as stream:
            stream.write(b"native")
        _, snapshot = self.snapshot()
        entries = self.entries(snapshot)
        native_order = [raw_path(e["path_key"]) for e in snapshot["entries"]]
        self.assertEqual(native_order, sorted(native_order), "sort native bytes, not base64 keys")
        for name in [b"Case", b"case", b"hardlink", raw_name]:
            self.assertEqual(entries[name]["state"], "observed")
            self.assertEqual(entries[name]["path_key"], key(name))
        self.assertEqual(entries[b"Case"]["content_sha256"], entries[b"hardlink"]["content_sha256"])
        self.assertNotEqual(entries[b"Case"]["path_key"], entries[b"hardlink"]["path_key"])
        self.assertNotEqual(entries[b"Case"]["content_sha256"], entries[b"case"]["content_sha256"])

    def test_F12_native_root_receipts_round_trip_without_loss(self):
        native = self.base / os.fsdecode(b"root-\xff")
        native.mkdir()
        initialized, _ = self.invoke("init", "--root", native)
        for field, suffix in [("workspace_path", ".compass/workspace.json"),
                              ("policy_path", ".compass/policy.json")]:
            self.assertIn(field + "_base64", initialized,
                          "native path receipts require a lossless byte representation")
            decoded = raw_path(initialized[field + "_base64"])
            self.assertEqual(decoded, os.fsencode(native / suffix))
            self.assertEqual(initialized[field + "_base64"], key(decoded))
            self.assertTrue(os.path.isfile(decoded))
        source = native / "sample"
        source.write_bytes(b"abc")
        snapshots = []
        for content in [b"abc", b"abd"]:
            source.write_bytes(content)
            receipt, _ = self.invoke("snapshot", "--root", native)
            self.assertIn("snapshot_path_base64", receipt)
            decoded = raw_path(receipt["snapshot_path_base64"])
            self.assertEqual(decoded, os.fsencode(
                native / ".compass/snapshots" / (receipt["snapshot_id"] + ".json")))
            self.assertEqual(receipt["snapshot_path_base64"], key(decoded))
            path = Path(os.fsdecode(decoded))
            self.assertEqual(json.loads(path.read_text())["snapshot_id"], receipt["snapshot_id"])
            snapshots.append(path)
        report, _ = self.compare(*snapshots)
        self.assertEqual(self.changes(report), [(b"sample", "modified")])

    def test_policy_explicit_subtree_and_gitignore_do_not_silently_change_scope(self):
        self.write(".gitignore", b"ignored.bin\n")
        self.write("ignored.bin", bytes(range(256)))
        self.write("dist/generated", b"excluded by explicit policy")
        self.write("distribution/source", b"not the dist subtree")
        self.edit_policy(exclude_paths=[key("dist")])
        _, snapshot = self.snapshot()
        entries = self.entries(snapshot)
        self.assertEqual(entries[b"ignored.bin"]["state"], "observed")
        self.assertEqual(entries[b"ignored.bin"]["content_sha256"],
                         hashlib.sha256(bytes(range(256))).hexdigest())
        self.assertEqual(entries[b"dist"]["state"], "excluded")
        self.assertNotIn(b"dist/generated", entries)
        self.assertEqual(entries[b"distribution/source"]["state"], "observed")

    def test_F13_byte_content_is_not_normalized(self):
        self.write("line", b"one\r\ntwo\r\n")
        self.write("bom", b"\xef\xbb\xbfabc")
        first, _ = self.snapshot()
        self.write("line", b"one\ntwo\n")
        self.write("bom", b"abc")
        second, s1 = self.snapshot()
        self.assertEqual(self.changes(self.compare(first, second)[0]),
                         [(b"bom", "modified"), (b"line", "modified")])
        self.assertEqual(self.entries(s1)[b"line"]["content_sha256"],
                         hashlib.sha256(b"one\ntwo\n").hexdigest())

    def test_F14_other_workspace_and_policy_are_incompatible(self):
        self.write("a", b"abc")
        first, _ = self.snapshot()
        other = self.base / "other"
        other.mkdir()
        self.invoke("init", "--root", other)
        second, _ = self.snapshot(root=other)
        error, _ = self.compare(first, second, exit_code=2)
        self.assertIn("code", error["error"])
        self.assertIn("message", error["error"])
        self.edit_policy(max_file_bytes="100")
        third, _ = self.snapshot()
        error, _ = self.compare(first, third, exit_code=2)
        self.assertIn("code", error["error"])

    def test_F15_file_and_entry_limits_are_partial(self):
        self.write("a", b"abc")
        self.write("b", b"abc")
        self.edit_policy(max_file_bytes="2")
        _, partial = self.snapshot(exit_code=3)
        self.assertEqual(partial["quality"], "partial")
        self.assertIsNone(partial["state_digest"])
        for name in [b"a", b"b"]:
            entry = self.entries(partial)[name]
            self.assertEqual(entry["state"], "limit_exceeded")
            self.assertNotIn("content_sha256", entry)
        self.edit_policy(max_file_bytes=None, max_entries="1")
        _, partial = self.snapshot(exit_code=3)
        self.assertEqual(partial["quality"], "partial")
        self.assertIsNone(partial["state_digest"])

    def test_F15_global_byte_limit_counts_actual_reads(self):
        self.write("a", b"abc")
        self.write("b", b"def")
        self.edit_policy(max_total_bytes_read="4")
        _, partial = self.snapshot(exit_code=3)
        self.assertEqual(partial["quality"], "partial")
        self.assertIsNone(partial["state_digest"])
        self.assertLessEqual(int(partial["stats"]["bytes_read"]), 4)
        self.assertTrue(any(e["state"] == "limit_exceeded" for e in partial["entries"]))

    def test_F16_concurrent_writers_preserve_identity_and_records(self):
        self.write("a", b"abc")
        command = [str(BINARY), "snapshot", "--root", str(self.root), "--json"]
        processes = [subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                      text=True) for _ in range(3)]
        ids = set()
        for process in processes:
            stdout, stderr = process.communicate(timeout=20)
            self.assertEqual(process.returncode, 0, stdout + stderr)
            result = json.loads(stdout, object_pairs_hook=unique_object)
            ids.add(result["snapshot_id"])
            snapshot = json.loads(Path(os.fsdecode(raw_path(result["snapshot_path_base64"]))).read_text())
            self.assertEqual(snapshot["workspace_id"], self.init["workspace_id"])
        self.assertEqual(len(ids), 3)
        again = self.invoke("init", "--root", self.root)[0]
        self.assertEqual(again["workspace_id"], self.init["workspace_id"])
        self.assertEqual(len(list((self.root / ".compass/snapshots").glob("*.json"))), 3)

    def test_F16_sigint_while_waiting_for_writer_lock(self):
        with (self.root / ".compass/lock").open("r+b") as lock:
            fcntl.flock(lock, fcntl.LOCK_EX)
            process = subprocess.Popen(
                [str(BINARY), "snapshot", "--root", str(self.root), "--json"],
                stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
            )
            try:
                deadline = time.monotonic() + 5
                handler_ready = False
                while time.monotonic() < deadline and process.poll() is None:
                    status = Path(f"/proc/{process.pid}/status").read_text()
                    caught = next(line.split()[1] for line in status.splitlines()
                                  if line.startswith("SigCgt:"))
                    if int(caught, 16) & (1 << (signal.SIGINT - 1)):
                        handler_ready = True
                        break
                    time.sleep(0.01)
                self.assertTrue(handler_ready, "CLI must install SIGINT handler before waiting")
                process.send_signal(signal.SIGINT)
                stdout, stderr = process.communicate(timeout=5)
                self.assertEqual(process.returncode, 130, stdout + stderr)
                self.assertEqual(json.loads(stdout)["error"]["code"], "cancelled")
                self.assertEqual(list((self.root / ".compass/snapshots").iterdir()), [])
            finally:
                if process.poll() is None:
                    process.kill()
                    process.communicate()
                fcntl.flock(lock, fcntl.LOCK_UN)

    def test_F18_reject_corruption_duplicate_keys_and_unknown_enum(self):
        self.write("a", b"abc")
        valid, snapshot = self.snapshot()
        cases = {
            "digest": lambda s: s.update(state_digest="0" * 64),
            "enum": lambda s: s["entries"][0].update(state="future_state"),
            "schema": lambda s: s.update(schema_version="999"),
            "duplicate_entry": lambda s: s["entries"].append(copy.deepcopy(s["entries"][0])),
            "bad_path": lambda s: s["entries"][0].update(path_key=key(b"../escape")),
            "unknown_coverage": lambda s: s["directory_coverage"][0].update(state="future_state"),
            "no_root_coverage": lambda s: s.update(directory_coverage=[]),
            "wrong_counts": lambda s: s["stats"].update(observed_files="999"),
            "bad_window": lambda s: s.update(finished_at="2000-01-01T00:00:00Z"),
            "invalid_nested_policy": lambda s: s["policy"].update(read_attempts="1"),
        }
        for name, mutate in cases.items():
            with self.subTest(name=name):
                path = self.corrupted(snapshot, mutate, name + ".json")
                error, _ = self.compare(valid, path, exit_code=1)
                self.assertIn("code", error["error"])
                self.assertIn("message", error["error"])
                self.assertNotIn("changes", error)
        for name, contents in [
            ("invalid", "{"),
            ("duplicate", '{"schema_version":"1",' + valid.read_text().lstrip()[1:]),
        ]:
            path = self.base / (name + ".json")
            path.write_text(contents)
            self.compare(valid, path, exit_code=1)

    def test_F19_determinism_mtime_and_executability(self):
        file = self.write("a", b"abc")
        first, s0 = self.snapshot()
        before = file.stat()
        os.utime(file, ns=(before.st_atime_ns, before.st_mtime_ns + 1_000_000_000))
        second, s1 = self.snapshot()
        self.assertEqual(s0["state_digest"], s1["state_digest"])
        report, raw = self.compare(first, second)
        self.assertEqual(report["relation"], "equal")
        self.assertEqual(self.compare(first, second)[1], raw)
        file.chmod(0o700)
        third, s2 = self.snapshot()
        self.assertNotEqual(s0["state_digest"], s2["state_digest"])
        report, raw = self.compare(first, third)
        self.assertEqual(self.changes(report), [(b"a", "metadata_changed")])
        self.assertEqual(self.compare(first, third)[1], raw)

    def test_F20_file_link_scope_transitions(self):
        path = self.write("entry", b"abc")
        first, _ = self.snapshot()
        path.unlink()
        os.symlink("missing", path)
        second, _ = self.snapshot()
        self.assertEqual(self.changes(self.compare(first, second)[0]), [(b"entry", "left_scope")])
        path.unlink()
        self.write("entry", b"abc")
        third, _ = self.snapshot()
        self.assertEqual(self.changes(self.compare(second, third)[0]), [(b"entry", "entered_scope")])

    def test_F20_file_directory_transitions(self):
        path = self.write("entry", b"abc")
        first, _ = self.snapshot()
        path.unlink()
        self.write("entry/child", b"abc")
        second, _ = self.snapshot()
        self.assertEqual(self.changes(self.compare(first, second)[0]),
                         [(b"entry", "removed"), (b"entry/child", "added")])
        self.assertEqual(self.changes(self.compare(second, first)[0]),
                         [(b"entry", "added"), (b"entry/child", "removed")])

    def test_offline_comparison_and_explicit_baseline(self):
        self.write("a", b"abc")
        first, _ = self.snapshot()
        self.write("a", b"abd")
        second, _ = self.snapshot()
        expected = self.compare(first, second)[1]
        exported0, exported1 = self.base / "s0.json", self.base / "s1.json"
        shutil.copyfile(first, exported0)
        shutil.copyfile(second, exported1)
        shutil.rmtree(self.root)
        self.assertEqual(self.compare(exported0, exported1)[1], expected)

    def test_errors_are_single_json_objects(self):
        error, _ = self.invoke("snapshot", "--root", self.base, exit_code=1)
        self.assertIn("code", error["error"])
        self.assertIn("message", error["error"])
        error, _ = self.invoke("compare", exit_code=2)
        self.assertIn("code", error["error"])
        self.assertIn("message", error["error"])
        self.edit_policy(max_file_bytes="0")
        self.invoke("snapshot", "--root", self.root, exit_code=2)
        identity = self.root / ".compass/workspace.json"
        self.edit_policy(max_file_bytes=None)
        identity.write_text("{bad")
        self.invoke("init", "--root", self.root, exit_code=1)
        self.assertEqual(identity.read_text(), "{bad")


def main():
    global BINARY
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    args, remaining = parser.parse_known_args()
    BINARY = args.binary.resolve()
    if not BINARY.is_file():
        parser.error(f"build compass first; binary does not exist: {BINARY}")
    unittest.main(argv=[sys.argv[0], *remaining], verbosity=2)


if __name__ == "__main__":
    main()
