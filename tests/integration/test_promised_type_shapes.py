"""§FS-distribution.3.1.2, §FS-distribution.3.1.3 — what an embedder's code meets
at compile time, read where §AR-bindings.2.1 puts it: every promised struct and
enum is `#[non_exhaustive]` at its definition and nothing else in grund-core is,
and the crate root shows each shape an embedder loses failing to compile beside a
twin that compiles. The promised types are the inventory's `frozen at 1.0` rows,
read through `test_public_surface_inventory`."""

import difflib
import os
import re
import subprocess
import unittest

from test_public_surface_inventory import (
    ATTRIBUTED_ITEM, FROZEN, LIB_RS, REPO_ROOT, _core_sources, _sample, inventory_dispositions,
)


# §FS-distribution.3.1.2: the structs and enums among the frozen rows.
PROMISED_STRUCTS, PROMISED_ENUMS = 79, 19
NON_EXHAUSTIVE = "#[non_exhaustive]"
# §FS-distribution.3.1.2, §FS-distribution.3.1.3: one `compile_fail` probe per
# shape an embedder loses, each naming the error rustc stops it at.
PROBE_CODES = ("E0425", "E0599", "E0639", "E0004")
FENCE = re.compile(r"^```(.*)$")
DOCTEST_NAME = re.compile(r"lib\.rs - \(line (\d+)\)")
COMPILER_ERROR = re.compile(r"error\[(E\d{4})\]")
ATTRIBUTED_TYPE = re.compile(
    ATTRIBUTED_ITEM.pattern.replace(r"(?:const\s+fn|fn|struct|enum|type|const|static)", r"(struct|enum)"),
    re.MULTILINE,
)


def type_definitions():
    """`name -> [(site, struct|enum, attribute span)]` for every `pub` struct and
    enum grund-core defines, so a promised type is read where it is declared."""
    found = {}
    for path, text in _core_sources():
        for match in ATTRIBUTED_TYPE.finditer(text):
            site = f"{path.relative_to(REPO_ROOT).as_posix()}:{text.count(chr(10), 0, match.start(3)) + 1}"
            found.setdefault(match.group(3), []).append((site, match.group(2), match.group(1)))
    return found


def stray_non_exhaustive(promised):
    """Every `#[non_exhaustive]` outside the attributes of a promised definition:
    on another type, a variant or anything else (§FS-distribution.3.1.2)."""
    stray = []
    for path, text in _core_sources():
        owned = [
            match.span(1)
            for match in ATTRIBUTED_TYPE.finditer(text)
            if match.group(3) in promised and NON_EXHAUSTIVE in match.group(1)
        ]
        for match in re.finditer(re.escape(NON_EXHAUSTIVE), text):
            if not any(start <= match.start() < end for start, end in owned):
                stray.append(f"{path.relative_to(REPO_ROOT).as_posix()}:{text.count(chr(10), 0, match.start()) + 1}")
    return stray


def crate_doc_blocks():
    """`(line, info, body)` for each fenced block of the crate documentation, the
    `//!` lines of `lib.rs`; `line` is the fence's, which is how rustdoc names it."""
    blocks, opened, info, body = [], None, "", []
    for number, line in enumerate(LIB_RS.read_text(encoding="utf-8").splitlines(), start=1):
        if not line.startswith("//!"):
            continue
        text = line[3:].removeprefix(" ")
        fence = FENCE.match(text)
        if fence and opened is None:
            opened, info, body = number, fence.group(1), []
        elif fence:
            blocks.append((opened, info, body))
            opened = None
        elif opened is not None:
            body.append(text)
    return blocks


def one_line_apart(probe, twin):
    """Whether `twin` is `probe` with exactly one line changed or one line added."""
    edits = [op for op in difflib.SequenceMatcher(a=probe, b=twin, autojunk=False).get_opcodes() if op[0] != "equal"]
    if len(edits) != 1:
        return False
    tag, start, end, twin_start, twin_end = edits[0]
    added = twin_end - twin_start
    return (tag == "replace" and end - start == 1 and added == 1) or (tag == "insert" and added == 1)


