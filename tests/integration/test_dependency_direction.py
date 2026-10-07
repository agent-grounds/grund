"""§AR-system.4 — no component of `grund-core` reads one above it. Every
`crate::<other>` reference in every `.rs` file under
`crates/grund-core/src/<component>/` is judged against the order §AR-system.1
draws: model, grammar, config, then workspace and templates as siblings that may
not read each other, scanner, resolver, rules, checker, then queries and writers
as siblings that may not read each other either, then api. The ledger of reads
that still run the other way is **empty**, and must stay so. The mechanism is
kept rather than deleted — a future read against the direction is recorded here
with its note, or it is not made at all — and the list can only shrink.

A component's test modules — `tests_*.rs` beside the code they pin, and the
shared fixtures in `testing.rs` — are skipped: a test module may read any
component, because what it exercises is reached from wherever the run that
produced it starts (§AR-core-module-layout.1.3)."""

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CORE = REPO_ROOT / "crates" / "grund-core" / "src"

# §AR-system.1 as a rank: a component may read a lower rank and nothing else.

# Two pairs share a rank because they are siblings that may not read each other:
# `workspace` and `templates` — one answers about config text, the other renders
# from config, and neither is about the other — and `queries` and `writers`.

ORDER = {
    "model": 0,
    "grammar": 1,
    "config": 2,
    "workspace": 3,
    "templates": 3,
    "scanner": 4,
    "resolver": 5,
    "rules": 6,
    "checker": 7,
    "queries": 8,
    "writers": 8,
    "api": 9,
}

# `crate::<component>::<item>` or `crate::<component>::{<item>, …}`, in a `use`
# line or in a path written out where it is called.
REFERENCE = re.compile(r"\bcrate::([a-z_][a-z0-9_]*)::(\{[^}]*\}|[A-Za-z0-9_]+)")
NOTE = "§AR-system.4"

# The ledger of reads against the direction of §AR-system.1, and it is empty.

# Six former compat reads printed four `[workspace]` findings. Three are now a
# `Diagnostic` in the run's warning channel (§FS-check.3.29.15, §FS-check.4.10.11,
# §FS-workspace.6.1.7), rendered by its frontend (§AR-system.2.9.1).

# The fourth, the absorbed scan, became the config error of §FS-check.3.30.

# An entry is (file, `<component>::<item>`) and buys nothing else: the file must
# still make the read, and the import must still carry its §AR-system.4 note.
# Empty is the state this dict is meant to stay in.
RECORDED_DEBT = {}

# §AR-config.5: the components still naming the `Config` façade outside their
# tests. Held like the ledger above: a component off the list may not name it,
# and one that stops naming it leaves the list, so the list only shrinks.
CONFIG_FACADE = {
    "api",
    "checker",
    "config",
    "queries",
    "resolver",
    "scanner",
    "workspace",
    "writers",
}

# The façade by name: the type, not `KindConfig` or a word inside a longer one.
CONFIG_NAME = re.compile(r"\bConfig\b")
# A binding that holds a `Config`: a parameter or local typed as one, or a local
# cloned from a receiver whose name says it is a config.
CONFIG_BINDING = re.compile(
    r"\b(?:mut\s+)?([a-z_][a-z0-9_]*)\s*:\s*(?:&\s*(?:'[a-z_]+\s+)?(?:mut\s+)?)?Config\b"
    r"|\blet\s+mut\s+([a-z_][a-z0-9_]*)\s*=\s*[a-z_.]*config[a-z_]*\.clone\(\)"
)
# `<receiver>.<field> = …`, `+=`, `-=`, or a mutating call on the field.
FIELD_WRITE = re.compile(
    r"\b((?:[a-z_][a-z0-9_]*\.)*[a-z_][a-z0-9_]*)\.([a-z_][a-z0-9_]*)"
    r"(?:\s*(?:=(?![=>])|\+=|-=)|\.(?:push|extend|retain|clear|insert|truncate|sort|dedup)\()"
)


def _components():
    return sorted(path.name for path in CORE.iterdir() if path.is_dir())


def _is_test_module(path):
    """Whether `path` is a test module rather than implementation.

    A test module may read any component, so the order says nothing about it.
    """
    return path.name.startswith("tests_") or path.name == "testing.rs"


def _references():
    """Every `crate::<other>::<item>` a component file makes, as
    (file, `<other>::<item>`) -> the line it is written on."""
    found = {}
    for component in _components():
        for path in sorted((CORE / component).glob("**/*.rs")):
            if _is_test_module(path):
                continue
            text = path.read_text(encoding="utf-8")
            relative = path.relative_to(CORE).as_posix()
            for match in REFERENCE.finditer(text):
                other = match.group(1)
                if other not in ORDER or other == component:
                    continue
                items = match.group(2)
                if items.startswith("{"):
                    names = [name.strip() for name in items.strip("{}").split(",")]
                else:
                    names = [items]
                line = text.count("\n", 0, match.start()) + 1
                for name in names:
                    if name:
                        found.setdefault((relative, f"{other}::{name}"), line)
    return found


def _code(path):
    """A file without its `//` comment lines, which name `Config` in prose."""
    lines = path.read_text(encoding="utf-8").splitlines()
    return "\n".join("" if line.lstrip().startswith("//") else line for line in lines)


def _implementation_files(component):
    for path in sorted((CORE / component).glob("**/*.rs")):
        if not _is_test_module(path):
            yield path


def components_naming_config():
    """Every component whose implementation names the `Config` type."""
    return {
        component
        for component in _components()
        if any(CONFIG_NAME.search(_code(path)) for path in _implementation_files(component))
    }


