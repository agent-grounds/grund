"""§AR-ci-fixture-cleanup.4 — a fixture's cleanup failure never aborts its test binary.

A fixture `Drop` that raises its removal failure as a panic raises it a second
time when the test holding it has already failed. Rust aborts the whole process
on a panic during unwinding, and on windows-latest that is the silent
`0xC0000409` of agent-grounds/grund#1226, which leaves the rest of the shard
unrun.

This check reads every `impl Drop` under `crates/` whose body removes a tree,
meaning it names `remove_dir_all` or calls the shared helper
`remove_fixture_tree`. Each one must take one of three shapes:

- it raises nothing (`let _ = fs::remove_dir_all(..)`, or only a call to
  `remove_fixture_tree`);
- or it tests `std::thread::panicking()` before the first thing it raises: an
  `.unwrap()`, `.expect(..)`, `panic!`, `assert!` or the like;
- and every definition of `fn remove_fixture_tree` holds both halves of the
  point itself. It tests `std::thread::panicking()` before anything it raises,
  writes the error with `eprintln!` while unwinding, and still raises on the
  passing path.

Strings and comments are blanked before anything is matched, so a brace or a
word inside a message changes nothing. A Drop that removes no tree, such as a
child process's or a mutex's, is outside the point.

The tree test landed as `expectedFailure`, one commit before the fix, because
the pre-commit hook runs this suite. It named the 24 fixture Drops that raise
while unwinding at that commit. The fix takes the decorator off.
"""

import re
import unittest
from dataclasses import dataclass
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
CRATES = REPO_ROOT / "crates"
SKIPPED_DIRS = {"target", "node_modules"}

HELPER = "remove_fixture_tree"
IMPL_DROP = re.compile(r"\bimpl\b[^{;]*?\bDrop\s+for\s+([A-Za-z_][\w:]*)[^{;]*\{")
HELPER_FN = re.compile(r"\bfn\s+" + HELPER + r"\b[^{;]*\{")
REMOVES_TREE = re.compile(r"\bremove_dir_all\b|\b" + HELPER + r"\s*\(")
RAISES = re.compile(
    r"\.\s*(?:unwrap|expect|unwrap_err|expect_err)\s*\("
    r"|\b(?:panic|(?:debug_)?assert(?:_eq|_ne)?|unreachable|todo|unimplemented)\s*!"
    r"|\bresume_unwind\s*\("
)
GUARD = re.compile(r"\bpanicking\s*\(\s*\)")
REPORTS = re.compile(r"\beprintln\s*!")
FIX = (
    "a fixture Drop raises its removal failure while unwinding, which aborts the "
    "whole test binary (§AR-ci-fixture-cleanup.1, agent-grounds/grund#1226): call "
    f"`{HELPER}` from the CLI tests' support, or test `std::thread::panicking()` "
    "before raising"
)


def blank(source):
    """`source` with every comment and string or char literal's text replaced by
    spaces, newlines kept, so offsets and line numbers still match."""
    out = list(source)
    n = len(source)
    i = 0

    def clear(start, end):
        for k in range(start, min(end, n)):
            if out[k] != "\n":
                out[k] = " "

    while i < n:
        c = source[i]
        if source.startswith("//", i):
            end = source.find("\n", i)
            end = n if end < 0 else end
            clear(i, end)
            i = end
        elif source.startswith("/*", i):
            depth, j = 1, i + 2
            while j < n and depth:
                if source.startswith("/*", j):
                    depth, j = depth + 1, j + 2
                elif source.startswith("*/", j):
                    depth, j = depth - 1, j + 2
                else:
                    j += 1
            clear(i, j)
            i = j
        elif c in "rb" and (i == 0 or not (source[i - 1].isalnum() or source[i - 1] == "_")):
            m = re.compile(r'b?r(#*)"').match(source, i)
            if m:
                close = '"' + m.group(1)
                end = source.find(close, m.end())
                end = n if end < 0 else end + len(close)
                clear(m.end(), end - len(close))
                i = end
            else:
                i += 1  # a `b"..."` byte string is read as a string on the next step
        elif c == '"':
            j = i + 1
            while j < n and source[j] != '"':
                j += 2 if source[j] == "\\" else 1
            clear(i + 1, j)
            i = j + 1
        elif c == "'":
            if source.startswith("\\", i + 1):
                end = source.find("'", i + 2)
                end = n if end < 0 else end
                clear(i + 1, end)
                i = end + 1
            elif i + 2 < n and source[i + 2] == "'":
                clear(i + 1, i + 2)
                i += 3
            else:
                i += 1
        else:
            i += 1
    return "".join(out)


