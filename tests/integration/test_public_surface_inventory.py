"""§AR-core-module-layout.1.2 — `lib.rs` holds the explicit `pub use` list that
is the crate's public surface, so the public-surface audit is complete exactly
when its inventory rows are that list: every root name has one row, no row names
something the root does not export, and no name is listed twice."""

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
LIB_RS = REPO_ROOT / "crates" / "grund-core" / "src" / "lib.rs"
INVENTORY = (
    REPO_ROOT
    / "docs"
    / "discussions"
    / "proposals"
    / "2026-09-22-grund-core-public-surface-inventory.md"
)

USE = re.compile(r"^\s*pub use\s+([^;]+);", re.MULTILINE | re.DOTALL)
# An unqualified `pub` at the crate root; `pub(crate) mod testing;` is not one.
ROOT_ITEM = re.compile(
    r"^pub\s+(?:unsafe\s+|async\s+|extern\s+\"[^\"]*\"\s+)*"
    r"(?:mod|fn|struct|enum|trait|union|type|const|static)\s+([A-Za-z_][A-Za-z0-9_]*)",
    re.MULTILINE,
)
# A row of the inventory: its first cell is the exported name in backticks.
ROW = re.compile(r"^\|\s*`([A-Za-z_][A-Za-z0-9_]*)`\s*\|", re.MULTILINE)

# The spine of the data-returning surface §FS-distribution.3.1 documents. A
# parser that stopped seeing these has stopped seeing the export list, and every
# set comparison below would agree with an empty inventory.
ENTRY_POINTS = (
    "check",
    "show",
    "scan",
    "refs",
    "list",
    "cover",
    "format_references",
    "propose_id",
    "complete_ids",
    "effective_config",
    "init",
)

# §AR-config.1, §DA-config-concern-records.4: the records ship at the root. The
# aliases they replaced were removed in 0.19.0 (§FS-distribution.3.1), each
# leaving one `lib.rs` line that says so in the release guard's landed tense.
CONCERN_RECORDS = ("Catalog", "Project", "Run", "Compiled", "effective_project")
REMOVED_ALIASES = ("Findings", "Config", "KindConfig", "CitationRules")
REMOVED_LINE = "was removed in 0.19.0"

