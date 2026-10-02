"""§AR-ci.3.3 — a test that reads an observation compiled only under a feature is
ignored in a build without that feature, and its ignore reason names the feature
as the gate passes it. The gate's own test run enables every such feature
(§AR-ci.3), so it cannot see a test that breaks the rule; a plain `cargo test`
would, and nothing in the gate runs one. This test therefore reads each observer's
test file as text, the way the parity test reads the hook list: a rustfmt'd test
file's top-level items open at column 0 and close at a column-0 `}`, so whatever
sits between one item's end and the next `fn` is that fn's attributes."""

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PRE_COMMIT = REPO_ROOT / ".pre-commit-config.yaml"

# Each feature the `cargo-test` hook passes, as it passes it, to the test file
# that reads its observation and the helper that reads it there.
OBSERVERS = {
    "grund/test-workspace-load-count": ("crates/grund-cli/tests/show_batch_contract.rs", "loads"),
}


def _top_level_fns(text):
    lines = text.splitlines()
    fns, attrs_start, i = [], 0, 0
    while i < len(lines):
        line = lines[i]
        started = re.match(r"(?:pub(?:\([^)]*\))? )?fn (\w+)", line)
        if started:
            end = i
            if not line.rstrip().endswith("}"):
                end = next((j for j in range(i + 1, len(lines)) if lines[j] == "}"), len(lines) - 1)
            fns.append(
                {
                    "name": started.group(1),
                    "attrs": "\n".join(lines[attrs_start:i]),
                    "body": "\n".join(lines[i + 1 : end + 1]),
                }
            )
            i = attrs_start = end + 1
            continue
        if line == "}" or re.match(r"(use|static|const) .*;$", line):
            attrs_start = i + 1
        i += 1
    return fns


def _calls(body, name):
    return re.search(rf"(?<![\w.]){re.escape(name)}\(", body) is not None


def _tests_reading(fns, reader):
    """The `#[test]` fns that reach `reader`, directly or through a helper."""
    reads = {fn["name"] for fn in fns if fn["name"] != reader and _calls(fn["body"], reader)}
    grew = True
    while grew:
        grew = False
        for fn in fns:
            if fn["name"] not in reads and fn["name"] != reader and any(_calls(fn["body"], r) for r in reads):
                reads.add(fn["name"])
                grew = True
    return [fn for fn in fns if fn["name"] in reads and "#[test]" in fn["attrs"]]


def _ignore_reason(attrs, cfg_feature):
    found = re.search(
        r'cfg_attr\(\s*not\(\s*feature\s*=\s*"' + re.escape(cfg_feature) + r'"\s*\)\s*,'
        r'\s*ignore\s*=\s*"([^"]*)"\s*,?\s*\)',
        attrs,
    )
    return found.group(1) if found else None


class FeatureGatedObserverTests(unittest.TestCase):
    def test_every_feature_the_test_hook_enables_has_its_observer_held_here(self):
        entry = re.search(r"entry:\s*(cargo test .*)$", PRE_COMMIT.read_text(encoding="utf-8"), re.M)
        self.assertIsNotNone(entry, "no `cargo test` hook entry")
        passed = set()
        for value in re.findall(r"--features[ =](\S+)", entry.group(1)):
            passed.update(value.split(","))
        self.assertEqual(set(OBSERVERS), passed)

    def test_a_test_reading_an_observer_is_ignored_naming_its_feature_without_it(self):
        for feature, (path, reader) in OBSERVERS.items():
            fns = _top_level_fns((REPO_ROOT / path).read_text(encoding="utf-8"))
            readers = _tests_reading(fns, reader)
            with self.subTest(feature=feature):
                self.assertTrue(readers, f"no test in {path} reads `{reader}` any more")
            for fn in readers:
                with self.subTest(feature=feature, test=fn["name"]):
                    reason = _ignore_reason(fn["attrs"], feature.rpartition("/")[2])
                    self.assertIsNotNone(
                        reason,
                        f'{path}: {fn["name"]} reads `{reader}` but has no '
                        f'`cfg_attr(not(feature = "{feature.rpartition("/")[2]}"), ignore = ...)`',
                    )
                    self.assertIn(feature, reason, f'{path}: {fn["name"]}\'s ignore reason does not name {feature}')


if __name__ == "__main__":
    unittest.main()
