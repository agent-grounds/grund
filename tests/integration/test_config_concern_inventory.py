"""§FS-config.concerns — every key of the config belongs to exactly one concern,
and the complete assignment is derived from the reader's parse sites rather than
written by hand. The three parse sites §DF-config-scope-override.2.2 enumerates
are the candidate set, and they are one per grammar because `[[kinds]]` and
`[citations]` read their own keys (§AR-core-module-layout.1): so the inventory is
complete exactly when its first column is that set — every key the reader accepts
has one row, no row names a key no parser accepts, no key is listed twice, and
every row carries exactly one classification from the closed set.

The classification is also held to the signatures that consume it
(§DA-config-concern-records.2.3, §AR-checker.1): which concern records the
checker's two halves and the scanner name is derived from their code and
compared with what each stage is handed, so a concern the inventory assigns
reaches a stage only as the record that stage takes.

A v2 file (§FS-config-v2) is spelled by concern, so its inventory is the
tables its reader's handlers read: each key's table names its concern, and the
same derivation holds every key of the v2 reader to exactly one of them.
"""

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CONFIG = REPO_ROOT / "crates" / "grund-core" / "src" / "config"


def _parse_site(name):
    """A parse site, in the v1 reader's directory once it has moved there
    (§AR-config.2), and beside discovery until then."""
    moved = CONFIG / "v1" / name
    return moved if moved.is_file() else CONFIG / name


PARSE = _parse_site("parse.rs")
KIND_TABLE = _parse_site("kind_table.rs")
CITATIONS = _parse_site("citations.rs")
INVENTORY = (
    REPO_ROOT / "docs" / "decisions" / "functional" / "DF-config-concerns.md"
)

# The closed set of classifications. Three concerns, and the envelope, which is
# what the concerns are read inside rather than a fourth one.
CLASSIFICATIONS = ("schema", "rules", "presentation", "envelope")

# `("<section>", "<key>")` and `("<section>", key @ ("<a>" | "<b>"))`, the two
# shapes the reader's section-and-key match is written in. The catch-all arms
# that delegate to the two grammars below carry no literal and match neither.
PAIR = re.compile(r'^\s*\("([a-z._]*)",\s*"([a-z_]+)"\)\s*=>', re.MULTILINE)
ALTERNATIVES = re.compile(
    r'^\s*\("([a-z._]*)",\s*key @ \(([^)]*)\)\)\s*=>', re.MULTILINE
)
LITERAL = re.compile(r'"([a-z_-]+)"')
# One `match key` arm of a grammar's own reader: at the function's own depth, so
# a match on a *value* nested inside an arm is not read as a key.
KIND_ARM = re.compile(r'^ {8}"([a-z_]+)" =>', re.MULTILINE)
CITATION_ARM = re.compile(r'^\s+"([a-z-]+)" =>', re.MULTILINE)
# A row of the inventory: a backticked key, then its classification.
ROW = re.compile(r"^\|\s*`([^`]+)`\s*\|\s*([a-z]*)\s*\|", re.MULTILINE)

CORE = REPO_ROOT / "crates" / "grund-core" / "src"
# The concern record each classification lowers into (§AR-config.1.2).
RECORD = {"schema": "Schema", "rules": "Rules", "presentation": "Presentation"}
# What each stage is handed (§DA-config-concern-records.2.3 as amended); presentation
# reaches judge only as `Expected`, and the scanner only at its entrypoint probe
# (§AR-scanner.7).
STAGE_CONCERNS = {
    "conform": {"schema"},
    "judge": {"schema", "rules"},
    "scanner": {"schema", "presentation"},
}

# Keys the reader recognizes only in order to refuse them: they are not keys of
# the format in force, so they are not the inventory's. Each is held to its
# refusal below, so the exclusion is checked rather than asserted.
REFUSED = {"[[kinds]] prefix": "was removed in"}

# One key per parse site and per shape, so a parser that stopped seeing its
# arms fails here rather than agreeing with an inventory that classifies
# nothing. `[id] format` and `[[kinds]] format` are both here on purpose: one
# name, two keys, and an inventory that carries one row for the two is wrong.
SPINE = (
    "grund_config_version",
    "[reference] marker",
    "[id] format",
    "[id] named_sections",
    "[[kinds]] require_grounding",
    "[[kinds]] folder",
    "[[kinds]] format",
    "[scan] exclude",
    "[output] color",
    "[fmt] exclude",
    "[fmt.cross_refs] enabled",
    "[workspace] members",
    "[citations] default",
    "[citations.<KIND>] must",
)