# §FS-distribution.3.1.2: every row is promised, hidden, or on its way out.
FROZEN, HIDDEN, LEAVING = "frozen at 1.0", "hidden", "is removed in 0.21.0"
DISPOSITIONS = (FROZEN, HIDDEN, LEAVING)
FROZEN_COUNT, HIDDEN_COUNT = 115, 150
# §FS-distribution.3.1.3: the 30 root names the CLI assembled `--write` from,
# and the 9 methods of the kept vocabularies, deprecated in 0.19.0.
LEAVING_NAMES = frozenset((
    "agent_override_table", "block_outcome_verb", "BlockOutcome", "detect_clients",
    "expand_target", "GLOBAL_AGENT_INSTRUCTION_TARGETS", "GlobalAgentTarget",
    "GRUND_OPEN_RESOLVER", "install_agent_guidance_block", "install_managed_block",
    "install_reference_key", "integration_is_current", "INTEGRATIONS_BLOCK_VERSION",
    "known_agent", "known_agents_list", "known_clients_line", "LinkSupport",
    "merge_outcomes", "needs_wezterm_wiring", "read_optional_text", "RESOLVER_TARGET",
    "scan_user_config", "USER_CONFIG_TARGET", "user_grund_config_path", "UserConfigScan",
    "VSCODE_EXTENSION_JS", "vscode_integration_is_current", "VSCODE_PACKAGE_JSON",
    "WEZTERM_APPLY_CALL", "write_resolver_script",
))
LEAVING_METHODS = (
    ("IntegrationClient", "is_terminal"), ("IntegrationClient", "install_kind"),
    ("IntegrationClient", "snippet"), ("IntegrationClient", "config_target"),
    ("IntegrationClient", "install_command"), ("IntegrationClient", "comment_prefix"),
    ("IntegrationClient", "prepends_block"), ("IntegrationClient", "fresh_config_scaffold"),
    ("ConversationTarget", "accepted_list"),
)
# §DISC-grund-core-public-surface.6.3: a hidden row names its real reader, and
# nothing outside grund-core reads these.
NO_OUTSIDE_READER = (
    "can_replace_trigger_at", "cover_with_run_warnings", "cover_text_with_run_warnings",
    "cover_lines_with_run_warnings", "format_references_with_run_warnings",
    "list_sizes_with_run_warnings", "parse_cover_line_range", "lsp_snapshot_with_metadata",
)
NO_OUTSIDE_READER_TEXT = "no reader outside grund-core"
CORE_SRC = REPO_ROOT / "crates" / "grund-core" / "src"
# A `pub use` tree or root item and the attributes stacked above it; comment
# lines are blanked by `_source`, so they never separate the two.
ATTRIBUTED_USE = re.compile(r"((?:^[ \t]*#\[[^\]]*\]\s*)*)^[ \t]*pub use\s+([^;]+);", re.MULTILINE)
ATTRIBUTED_ITEM = re.compile(
    r"((?:^[ \t]*#\[(?:[^\[\]]|\[[^\]]*\])*\]\s*)*)^[ \t]*pub\s+(?:const\s+fn|fn|struct|enum|type|const|static)"
    r"\s+([A-Za-z_][A-Za-z0-9_]*)",
    re.MULTILINE,
)
ROOT_ATTRIBUTED_ITEM = re.compile(ATTRIBUTED_ITEM.pattern.replace("^[ \\t]*", "^"), re.MULTILINE)
# A deprecation whose note is one source line, so the release guard reads it.
ONE_LINE_DEPRECATION = re.compile(
    r'#\[deprecated\(\s*since\s*=\s*"0\.19\.0"\s*,\s*note\s*=\s*"([^"\n]*)"\s*,?\s*\)\]'
)
# An inventory row's first and last cells: the name and its disposition.
DISPOSITION = re.compile(r"^\|\s*`([A-Za-z_][A-Za-z0-9_]*)`\s*\|.*\|\s*([^|]+?)\s*\|\s*$", re.MULTILINE)


class GlobReExport(AssertionError):
    pass


def _source():
    """`lib.rs` without its comments, which quote the syntax below in prose."""
    lines = LIB_RS.read_text(encoding="utf-8").splitlines()
    return "\n".join("" if line.lstrip().startswith("//") else line for line in lines)


def _top_level_items(inner):
    depth = 0
    item = ""
    for character in inner:
        if character == "{":
            depth += 1
        elif character == "}":
            depth -= 1
        if character == "," and depth == 0:
            yield item
            item = ""
        else:
            item += character
    yield item


def _exported_names(tree):
    """The names one `pub use` tree puts at the crate root."""
    tree = " ".join(tree.split())
    if tree.endswith("}"):
        head, inner = tree.split("{", 1)
        names = []
        for item in _top_level_items(inner[: inner.rindex("}")]):
            item = item.strip()
            if not item:
                continue
            if item == "self":
                names.append(head.strip().rstrip(":").split("::")[-1])
            else:
                names.extend(_exported_names(item))
        return names
    if tree.endswith("*"):
        raise GlobReExport(f"glob re-export at the crate root: pub use {tree};")
    if " as " in tree:
        return [tree.split(" as ")[-1].strip()]
    return [tree.split("::")[-1].strip()]


def public_root_names():
    """Every distinct spelling reachable as `grund_core::<name>`."""
    source = _source()
    names = []
    for tree in USE.findall(source):
        names.extend(_exported_names(tree))
    names.extend(ROOT_ITEM.findall(source))
    return names