def block_end(code, open_brace):
    """The offset just past the brace that closes the one at `open_brace`."""
    depth, i = 0, open_brace
    while i < len(code):
        depth += {"{": 1, "}": -1}.get(code[i], 0)
        i += 1
        if depth == 0:
            return i
    return len(code)


def raises_unguarded(body):
    """Whether `body` can raise before any test of `std::thread::panicking()`."""
    first_raise = RAISES.search(body)
    if first_raise is None:
        return False
    guard = GUARD.search(body)
    return guard is None or first_raise.start() < guard.start()


@dataclass(frozen=True)
class Site:
    file: str
    line: int
    name: str
    problem: str

    def __str__(self):
        return f"{self.file}:{self.line} {self.name}: {self.problem}"


def violations_in(source, file="<source>"):
    """The tree-removing Drops and helper definitions in `source` that break §AR-ci-fixture-cleanup."""
    code = blank(source)
    found = []
    for match in IMPL_DROP.finditer(code):
        body = code[match.end() - 1 : block_end(code, match.end() - 1)]
        if REMOVES_TREE.search(body) and raises_unguarded(body):
            line = code.count("\n", 0, match.start()) + 1
            found.append(Site(file, line, f"impl Drop for {match.group(1)}",
                              "raises its removal failure while unwinding"))
    for match in HELPER_FN.finditer(code):
        body = code[match.end() - 1 : block_end(code, match.end() - 1)]
        line = code.count("\n", 0, match.start()) + 1
        if GUARD.search(body) is None or raises_unguarded(body):
            problem = "raises before testing std::thread::panicking()"
        elif REPORTS.search(body) is None:
            problem = "writes nothing to stderr while unwinding"
        elif RAISES.search(body) is None:
            problem = "never raises, so the passing path no longer fails"
        else:
            continue
        found.append(Site(file, line, f"fn {HELPER}", problem))
    return found


def rust_sources(root=CRATES):
    for path in sorted(root.rglob("*.rs")):
        if not SKIPPED_DIRS.intersection(path.relative_to(root).parts):
            yield path


def tree_violations():
    found = []
    for path in rust_sources():
        rel = path.relative_to(REPO_ROOT).as_posix()
        found.extend(violations_in(path.read_text(encoding="utf-8"), rel))
    return found


class TreeTest(unittest.TestCase):
    maxDiff = None

    # §AR-ci-fixture-cleanup.1: red until #1226 guards every copy; the fix removes this.
    @unittest.expectedFailure
    def test_no_tree_removing_fixture_drop_raises_while_unwinding(self):
        found = tree_violations()
        self.assertEqual([], [str(site) for site in found], f"{len(found)} sites: {FIX}")

    def test_the_scan_reads_the_already_guarded_copy(self):
        source = (CRATES / "grund-cli/tests/support/case_copy.rs").read_text(encoding="utf-8")
        code = blank(source)
        drops = [m.group(1) for m in IMPL_DROP.finditer(code)]
        self.assertIn("CaseCopy", drops, "the scan finds the fixture Drops it holds")
        self.assertEqual([], violations_in(source), "the guarded shape is accepted")


def drop_of(body, name="Fixture"):
    return f"struct {name}(PathBuf);\nimpl Drop for {name} {{\n    fn drop(&mut self) {{\n{body}\n    }}\n}}\n"