def _source(path):
    """The file without its comments, which quote key names in prose."""
    lines = path.read_text(encoding="utf-8").splitlines()
    return "\n".join("" if line.lstrip().startswith("//") else line for line in lines)


def _qualify(section, key):
    """How one arm's section and key are spelled as a row of the inventory."""
    if section == "":
        return key
    if section == "kinds":
        # An array of tables in the file, whichever reader answers for the key:
        # the section walk hands the two grounding keys to `grounding.rs` and
        # everything else to `kind_table.rs`, and both write the same row.
        return f"[[kinds]] {key}"
    return f"[{section}] {key}"


def project_keys():
    """Every key `parse.rs` accepts, by the table it is written under."""
    source = _source(PARSE)
    keys = [_qualify(section, key) for section, key in PAIR.findall(source)]
    for section, alternatives in ALTERNATIVES.findall(source):
        keys.extend(_qualify(section, key) for key in LITERAL.findall(alternatives))
    return keys


def _function(path, name):
    """One function's body: from its signature to the next item at column 0."""
    source = _source(path)
    start = source.index(f"fn {name}(")
    rest = source[start:]
    end = re.search(r"^\}", rest, re.MULTILINE)
    return rest[: end.end()]


def kind_row_keys():
    """Every key a `[[kinds]]` row accepts, from its own reader."""
    return [f"[[kinds]] {key}" for key in KIND_ARM.findall(_function(KIND_TABLE, "parse_kinds_key"))]


def citation_keys():
    """`[citations]`'s own key, and the keys of a `[citations.<KIND>]` table.

    The reader answers for the two tables in one function, splitting at the
    point it stops being about `[citations]`, so the split is read here from
    that same line rather than from the arms' indentation."""
    body = _function(CITATIONS, "parse_citation_entry")
    boundary = body.index('strip_prefix("citations.")')
    return [f"[citations] {key}" for key in CITATION_ARM.findall(body[:boundary])] + [
        f"[citations.<KIND>] {key}" for key in CITATION_ARM.findall(body[boundary:])
    ]


def reader_keys():
    """Every key of the format in force, as the three parse sites accept it."""
    keys = project_keys() + kind_row_keys() + citation_keys()
    return [key for key in keys if key not in REFUSED]


def inventory_rows():
    """The rows the record's inventory claims to have classified.

    An inventory that is absent classifies nothing, which is the same finding
    as one that stops short: the keys it does not carry are the ones missing."""
    if not INVENTORY.is_file():
        return []
    return ROW.findall(INVENTORY.read_text(encoding="utf-8"))


def _sample(names, limit=8):
    names = sorted(names)
    shown = ", ".join(names[:limit])
    return shown if len(names) <= limit else f"{shown}, … ({len(names)} in all)"


def _signature(component, name):
    """The parameter list of `fn <name>(…)` in `component`, or ''."""
    for path in sorted((CORE / component).glob("**/*.rs")):
        if path.name.startswith("tests_") or path.name == "testing.rs":
            continue
        match = re.search(rf"\bfn {name}\s*(?:<[^>]*>)?\(([^)]*)\)", _source(path), re.S)
        if match:
            return match.group(1)
    return ""


def _component_code(component):
    return "\n".join(
        _source(path)
        for path in sorted((CORE / component).glob("**/*.rs"))
        if not (path.name.startswith("tests_") or path.name == "testing.rs")
    )


def _concerns_named(text):
    return {concern for concern, record in RECORD.items() if re.search(rf"\b{record}\b", text)}


def stage_concerns():
    """Which concern records each stage names, read from its code."""
    return {
        "conform": _concerns_named(_signature("checker", "conform")),
        "judge": _concerns_named(_signature("checker", "judge")),
        "scanner": _concerns_named(_component_code("scanner")),
    }


