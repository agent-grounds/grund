"""§AR-system.principle, held as the primitive guards of §AR-system.5: no second
derivation of a fact a component owns. Across every production file of
`grund-core`, each primitive below is forbidden outside its owners: reading the
disk and loading a configuration outside the sources and config components, the
scanner's line predicates outside the scanner and the grammar, and splitting an
ID or a kind on `/` outside the alias splitter. This generalizes the checker-only
`checker/tests_post_scan_readers.rs` to the crate.

What uses a primitive outside its owners today is listed in
`primitive_allowlist.toml`, one row per file with the primitives it uses and the
step of #576 that removes them. The allowlist only shrinks: a use it does not
list fails, and so does a listed use that is gone."""

import re
import tomllib
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CRATE = REPO_ROOT / "crates" / "grund-core" / "src"
ALLOWLIST = Path(__file__).resolve().parent / "primitive_allowlist.toml"
STEP = re.compile(r"^#(59[5-9]|600)$")

# primitive -> (the pattern that finds a use of it, the directories or files that own it)
PRIMITIVES = {
    "std::fs": (r"\bstd::fs\b", ("config/", "sources/")),
    "input_read_": (r"\binput_read_\w+\s*\(", ("config/", "sources/")),
    ".exists()": (r"\.exists\(\)", ("config/", "sources/")),
    ".is_file()": (r"\.is_file\(\)", ("config/", "sources/")),
    "canonicalize": (r"\bcanonicalize\b", ("config/", "sources/")),
    "load_config": (r"\bload_config(?:_at\w*)?\s*\(", ("config/",)),
    "markdown_fence_delimiter": (r"\bmarkdown_fence_delimiter\b", ("scanner/", "grammar/")),
    "declaration_id_on_line": (r"\bdeclaration_id_on_line\b", ("scanner/", "grammar/")),
    "source_scan_line": (r"\bsource_scan_line\b", ("scanner/", "grammar/")),
    "section_re": (r"\bsection_re\b", ("scanner/", "grammar/")),
    "citation_re": (r"\bcitation_re\b", ("scanner/", "grammar/")),
    "split on /": (r"\b(?:r?split_once|r?splitn?|split_terminator)\(\s*(?:'/'|\"/\")",
                   ("workspace/id_arg.rs",)),
}


def is_production(relative):
    name = relative.rsplit("/", 1)[-1]
    return relative.endswith(".rs") and not name.startswith("tests_") and relative != "testing.rs"


def code_lines(text):
    """The file's lines with `//` comments taken off, so a doc-comment naming a
    primitive is not a use of it."""
    for line in text.splitlines():
        stripped = line.lstrip()
        if stripped.startswith("//"):
            yield ""
            continue
        yield line.split(" // ", 1)[0]


def uses(files):
    """Every `(file, primitive)` used outside the primitive's owners, `files` being
    `(path relative to the crate's src, text)`."""
    found = set()
    for relative, text in files:
        for primitive, (pattern, owners) in PRIMITIVES.items():
            if any(relative == owner or relative.startswith(owner) for owner in owners):
                continue
            if any(re.search(pattern, line) for line in code_lines(text)):
                found.add((relative, primitive))
    return found


def problems(found, allowlist):
    """What is wrong with the allowlist against the uses found, both ways round."""
    wrong = []
    allowed = set()
    for row in allowlist:
        if not STEP.match(row.get("step", "")):
            wrong.append(f"{row['file']}: the row names no step of #576 (#595-#600)")
        for primitive in row["primitives"]:
            if primitive not in PRIMITIVES:
                wrong.append(f"{row['file']}: `{primitive}` is no guarded primitive")
            allowed.add((row["file"], primitive))
    for file, primitive in sorted(found - allowed):
        owners = ", ".join(PRIMITIVES[primitive][1])
        wrong.append(f"{file} uses `{primitive}`, which only {owners} may: read the owner's "
                     "answer instead")
    for file, primitive in sorted(allowed - found):
        wrong.append(f"{file} no longer uses `{primitive}`: take it off the allowlist")
    return wrong


def crate_files():
    return [(path.relative_to(CRATE).as_posix(), path.read_text(encoding="utf-8"))
            for path in sorted(CRATE.rglob("*.rs"))
            if is_production(path.relative_to(CRATE).as_posix())]


class PrimitiveGuardTests(unittest.TestCase):
    def test_no_primitive_is_used_outside_its_owners_but_as_allowed(self):
        allowlist = tomllib.loads(ALLOWLIST.read_text(encoding="utf-8")).get("allow", [])
        wrong = problems(uses(crate_files()), allowlist)
        self.assertEqual([], wrong, "\n" + "\n".join(wrong))


class PrimitiveGuardHarnessTests(unittest.TestCase):
    """The guard fails on a new use and on an allowlist row that no longer applies."""

    ROW = {"file": "api/a.rs", "primitives": ["std::fs"], "step": "#595"}

    def test_an_allowed_use_passes(self):
        found = uses([("api/a.rs", "use std::fs;\n")])
        self.assertEqual([], problems(found, [self.ROW]))

    def test_an_owner_is_never_a_use(self):
        found = uses([("config/inputs.rs", "std::fs::read(p)\n"),
                      ("scanner/x.rs", "citation_re()\n"), ("grammar/y.rs", "section_re()\n"),
                      ("workspace/id_arg.rs", "id.split_once('/')\n")])
        self.assertEqual(set(), found)

    def test_a_new_use_fails(self):
        found = uses([("api/a.rs", "use std::fs;\n"),
                      ("model/b.rs", "let x = citation_re();\n"),
                      ("queries/c.rs", "let (alias, id) = arg.split_once('/')?;\n")])
        wrong = " ".join(problems(found, [self.ROW]))
        self.assertIn("model/b.rs uses `citation_re`", wrong)
        self.assertIn("queries/c.rs uses `split on /`", wrong)

    def test_a_use_that_is_gone_fails(self):
        self.assertIn("no longer uses", " ".join(problems(set(), [self.ROW])))

    def test_a_row_without_its_step_fails(self):
        row = dict(self.ROW, step="")
        self.assertIn("names no step", " ".join(problems({("api/a.rs", "std::fs")}, [row])))

    def test_a_comment_naming_a_primitive_is_not_a_use(self):
        found = uses([("api/a.rs", "/// Reads through `std::fs` once.\nfn f() {} // citation_re\n")])
        self.assertEqual(set(), found)


if __name__ == "__main__":
    unittest.main()
