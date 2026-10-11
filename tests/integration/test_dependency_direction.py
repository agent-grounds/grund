"""§AR-system.4 — no component of `grund-core` reads one above it. Every
`crate::<other>` reference in every `.rs` file under
`crates/grund-core/src/<component>/` is judged against the order §AR-system.1
draws: model, grammar, config, then workspace and templates as siblings that may
not read each other, scanner, resolver, rules, checker, then queries and writers
as siblings that may not read each other either, then api. The ledger of reads
that still run the other way is **empty**, and must stay so. The mechanism is
kept rather than deleted — a future read against the direction is recorded here
with its note, or it is not made at all — and the list can only shrink.

A read through a root re-export is the read it is: `crate::Finding`, or a name
in `use crate::{…}`, that `lib.rs` re-exports from a component is judged as a
read of that component (§AR-system.5), so the crate root is no way around the
order.

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
# `pub use <component>::{…};` or `pub type <Name> = <component>::…;` in `lib.rs`.
ROOT_USE = re.compile(r"^pub use ([a-z_]+)::(\{[^}]*\}|[A-Za-z0-9_]+);", re.M | re.S)
ROOT_TYPE = re.compile(r"^pub type ([A-Za-z0-9_]+)\s*=\s*([a-z_]+)::", re.M)
# `crate::<Name>` or `crate::{<Name>, …}`: a read through the crate root.
ROOT_READ = re.compile(r"\bcrate::(\{[^}]*\}|[A-Za-z_][A-Za-z0-9_]*)(?!\w|\s*::)")

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
    "config",
    "queries",
    "workspace",
    "writers",
}


# §AR-system.4's concern ledger: `Rules` in no scanner or node-local checker file
# (§AR-checker.1.1); `Presentation` in no checker file, and in the scanner and the
# resolver only in the two files handed it explicitly.
RULES_NAME = re.compile(r"\bRules\b")
PRESENTATION_NAME = re.compile(r"\bPresentation\b")
CONFORM_FILES = (
    "checker/inline_style.rs",
    "checker/near_miss.rs",
    "checker/sections.rs",
    "checker/sizes.rs",
)
PRESENTATION_HANDED = ("resolver/link_targets.rs", "scanner/agent_entrypoints.rs")

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


def root_re_exports(text):
    """Each name `lib.rs` re-exports from a component, mapped to that component."""
    exported = {}
    for component, items in ROOT_USE.findall(text):
        for name in items.strip("{}").split(","):
            name = name.strip().split(" as ")[-1].strip()
            if name and component in ORDER:
                exported[name] = component
    for name, component in ROOT_TYPE.findall(text):
        if component in ORDER:
            exported[name] = component
    return exported


def root_reads(text, exported):
    """`(<component>::<Name>, line)` for every code read of a re-exported name
    through the crate root; a doc-comment link is not a read."""
    for number, line in enumerate(text.splitlines(), 1):
        if line.lstrip().startswith("//"):
            continue
        for match in ROOT_READ.finditer(line):
            for name in match.group(1).strip("{}").split(","):
                name = name.strip().split(" as ")[0].strip()
                if name in exported:
                    yield f"{exported[name]}::{name}", number


def _references():
    """Every `crate::<other>::<item>` a component file makes, as
    (file, `<other>::<item>`) -> the line it is written on."""
    found = {}
    exported = root_re_exports((CORE / "lib.rs").read_text(encoding="utf-8"))
    for component in _components():
        for path in sorted((CORE / component).glob("**/*.rs")):
            if _is_test_module(path):
                continue
            text = path.read_text(encoding="utf-8")
            relative = path.relative_to(CORE).as_posix()
            for item, line in root_reads(text, exported):
                if item.split("::", 1)[0] != component:
                    found.setdefault((relative, item), line)
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


class RootReExportTests(unittest.TestCase):
    """§AR-system.5: the crate root is no way around the order."""

    LIB = (
        "pub use model::{\n    Finding, Report,\n};\n"
        "pub use api::{ListEntry, list};\n"
        "pub type Config = config::Config;\n"
    )

    def test_the_root_re_exports_are_read_off_lib_rs(self):
        self.assertEqual(
            {"Finding": "model", "Report": "model", "ListEntry": "api", "list": "api",
             "Config": "config"},
            root_re_exports(self.LIB),
        )

    def test_a_read_through_the_root_is_the_read_it_is(self):
        text = (
            "use crate::{Finding, ListEntry};\n"
            "/// See [`ListEntry`](crate::ListEntry).\n"
            "fn f(x: crate::Config) -> crate::list::Thing {}\n"
        )
        self.assertEqual(
            [("model::Finding", 1), ("api::ListEntry", 1), ("config::Config", 3)],
            list(root_reads(text, root_re_exports(self.LIB))),
        )

    def test_the_tree_s_root_re_exports_are_seen(self):
        exported = root_re_exports((CORE / "lib.rs").read_text(encoding="utf-8"))
        self.assertEqual("model", exported.get("Finding"))
        self.assertEqual("api", exported.get("check"))


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

    def test_the_checker_resolver_and_scanner_name_no_config(self):
        """§AR-config.5: the split hands each of them records, not the façade."""
        self.assertEqual([], sorted(components_naming_config() & {"checker", "resolver", "scanner"}))

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


def _names(pattern, component, skip=()):
    """`file:line` for every code line of `component` naming `pattern`."""
    hits = []
    for path in _implementation_files(component):
        relative = path.relative_to(CORE).as_posix()
        if relative in skip:
            continue
        for number, line in enumerate(_code(path).splitlines(), 1):
            if pattern.search(line):
                hits.append(f"{relative}:{number}")
    return hits


def _signature(name):
    """The parameter list of `fn <name>(…)` under checker/, or None."""
    for path in _implementation_files("checker"):
        match = re.search(rf"\bfn {name}\s*(?:<[^>]*>)?\(([^)]*)\)", _code(path), re.S)
        if match:
            return match.group(1)
    return None


class ConcernLedgerTests(unittest.TestCase):
    """§AR-system.4: a stage that is not handed a concern cannot read it
    (§DA-config-concern-records.2.3)."""

    maxDiff = None

    def test_the_scanner_names_no_rules(self):
        self.assertEqual([], _names(RULES_NAME, "scanner"))

    def test_the_node_local_checks_name_no_rules(self):
        hits = []
        for file in CONFORM_FILES:
            path = CORE / file
            hits.extend(f"{file}:{n}" for n, line in enumerate(_code(path).splitlines(), 1) if RULES_NAME.search(line))
        self.assertEqual([], hits)

    def test_the_checker_names_no_presentation(self):
        self.assertEqual([], _names(PRESENTATION_NAME, "checker"))

    def test_presentation_is_named_below_the_writers_only_where_handed(self):
        hits = _names(PRESENTATION_NAME, "scanner", PRESENTATION_HANDED)
        hits += _names(PRESENTATION_NAME, "resolver", PRESENTATION_HANDED)
        self.assertEqual([], hits)

    def test_link_targets_and_surface_reach_are_handed_presentation(self):
        """§AR-resolver.6, §AR-scanner.7: each takes `&Presentation` explicitly."""
        missing = [file for file in PRESENTATION_HANDED if not PRESENTATION_NAME.search(_code(CORE / file))]
        self.assertEqual([], missing)

    def test_conform_and_judge_are_handed_their_concerns_only(self):
        """§AR-checker.1.1, §AR-checker.1.2: conform names no `Rules` and no
        `Presentation`; judge names `Rules`, `Schema` and `Expected`, and no
        `Presentation`."""
        conform, judge = _signature("conform"), _signature("judge")
        self.assertIsNotNone(conform, "no `fn conform(` under checker/")
        self.assertIsNotNone(judge, "no `fn judge(` under checker/")
        self.assertIn("Schema", conform)
        for name in ("Rules", "Presentation", "Config"):
            self.assertNotRegex(conform, rf"\b{name}\b")
        for name in ("Rules", "Schema", "Expected"):
            self.assertRegex(judge, rf"\b{name}\b")
        for name in ("Presentation", "Config"):
            self.assertNotRegex(judge, rf"\b{name}\b")


if __name__ == "__main__":
    unittest.main()