class ConfigConcernInventoryTests(unittest.TestCase):
    def test_the_three_parse_sites_still_read_as_key_literals(self):
        """The guard on every comparison below: a parse that found nothing
        would agree with an inventory that classified nothing."""
        keys = set(reader_keys())
        missing = sorted(key for key in SPINE if key not in keys)
        self.assertEqual(
            [],
            missing,
            f"the config reader under {CONFIG.relative_to(REPO_ROOT)} no longer "
            f"spells these keys where this test reads them: {missing}",
        )

    def test_every_refused_key_is_still_refused_rather_than_read(self):
        """The exclusion list is a claim about the reader, so it is checked:
        a key dropped from the inventory must be one no config may write."""
        arms = _function(KIND_TABLE, "parse_kinds_key")
        for key, refusal in REFUSED.items():
            self.assertIn(
                refusal,
                arms,
                f"{key} is excluded from the inventory as a key the reader only "
                f"refuses, but its refusal is gone from {KIND_TABLE.name}",
            )

    def test_the_record_carries_an_inventory(self):
        """The rows are the classification: without them the comparisons below
        hold an empty set against an empty set and prove nothing."""
        self.assertNotEqual(
            [],
            inventory_rows(),
            f"{INVENTORY.relative_to(REPO_ROOT)} carries no key -> concern table, "
            "so no key is classified and the four checks below are vacuous",
        )

    def test_every_key_the_reader_accepts_has_an_inventory_row(self):
        missing = set(reader_keys()) - {key for key, _ in inventory_rows()}
        self.assertEqual(
            "",
            _sample(missing),
            f"config keys with no row in {INVENTORY.relative_to(REPO_ROOT)}",
        )

    def test_the_inventory_names_only_keys_the_reader_accepts(self):
        extra = {key for key, _ in inventory_rows()} - set(reader_keys())
        self.assertEqual(
            "",
            _sample(extra),
            "inventory rows naming what no parse site accepts",
        )

    def test_every_row_carries_exactly_one_of_the_three_concerns(self):
        wrong = {
            f"{key} → {classification or '(blank)'}"
            for key, classification in inventory_rows()
            if classification not in CLASSIFICATIONS
        }
        self.assertEqual(
            "",
            _sample(wrong),
            f"inventory rows whose classification is not one of {CLASSIFICATIONS}",
        )

    def test_no_key_is_listed_twice(self):
        counted = {}
        for key, _ in inventory_rows():
            counted[key] = counted.get(key, 0) + 1
        duplicated = {key for key, count in counted.items() if count > 1}
        self.assertEqual(
            "",
            _sample(duplicated),
            "inventory rows repeating one key, so one key has two concerns",
        )

    def test_each_stage_is_handed_the_concerns_its_signature_derives(self):
        """§AR-checker.1: the concerns each stage names are the ones it is
        handed, presentation reaches judge only as `Expected`, and every
        concern the inventory assigns reaches some stage."""
        self.assertEqual(STAGE_CONCERNS, stage_concerns())
        self.assertRegex(_signature("checker", "judge"), r"\bExpected\b")
        assigned = {c for _, c in inventory_rows() if c in RECORD}
        reached = set().union(*STAGE_CONCERNS.values())
        self.assertEqual(set(), assigned - reached)


# §FS-config-v2: each `config/v2/` handler and the table(s) it reads, as written.
# Its keys are read from the handler, so a table's first segment is the concern.
V2 = CONFIG / "v2"
V2_HANDLERS = {
    "envelope_key": ("", "arms"),
    "workspace_key": ("workspace", "arms"),
    "schema_key": ("schema", "arms"),
    "sources_key": ("schema.sources", "arms"),
    "row_key": ("schema.kinds.<NAME>", "arms"),
    "measure_key": ("schema.notes.lines|schema.notes.columns|schema.leads.words", "strength"),
    "text_key": ("schema.notes.text", "single"),
    "layout_key": ("schema.notes.layout", "strength"),
    "citations_key": ("rules.citations", "single"),
    "citation_kind_key": ("rules.citations.<KIND>", "arms"),
    "ladder_key": ("rules.citations.grounding|rules.citations.<NAME>.grounding", "strength"),
    "resolution_key": ("rules.resolution", "kind"),
    "presentation_key": ("presentation", "arms"),
    "kind_key": ("presentation.kinds.<KIND>", "single"),
    "fmt_key": ("presentation.fmt", "arms"),
}
# The envelope's tables (§FS-config.concerns): read inside the concerns, never one.
V2_ENVELOPE = ("", "workspace")
V2_HANDLER = re.compile(r"^( *)(?:pub\(super\) )?fn ([a-z_]+_key)\(", re.MULTILINE)
V2_SINGLE = re.compile(r'if key != "([a-z_-]+)"')
V2_SPINE = (
    "grund_config_version",
    "[workspace] members",
    "[schema] marker",
    "[schema.sources] languages",
    "[schema.kinds.<NAME>] folders",
    "[schema.notes.lines] warn",
    "[schema.notes.text] must",
    "[rules.citations] default",
    "[rules.citations.<KIND>] warn-not",
    "[rules.citations.grounding] must",
    "[rules.resolution] <KIND>",
    "[presentation.kinds.<KIND>] title",
    "[presentation.fmt] exclude",
)