class ShapeTest(unittest.TestCase):
    """The check names what aborts and accepts what cannot, read from source text."""

    def assertFlags(self, source, count=1):
        self.assertEqual(count, len(violations_in(source)), violations_in(source))

    def test_a_removal_that_unwraps_or_expects_is_flagged(self):
        self.assertFlags(drop_of("        fs::remove_dir_all(&self.0).unwrap();"))
        self.assertFlags(drop_of('        fs::remove_dir_all(&self.0).expect("remove fixture");'))
        self.assertFlags(drop_of("        fs::remove_dir_all(&self.root).expect(\n"
                                 '            "remove isolated fixture",\n        );'))

    def test_a_removal_that_raises_nothing_is_accepted(self):
        self.assertFlags(drop_of("        let _ = fs::remove_dir_all(&self.0);"), 0)

    def test_the_helper_call_is_accepted(self):
        self.assertFlags(drop_of('        remove_fixture_tree(&self.0, "configured fixture");'), 0)

    def test_the_guarded_shape_is_accepted(self):
        self.assertFlags(drop_of(
            "        if let Err(err) = fs::remove_dir_all(&self.scratch)\n"
            "            && err.kind() != std::io::ErrorKind::NotFound\n        {\n"
            '            let message = format!("remove {}: {err}", self.scratch.display());\n'
            "            if std::thread::panicking() {\n"
            '                eprintln!("{message}");\n'
            "            } else {\n"
            '                panic!("{message}");\n'
            "            }\n        }"), 0)

    def test_an_early_return_while_panicking_is_accepted(self):
        self.assertFlags(drop_of(
            "        if std::thread::panicking() {\n"
            "            let _ = fs::remove_dir_all(&self.0);\n            return;\n        }\n"
            "        fs::remove_dir_all(&self.0).unwrap();"), 0)

    def test_raising_before_the_guard_is_flagged(self):
        self.assertFlags(drop_of(
            "        fs::remove_dir_all(&self.0).unwrap();\n"
            "        if std::thread::panicking() {\n            eprintln!(\"late\");\n        }"))

    def test_a_drop_that_removes_no_tree_is_outside_the_point(self):
        self.assertFlags(drop_of("        *CONTROL.lock().unwrap() = None;", "Reset"), 0)
        self.assertFlags(drop_of(
            "        self.child.wait().expect(\"host exit\");", "Host"), 0)

    def test_strings_and_comments_change_nothing(self):
        self.assertFlags(drop_of(
            '        // a note that says .unwrap() and panic! and {\n'
            '        let _ = fs::remove_dir_all(&self.0); /* } .expect( */\n'
            "        let _brace = '{';\n"
            '        let _ = r#"}" .unwrap()"#;'), 0)
        self.assertFlags(drop_of(
            '        let _ = format!("{{}} if panicking() {{");\n'
            "        fs::remove_dir_all(&self.0).unwrap();"))

    def test_every_drop_in_a_file_is_read(self):
        two = drop_of("        fs::remove_dir_all(&self.0).unwrap();", "One") + drop_of(
            "        let _ = fs::remove_dir_all(&self.0);", "Two") + drop_of(
            '        fs::remove_dir_all(&self.0).expect("x");', "Three")
        self.assertEqual(["impl Drop for One", "impl Drop for Three"],
                         [site.name for site in violations_in(two)])

    def test_the_helper_must_hold_both_halves(self):
        helper = ("pub fn remove_fixture_tree(path: &Path, what: &str) {{\n"
                  "    if let Err(error) = fs::remove_dir_all(path) {{\n{}\n    }}\n}}\n")
        good = ("        if std::thread::panicking() {\n"
                '            eprintln!("remove {what} {}: {error}", path.display());\n'
                "        } else {\n"
                '            panic!("remove {what} {}: {error}", path.display());\n        }')
        self.assertFlags(helper.format(good), 0)
        unguarded = '        panic!("remove {what}: {error}");'
        silent = ("        if !std::thread::panicking() {\n"
                  '            panic!("remove {what}: {error}");\n        }')
        lenient = ("        if std::thread::panicking() {\n"
                   '            eprintln!("remove {what}: {error}");\n        }')
        for body in (unguarded, silent, lenient):
            with self.subTest(body=body):
                self.assertFlags(helper.format(body))


if __name__ == "__main__":
    unittest.main()
