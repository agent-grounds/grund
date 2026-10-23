"""§AR-ci.3.3 — a test that reads an observation compiled only under a feature is
ignored in a build without that feature, and its ignore reason names the feature
as the gate passes it. The gate's own test run enables every such feature
(§AR-ci.3), so it cannot see a test that breaks the rule; a plain `cargo test`
would, and nothing in the gate runs one. This test therefore reads each observer's
test file as text, the way the parity test reads the hook list: a rustfmt'd test
file's top-level items open at column 0 and close at a column-0 `}`, so whatever
sits between one item's end and the next `fn` is that fn's attributes.

§AR-ci.3.3.1 — a featured test build does not make a featured binary: the
uplifted path `CARGO_BIN_EXE_<name>` names is re-linked by every later build of
the package, whatever its features, so an observed run reads its count from a
process the test's own build made. Read the same way, no fn in an observer's file
or the modules it declares may reach both the observer's variable and
`CARGO_BIN_EXE_`, itself or through a helper it calls."""

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PRE_COMMIT = REPO_ROOT / ".pre-commit-config.yaml"

# Each feature the `cargo-test` hook passes, as it passes it, to the test file
# that reads its observation, the helper that reads it there, and the variable
# that asks the observed process for it.
OBSERVERS = {
    "grund/test-workspace-load-count": (
        "crates/grund-cli/tests/show_batch_contract.rs",
        "loads",
        "GRUND_TEST_WORKSPACE_LOAD_LOG",
    ),
}
RELINKED = "CARGO_BIN_EXE_"


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


def _module_files(path):
    """The files a test crate root declares as modules: `#[path]`, or `<name>.rs`."""
    text, found = path.read_text(encoding="utf-8"), []
    for given, name in re.findall(r'^(?:#\[path = "([^"]+)"\]\n)?(?:pub(?:\([^)]*\))? )?mod (\w+);$', text, re.M):
        for candidate in [given] if given else [f"{name}.rs", f"{name}/mod.rs"]:
            if (path.parent / candidate).is_file():
                found.append(path.parent / candidate)
                break
    return found


def _uncommented(body):
    return "\n".join(re.sub(r"(^|\s)//.*$", "", line) for line in body.splitlines())


def _spellings(text, needle):
    """`needle` and every top-level const or static that holds it under a name."""
    aliases = re.findall(rf"^(?:pub(?:\([^)]*\))? )?(?:const|static) (\w+)\b.*{re.escape(needle)}", text, re.M)
    return [re.escape(needle)] + [rf"\b{alias}\b" for alias in aliases]


def _reaching_both(fns, first, second):
    """The fns that reach a `first` spelling and a `second` one, in their own body or
    through a top-level fn they call. Only the innermost are returned, those none of
    whose callees reaches both, so one mixed helper is named once rather than once
    for every test that calls it."""
    bodies = {fn["name"]: _uncommented(fn["body"]) for fn in fns}
    callees = {
        name: {other for other in bodies if other != name and _calls(body, other)}
        for name, body in bodies.items()
    }

    def reach(spellings):
        reached = {n for n, b in bodies.items() if any(re.search(s, b) for s in spellings)}
        grew = True
        while grew:
            grew = False
            for name in bodies:
                if name not in reached and callees[name] & reached:
                    reached.add(name)
                    grew = True
        return reached

    both = reach(first) & reach(second)
    return [fn for fn in fns if fn["name"] in both and not callees[fn["name"]] & both]


def _ignore_reason(attrs, cfg_feature):
    found = re.search(
        r'cfg_attr\(\s*not\(\s*feature\s*=\s*"' + re.escape(cfg_feature) + r'"\s*\)\s*,'
        r'\s*ignore\s*=\s*"([^"]*)"\s*,?\s*\)',
        attrs,
    )
    return found.group(1) if found else None


class FeatureGatedObserverTests(unittest.TestCase):
    def test_every_feature_the_test_hook_enables_has_its_observer_held_here(self):
        # The hook is found by its id, whatever runner its entry spells (§AR-ci.3).
        entry = re.search(
            r"^\s*- id: cargo-test\n(?:\s+(?!- id:)\S.*\n)*?\s+entry:\s*(.*)$",
            PRE_COMMIT.read_text(encoding="utf-8"),
            re.M,
        )
        self.assertIsNotNone(entry, "no `cargo-test` hook entry")
        passed = set()
        for value in re.findall(r"--features[ =](\S+)", entry.group(1)):
            passed.update(value.split(","))
        self.assertEqual(set(OBSERVERS), passed)

    def test_a_test_reading_an_observer_is_ignored_naming_its_feature_without_it(self):
        for feature, (path, reader, _variable) in OBSERVERS.items():
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

    def test_an_observed_run_never_spawns_an_artifact_another_build_relinks(self):
        for feature, (path, _reader, variable) in OBSERVERS.items():
            files = [REPO_ROOT / path, *_module_files(REPO_ROOT / path)]
            texts = {file.relative_to(REPO_ROOT): file.read_text(encoding="utf-8") for file in files}
            fns = [dict(fn, file=file) for file, text in texts.items() for fn in _top_level_fns(text)]
            every = "\n".join(texts.values())
            observed = _spellings(every, variable)
            relinked = _spellings(every, RELINKED)
            with self.subTest(feature=feature):
                self.assertTrue(
                    any(re.search(s, _uncommented(fn["body"])) for fn in fns for s in observed),
                    f"no fn in {path} or its modules names {variable} any more, so its "
                    f"observer is never asked for a count (§AR-ci.3.3.1)",
                )
            for fn in _reaching_both(fns, observed, relinked):
                with self.subTest(feature=feature, fn=fn["name"]):
                    self.fail(
                        f'{fn["file"]}: `{fn["name"]}` reaches both {variable} and `{RELINKED}`, '
                        f"itself or through a helper it calls. `{RELINKED}<name>` is the uplifted "
                        f"binary every later build of the package re-links, whatever its features, "
                        f"so an observed run there can read a count nobody wrote (§AR-ci.3.3.1). "
                        f"Run the observed command from a process this test's own build made - "
                        f"re-spawn std::env::current_exe() into the CLI - and keep the runs that "
                        f"observe nothing, which may use `{RELINKED}<name>`, in fns that never "
                        f"reach {variable}."
                    )


if __name__ == "__main__":
    unittest.main()
