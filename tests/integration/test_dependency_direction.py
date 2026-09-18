"""§AR-system.4 — no component of `grund-core` reads one above it. Every
`crate::<other>` reference in every `.rs` file under
`crates/grund-core/src/<component>/` is judged against the order §AR-system.1
draws: model, grammar, config, workspace, scanner, checker, then queries and
writers as siblings that may not read each other, then api, then compat, which
may read anything and which nothing may read. The reads that still run the
other way are listed below, one entry per (file, item), and each one must still
be in the tree and still carry its §AR-system.4 note — so the list can only
shrink, and a new upward read fails."""

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CORE = REPO_ROOT / "crates" / "grund-core" / "src"

# §AR-system.1 as a rank: a component may read a lower rank and nothing else.
# `queries` and `writers` share a rank because they are siblings that may not
# read each other.

# `compat` is above everything, which is the same fact twice: the deprecated
# frontend may read anything, and nothing may read it.
ORDER = {
    "model": 0,
    "grammar": 1,
    "config": 2,
    "workspace": 3,
    "scanner": 4,
    "checker": 5,
    "queries": 6,
    "writers": 6,
    "api": 7,
    "compat": 8,
}

# `crate::<component>::<item>` or `crate::<component>::{<item>, …}`, in a `use`
# line or in a path written out where it is called.
REFERENCE = re.compile(r"\bcrate::([a-z_][a-z0-9_]*)::(\{[^}]*\}|[A-Za-z0-9_]+)")
NOTE = "§AR-system.4"

# The debt this refactor recorded rather than resolved: turning each component
# into a Rust module made every one of these visible, and resolving them is a
# design move rather than a refactor (§AR-core-module-layout.2).

# An entry is (file, `<component>::<item>`) and buys nothing else: the file must
# still make the read, and the import must still carry its §AR-system.4 note.
RECORDED_DEBT = {
    # grammar reads four components above it for records it is *handed*: the
    # `Config` a compiled grammar hangs off, the `[[kinds]]` row a near miss is
    # measured against, the scanner's per-line citation record, and the
    # workspace vocabulary a qualified citation is resolved with.

    # Every one of them is an argument rather than a lookup, so grammar still
    # opens no file and holds no rule of its own.
    "grammar/comment_block.rs": ("config::Config",),
    "grammar/comment_line.rs": ("config::Config", "workspace::WorkspaceCitationTarget"),
    "grammar/compiled.rs": ("config::KindConfig",),
    "grammar/ids.rs": ("config::Config",),
    "grammar/inline_note_layout.rs": ("config::Config", "workspace::WorkspaceCitationTarget"),
    "grammar/near_miss.rs": ("config::KindConfig",),
    "grammar/never_rewrite.rs": ("scanner::CitationLine",),
    "grammar/shorthand.rs": (
        "checker::ReferenceTier",
        "checker::WorkspaceCheckTarget",
        "config::Config",
        "config::ShorthandPolicy",
        "scanner::CitationLine",
        "workspace::WorkspaceProject",
    ),
    "grammar/shorthand_targets.rs": ("config::Config", "workspace::WorkspaceContext"),
    # the workspace reaches the scanner for the walk it asks about, and
    # `compat/` for four stderr lines: §FS-check.4.7, §FS-check.4.8,
    # §FS-check.4.10 and §FS-workspace.6.1 are settled before a report exists.

    # The query surfaces have no report to carry them
    # (§DF-unlisted-workspace-block.2.3): printing them lower would put a stream
    # write in a component directory, and carrying them higher is a redesign.

    # It is the first thing the retirement of `compat/` has to answer.
    "workspace/context.rs": (
        "compat::print_unlisted_workspace_block_warnings",
        "scanner::ScanError",
        "scanner::promote_qualified_legacy_citations",
        "scanner::scan_tree_with_workspace_overlays",
    ),
    "workspace/expand.rs": ("compat::warn_if_members_absorb_scan", "compat::warn_unread_block"),
    "workspace/id_candidates.rs": ("scanner::resolve_id_arg",),
    "workspace/members.rs": (
        "compat::warn_undecidable_ancestor_claim",
        "scanner::canonical_config_root",
        "scanner::is_hidden",
        "scanner::root_scope_roots",
        "scanner::walk_reads_any_file",
    ),
    "workspace/scope.rs": ("compat::warn_if_members_absorb_scan", "compat::warn_unread_block"),
    # the checker reads the writers for what it must compare a verdict to: the
    # entrypoint list and the three template renderers a managed block is
    # byte-compared against (§FS-check.3.5).

    # And the link target `grund fmt` would write (§FS-check.3.18).

    # Each of those is the writer's plan rather than a function of text, so none
    # of them moves down into grammar. The point-body pair is one query's answer
    # and not the checker's to own (§FS-show).
    "checker/agents.rs": (
        "writers::ConversationSurface",
        "writers::citation_directions_section",
        "writers::clickable_citations_section",
        "writers::companion_agent_entrypoints",
    ),
    "checker/index_entries.rs": ("writers::markdown_link_target",),
    "checker/sizes.rs": ("queries::PointBodyCache", "queries::point_body_pair"),
    # the three sibling edges between the queries and the writers, all of them
    # one fact: an answer that must agree with the formatter line for line
    # (§DF-show-cross-ref-flattening, §FS-lsp.1.4).

    # Either they move to a lower component together with the scanner state they
    # are written in terms of, or one of the two siblings owns both halves.
    "queries/batch.rs": ("writers::flatten_cross_ref_links",),
    "queries/body.rs": ("writers::flatten_cross_ref_links",),
    "queries/editor_on_type.rs": ("writers::FmtDirectives", "writers::FmtExcluded"),
}


def _components():
    return sorted(path.name for path in CORE.iterdir() if path.is_dir())


def _references():
    """Every `crate::<other>::<item>` a component file makes, as
    (file, `<other>::<item>`) -> the line it is written on."""
    found = {}
    for component in _components():
        for path in sorted((CORE / component).glob("**/*.rs")):
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


if __name__ == "__main__":
    unittest.main()