def _v2_handlers():
    """Every `*_key` handler under `config/v2/`, with its body and indentation:
    `read_key`, which dispatches to them, reads no key of its own."""
    found = {}
    for path in sorted(V2.glob("*.rs")):
        source = _source(path)
        for match in V2_HANDLER.finditer(source):
            indent, name = match.groups()
            if name == "read_key":
                continue
            rest = source[match.start():]
            end = re.search(rf"^{indent}\}}", rest, re.MULTILINE)
            found[name] = (path, indent, rest[: end.end()])
    return found


def _strengths():
    """The positive strengths `Strength::parse` admits as a key."""
    body = _function(CONFIG / "project.rs", "parse")
    return re.findall(r'^\s*"([a-z]+)" => Some\(Self::', body, re.MULTILINE)


def v2_keys():
    """Every `(table, key)` the v2 reader admits, read from its handlers."""
    handlers = _v2_handlers()
    keys = []
    for name, (tables, mode) in V2_HANDLERS.items():
        _, indent, body = handlers[name]
        if mode == "arms":
            # The handler's own `match key` arms, one level inside its body.
            arm = re.compile(
                rf'^ {{{len(indent) + 8}}}("[a-z_-]+"(?: \| "[a-z_-]+")*) =>', re.MULTILINE
            )
            names = [key for arm_text in arm.findall(body) for key in LITERAL.findall(arm_text)]
        elif mode == "single":
            names = V2_SINGLE.findall(body)
        elif mode == "strength":
            names = _strengths() if "Strength::parse(key)" in body else []
        else:
            names = ["<KIND>"]
        keys.extend((table, key) for table in tables.split("|") for key in names)
    return keys


def _spell(table, key):
    return key if table == "" else f"[{table}] {key}"


class V2ConfigConcernInventoryTests(unittest.TestCase):
    """§FS-config-v2: every v2 key is written under the concern it belongs to."""

    def test_every_v2_handler_is_named_and_dispatched(self):
        handlers = _v2_handlers()
        self.assertEqual(sorted(V2_HANDLERS), sorted(handlers), "v2 `*_key` handlers")
        walk = _source(V2 / "walk.rs")
        body = walk[walk.index("fn read_key(") :]
        undispatched = [
            name for name in handlers if not re.search(rf"[:.]{name}\(", body)
        ]
        self.assertEqual([], undispatched, "handlers `read_key` never calls")

    def test_the_v2_handlers_still_read_as_key_literals(self):
        keys = {_spell(table, key) for table, key in v2_keys()}
        missing = [key for key in V2_SPINE if key not in keys]
        self.assertEqual([], missing, f"keys {V2.relative_to(REPO_ROOT)} no longer spells")

    def test_every_v2_key_sits_under_exactly_one_concern(self):
        wrong = {
            _spell(table, key)
            for table, key in v2_keys()
            if table not in V2_ENVELOPE and table.split(".")[0] not in CLASSIFICATIONS[:3]
        }
        self.assertEqual("", _sample(wrong), "v2 keys outside schema, rules and presentation")

    def test_no_v2_key_is_admitted_twice(self):
        counted = {}
        for table, key in v2_keys():
            counted[(table, key)] = counted.get((table, key), 0) + 1
        duplicated = {_spell(*pair) for pair, count in counted.items() if count > 1}
        self.assertEqual("", _sample(duplicated), "v2 keys two arms admit")

    def test_a_project_setting_is_written_again_on_the_kind(self):
        """§FS-config.principle: a narrower scope writes the same key, in its own table."""
        keys = {_spell(table, key) for table, key in v2_keys()}
        pairs = (
            ("[schema] id_format", "[schema.kinds.<NAME>] id_format"),
            ("[rules.citations] default", "[rules.citations.<KIND>] default"),
            ("[rules.citations.grounding] must", "[rules.citations.<NAME>.grounding] must"),
        )
        missing = [key for pair in pairs for key in pair if key not in keys]
        self.assertEqual([], missing, "a scope the v2 reader no longer admits")


if __name__ == "__main__":
    unittest.main()