def inventory_names():
    """The names the audit's table claims to have classified.

    An inventory that is absent classifies nothing, which is the same finding as
    one that stops short: the names it does not carry are the ones missing."""
    if not INVENTORY.is_file():
        return []
    return ROW.findall(INVENTORY.read_text(encoding="utf-8"))


def hidden_root_names():
    """The root names `lib.rs` itself puts under `#[doc(hidden)]`: a hidden
    `pub use` tree or a hidden root item (§FS-distribution.3.1.2)."""
    source = _source()
    hidden = set()
    for attributes, tree in ATTRIBUTED_USE.findall(source):
        if "#[doc(hidden)]" in attributes:
            hidden.update(_exported_names(tree))
    for attributes, name in ROOT_ATTRIBUTED_ITEM.findall(source):
        if "#[doc(hidden)]" in attributes:
            hidden.add(name)
    return hidden


def one_line_deprecations():
    """`name -> note` for every item in grund-core deprecated in 0.19.0 with a
    note on one source line, and `Type::method -> note` for methods."""
    found = {}
    for path in sorted(CORE_SRC.rglob("*.rs")):
        lines = path.read_text(encoding="utf-8").splitlines()
        text = "\n".join("" if line.lstrip().startswith("//") else line for line in lines)
        for attributes, name in ATTRIBUTED_ITEM.findall(text):
            match = ONE_LINE_DEPRECATION.search(attributes)
            if match:
                found[name] = match.group(1)
        for impl_name, body in re.findall(r"^impl\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{(.*?)^\}", text, re.MULTILINE | re.DOTALL):
            for attributes, name in ATTRIBUTED_ITEM.findall(body):
                match = ONE_LINE_DEPRECATION.search(attributes)
                if match:
                    found[f"{impl_name}::{name}"] = match.group(1)
    return found


def inventory_rows():
    """`name -> the row's whole line`, for reading a cell's wording."""
    if not INVENTORY.is_file():
        return {}
    return {
        match.group(1): match.group(0)
        for match in re.finditer(r"^\|\s*`([A-Za-z_][A-Za-z0-9_]*)`\s*\|.*$", INVENTORY.read_text(encoding="utf-8"), re.MULTILINE)
    }


def inventory_dispositions():
    if not INVENTORY.is_file():
        return {}
    return dict(DISPOSITION.findall(INVENTORY.read_text(encoding="utf-8")))


def _sample(names, limit=8):
    names = sorted(names)
    shown = ", ".join(names[:limit])
    return shown if len(names) <= limit else f"{shown}, … ({len(names)} in all)"


class PublicSurfaceInventoryTests(unittest.TestCase):
    def test_the_crate_root_still_reads_as_an_explicit_export_list(self):
        """The guard on every comparison below: a parse that found nothing would
        agree with an inventory that classified nothing."""
        names = set(public_root_names())
        missing = sorted(name for name in ENTRY_POINTS if name not in names)
        self.assertEqual(
            [],
            missing,
            f"{LIB_RS} no longer exports the documented entry points, or the "
            f"export list no longer parses: {missing}",
        )

    def test_the_crate_root_re_exports_nothing_by_glob(self):
        """A glob makes the public surface unenumerable, so an audit of it could
        only be a guess (§AR-core-module-layout.1.2)."""
        try:
            public_root_names()
        except GlobReExport as glob:
            self.fail(str(glob))

    def test_every_public_root_name_has_an_inventory_row(self):
        missing = set(public_root_names()) - set(inventory_names())
        self.assertEqual(
            "",
            _sample(missing),
            f"public root names with no row in {INVENTORY.relative_to(REPO_ROOT)}",
        )

    def test_the_inventory_names_only_public_root_names(self):
        extra = set(inventory_names()) - set(public_root_names())
        self.assertEqual(
            "",
            _sample(extra),
            "inventory rows naming what the crate root does not export",
        )

    def test_no_public_root_name_is_listed_twice(self):
        counted = {}
        for name in inventory_names():
            counted[name] = counted.get(name, 0) + 1
        duplicated = {name for name, count in counted.items() if count > 1}
        self.assertEqual(
            "",
            _sample(duplicated),
            "inventory rows repeating one name, so one name has two dispositions",
        )