def run_crate_doctests():
    """`cargo test --doc` over grund-core, one test at a time and uncaptured, so
    each error a `compile_fail` block stopped at is printed after its name. Stable
    rustdoc does not compare a fence's error code, so the caller does."""
    return subprocess.run(
        [os.environ.get("CARGO", "cargo"), "test", "--doc", "-p", "grund-core", "--locked",
         "--", "--nocapture", "--test-threads=1"],
        cwd=REPO_ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
        text=True, encoding="utf-8", errors="replace",
    )


def errors_by_doctest(output):
    """`fence line -> the error codes printed between its name and the next one`."""
    parts = DOCTEST_NAME.split(output)
    found = {}
    for line, segment in zip(parts[1::2], parts[2::2]):
        found.setdefault(int(line), set()).update(COMPILER_ERROR.findall(segment.split("test result:")[0]))
    return found


class PromisedTypesAreNonExhaustiveTests(unittest.TestCase):
    """§FS-distribution.3.1.2: every promised struct and enum is
    `#[non_exhaustive]` where it is defined, and nothing else in grund-core is."""

    @classmethod
    def setUpClass(cls):
        frozen = {n for n, d in inventory_dispositions().items() if d == FROZEN}
        cls.definitions = {n: sites for n, sites in type_definitions().items() if n in frozen}

    def test_the_promised_types_are_the_77_structs_and_18_enums_the_frozen_rows_name(self):
        twice = {n for n, sites in self.definitions.items() if len(sites) != 1}
        self.assertEqual("", _sample(twice), "promised types defined more than once")
        kinds = [sites[0][1] for sites in self.definitions.values()]
        self.assertEqual((PROMISED_STRUCTS, PROMISED_ENUMS), (kinds.count("struct"), kinds.count("enum")))

    @unittest.expectedFailure  # red until #808 marks the promised types; implement removes this marker
    def test_every_promised_struct_and_enum_is_non_exhaustive(self):
        unmarked = {
            n for n, sites in self.definitions.items() if any(NON_EXHAUSTIVE not in s[2] for s in sites)
        }
        self.assertEqual("", _sample(unmarked), f"promised types without {NON_EXHAUSTIVE}")

    def test_no_other_type_and_no_variant_is_non_exhaustive(self):
        stray = stray_non_exhaustive(set(self.definitions))
        self.assertEqual("", _sample(stray), f"{NON_EXHAUSTIVE} outside a promised definition")


class RemovalProbeTests(unittest.TestCase):
    """§FS-distribution.3.1.2, §FS-distribution.3.1.3: the crate root shows each
    shape an embedder loses failing to compile, beside a twin that compiles."""

    def test_the_crate_root_carries_one_probe_per_shape_each_beside_its_twin(self):
        blocks = crate_doc_blocks()
        probes = [i for i, (_, info, _) in enumerate(blocks) if "compile_fail" in info]
        codes = [blocks[i][1].split(",")[-1].strip() for i in probes]
        self.assertEqual(sorted(PROBE_CODES), sorted(codes), "one `compile_fail,<code>` block per shape")
        for i in probes:
            line, _, probe = blocks[i]
            with self.subTest(line=line):
                self.assertLess(i + 1, len(blocks), "a probe with no twin after it")
                _, twin_info, twin = blocks[i + 1]
                self.assertNotIn("compile_fail", twin_info)
                self.assertTrue(one_line_apart(probe, twin), "the twin differs in more than one line")

    @unittest.expectedFailure  # red until #808 removes the names and marks the types; implement removes this marker
    def test_each_probe_stops_at_its_code_and_each_twin_passes(self):
        run = run_crate_doctests()
        self.assertEqual(0, run.returncode, run.stdout[-4000:])
        errors = errors_by_doctest(run.stdout)
        for line, info, _ in crate_doc_blocks():
            with self.subTest(line=line):
                self.assertIn(line, errors, "the block did not run as a doctest")
                wanted = {info.split(",")[-1].strip()} if "compile_fail" in info else set()
                self.assertEqual(wanted, errors[line])


if __name__ == "__main__":
    unittest.main()