def config_fields():
    """The public fields of the `Config` façade, read from its declaration."""
    for path in sorted((CORE / "config").glob("**/*.rs")):
        match = re.search(r"^pub struct Config \{(.*?)^\}", _code(path), re.M | re.S)
        if match:
            return set(re.findall(r"^\s*pub(?:\([a-z]+\))?\s+([a-z_][a-z0-9_]*)\s*:", match.group(1), re.M))
    raise AssertionError("no `pub struct Config` under config/")


def config_writes(text, fields):
    """`(line, receiver.field)` for every write to a `Config` field in `text`.

    A receiver is a config when its last name says so (`config`,
    `root_config`, `entry.config`) or when the file binds that name to a
    `Config` (§AR-config.5). A heuristic, held to the cases below."""
    bound = {name for pair in CONFIG_BINDING.findall(text) for name in pair if name}
    writes = []
    for match in FIELD_WRITE.finditer(text):
        receiver, field = match.groups()
        last = receiver.split(".")[-1]
        if field in fields and ("config" in last or last in bound):
            writes.append((text.count("\n", 0, match.start()) + 1, f"{receiver}.{field}"))
    return writes


def _reads_downward(component, other):
    return ORDER[other] < ORDER[component]


def _recorded():
    return {(file, item) for file, items in RECORDED_DEBT.items() for item in items}


def _note_is_above(path, line):
    """Whether the import on `line` carries its §AR-system.4 note.

    The note sits in the three lines above the `use` statement, or above the run
    of imports that statement belongs to — one note may mark a group the
    compiler forces apart into one line per component, so the search walks up
    over the neighbouring import lines and stops at the first comment or blank.
    """
    lines = (CORE / path).read_text(encoding="utf-8").splitlines()
    start = line - 1
    while start > 0:
        above = lines[start - 1].strip()
        if not above or above.startswith("//"):
            break
        start -= 1
    return any(NOTE in text for text in lines[max(0, start - 3):start])


class DependencyDirectionTests(unittest.TestCase):
    def test_every_component_directory_has_a_place_in_the_order(self):
        """A new directory under `src/` is a new component, and §AR-system.1 has
        to say where it sits before this test can judge anything it reads."""
        self.assertEqual([], [name for name in _components() if name not in ORDER])

    def test_no_component_reads_one_above_it(self):
        recorded = _recorded()
        upward = []
        for (file, item), line in sorted(_references().items()):
            component = file.split("/", 1)[0]
            if _reads_downward(component, item.split("::", 1)[0]):
                continue
            if (file, item) in recorded:
                continue
            upward.append(f"{file}:{line} reads {item}")
        self.assertEqual(
            [],
            upward,
            "reads against the direction of §AR-system.4 that RECORDED_DEBT does "
            "not list; resolve the read, or record it there with its note",
        )

    def test_every_recorded_read_is_still_made(self):
        """The list may only shrink: an entry whose read has gone is removed in
        the same change that removes the read."""
        references = _references()
        stale = sorted(entry for entry in _recorded() if entry not in references)
        self.assertEqual([], [f"{file} no longer reads {item}" for file, item in stale])

    def test_every_recorded_read_carries_its_note(self):
        """No read against the direction is silent at its site: the import says
        §AR-system.4, so a reader of the file sees the debt without this test."""
        references = _references()
        unmarked = []
        for file, item in sorted(_recorded()):
            line = references.get((file, item))
            if line is not None and not _note_is_above(file, line):
                unmarked.append(f"{file}:{line} ({item})")
        self.assertEqual([], unmarked, "recorded reads with no §AR-system.4 note above them")


class ConfigFacadeTests(unittest.TestCase):
    """§AR-config.5: the façade list only shrinks, and nothing writes the façade."""

    maxDiff = None

    def test_the_writer_detector_sees_the_shapes_it_must(self):
        fields = {"scan_full", "owner_lines", "require_grounding"}
        text = (
            "fn a(config: &mut Config) { config.scan_full = true; }\n"
            "fn b(c: &Config) { let mut walk = c.clone(); }\n"
            "fn c(root: &Config) { let mut alone = root_config.clone(); alone.owner_lines.clear(); }\n"
            "fn d(p: P) { p.config.require_grounding = true; }\n"
            "fn e(r: &mut Run) { r.scan_full = true; if x.scan_full == y {} }\n"
            "fn f(c: &Config) { match c { s if s.scan_full => 1 } }\n"
        )
        self.assertEqual(
            ["config.scan_full", "alone.owner_lines", "p.config.require_grounding"],
            [write for _, write in config_writes(text, fields)],
        )

    def test_no_component_off_the_list_names_config(self):
        self.assertEqual([], sorted(components_naming_config() - CONFIG_FACADE))

    def test_every_component_on_the_list_still_names_config(self):
        """Remove a component from CONFIG_FACADE in the change that moves it off."""
        self.assertEqual([], sorted(CONFIG_FACADE - components_naming_config()))

    def test_no_config_field_is_written_outside_config(self):
        """A per-run change is made to the `Run`, and the façade is rebuilt from it."""
        fields = config_fields()
        writes = []
        for component in _components():
            if component == "config":
                continue
            for path in _implementation_files(component):
                relative = path.relative_to(CORE).as_posix()
                writes.extend(f"{relative}:{line} writes {write}" for line, write in config_writes(_code(path), fields))
        self.assertEqual([], writes)


if __name__ == "__main__":
    unittest.main()