class ConcernRecordSurfaceTests(unittest.TestCase):
    """§AR-config.1: the records are root names, and the aliases they replaced
    are gone from the root, as their 0.17.0 notes said (§FS-distribution.3.1)."""

    def test_the_concern_records_are_root_names(self):
        missing = [name for name in CONCERN_RECORDS if name not in set(public_root_names())]
        self.assertEqual([], missing)

    def test_the_removed_aliases_are_absent_and_each_leaves_its_removal_line(self):
        roots = set(public_root_names())
        present = [name for name in REMOVED_ALIASES if name in roots]
        self.assertEqual([], present, "aliases noticed for removal in 0.19.0 are still root names")
        lines = LIB_RS.read_text(encoding="utf-8").splitlines()
        unmarked = [
            name
            for name in REMOVED_ALIASES
            if not any(f"`{name}`" in line and REMOVED_LINE in line for line in lines)
        ]
        self.assertEqual(
            [], unmarked, f"aliases with no `{REMOVED_LINE}` line naming them in {LIB_RS.name}"
        )


class OneDotZeroPromiseTests(unittest.TestCase):
    """§FS-distribution.3.1.2: every root name is promised, hidden, or noticed
    for removal in 0.21.0, and `lib.rs` says which of the first two it is."""

    def test_every_row_has_one_of_the_three_dispositions(self):
        wrong = {
            name: disposition
            for name, disposition in inventory_dispositions().items()
            if disposition not in DISPOSITIONS
        }
        self.assertEqual("", _sample(f"{n}: {d}" for n, d in wrong.items()))

    def test_the_frozen_rows_are_the_115_names_the_promise_reaches(self):
        frozen = [n for n, d in inventory_dispositions().items() if d == FROZEN]
        self.assertEqual(FROZEN_COUNT, len(frozen))

    def test_hidden_rows_are_hidden_in_lib_rs_and_frozen_rows_are_not(self):
        dispositions = inventory_dispositions()
        hidden = hidden_root_names()
        hidden_rows = {n for n, d in dispositions.items() if d == HIDDEN}
        self.assertEqual(HIDDEN_COUNT, len(hidden_rows), "the inventory's `hidden` rows")
        visible = hidden_rows - hidden
        self.assertEqual("", _sample(visible), "`hidden` rows lib.rs does not put under #[doc(hidden)]")
        frozen_hidden = {n for n, d in dispositions.items() if d == FROZEN} & hidden
        self.assertEqual("", _sample(frozen_hidden), "`frozen at 1.0` rows lib.rs hides")

    def test_the_leaving_rows_are_the_30_integrations_names(self):
        leaving = {n for n, d in inventory_dispositions().items() if d == LEAVING}
        self.assertEqual("", _sample(leaving ^ LEAVING_NAMES), "rows differing from the 30")

    def test_every_leaving_name_and_method_carries_a_one_line_note_naming_0_21_0(self):
        notes = one_line_deprecations()
        wanted = sorted(LEAVING_NAMES) + [f"{owner}::{method}" for owner, method in LEAVING_METHODS]
        wrong = [name for name in wanted if LEAVING not in notes.get(name, "")]
        self.assertEqual(
            "", _sample(wrong), f'no one-line #[deprecated(since = "0.19.0")] note with `{LEAVING}`'
        )

    def test_the_hidden_rows_nothing_outside_core_reads_say_so(self):
        rows = inventory_rows()
        wrong = [name for name in NO_OUTSIDE_READER if NO_OUTSIDE_READER_TEXT not in rows.get(name, "")]
        self.assertEqual([], wrong, f"rows that do not say `{NO_OUTSIDE_READER_TEXT}`")


if __name__ == "__main__":
    unittest.main()
