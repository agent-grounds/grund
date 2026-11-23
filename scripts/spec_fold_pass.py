"""The record-driven fold pass over one checkout: what it writes, and what it leaves for a person.

The middle of `move_spec_sections.py` (§REQ-spec-section-names.permanence): given the
records and the section maps before and after, it decides each tracked file's new text,
surface by surface through `spec_fold_sites`, and gathers every site that needs a hand edit.
It writes nothing; the script writes what it returns, after the refusals it owns.
"""

from __future__ import annotations

from pathlib import Path

import spec_fold_map as folds
import spec_fold_sites as sites


# The surfaces of a fold that hold a coordinate no scanner reads, by path.
SCHEMA = "schemas/grund.schema.json"
LEDGER = "tests/integration/citation_ledger.toml"
POINTER_GUARD = "tests/integration/test_pointer_guard.py"
COVERAGE_POLICY = "tests/integration/functional_spec_coverage_policy.rs"
AGREEMENT_FACTS = "tests/integration/command_agreement_facts/"


def read_text(path: Path) -> str | None:
    try:
        return path.read_text(encoding="utf-8")
    except (UnicodeDecodeError, FileNotFoundError, IsADirectoryError):
        return None


class FoldPass:
    """One run of the record-driven pass over one checkout."""

    def __init__(self, root: Path, records: list[folds.Record], before: folds.Spec, after: folds.Spec):
        self.root, self.before, self.after = root, before, after
        self.fold, _ = folds.fold_map(records)
        self.fold.known |= set(before.declarations)
        self.stale = folds.stale_anchors(before, after, self.fold)
        self.hand: list[str] = [line for record in records for line in record.hand]
        self.evidence: set[str] = set()
        self.agreement: dict[str, tuple[bytes, bytes]] = {}

    def rewrite(self, relative: str, text: str) -> str:
        """The text the pass writes for one file, every surface but the coverage policy."""
        fold = self.fold
        if relative.rsplit("/", 1)[-1] == "spec.refs":
            return sites.rewrite_refs(text, fold)
        start = sites.released_part(relative, text)
        if start is None:
            text = sites.retarget_marked(text, fold)
        else:
            text = sites.retarget_marked(text[:start], fold) + sites.escape_released(text[start:], fold)
        if relative == SCHEMA:
            text = sites.retarget_schema(text, fold, self.after, self.stale, relative, self.hand)
        text = sites.repoint_links(text, relative, self.stale, self.hand)
        if relative == LEDGER:
            text = sites.retarget_ledger(text, fold)
        if relative == POINTER_GUARD:
            text = sites.retarget_pointers(text, fold, relative, self.hand)
        if relative.startswith("tests/integration/test_") and relative.endswith(".py"):
            text = sites.retarget_chapters(text, fold, self.before, relative, self.hand)
        case = relative.removeprefix(AGREEMENT_FACTS).removesuffix(".json")
        if relative.startswith(AGREEMENT_FACTS) and case in self.agreement:
            patched = sites.patch_agreement(text, *self.agreement[case])
            if patched is None:
                self.hand.append(f"{relative}: its `spec.refs` bytes are not the manifest's; regenerate them")
            text = patched or text
        return text

    def note(self, relative: str, before: str, after: str) -> None:
        """What the rewritten text gives the coverage gate, and what is left for a person."""
        if relative.rsplit("/", 1)[-1] == "spec.refs":
            self.evidence.update(line.strip() for line in after.split("\n") if line.strip().startswith("FS-"))
            case = relative.removeprefix("tests/e2e/cases/").removesuffix("/spec.refs")
            if after != before and "/" not in case:
                self.agreement[case] = (before.encode("utf-8"), after.encode("utf-8"))
            return
        if sites.is_live_test_source(relative):
            self.evidence.update(match.group(1) for match in sites.MARKED.finditer(after))
        if sites.released_part(relative, after) is None:
            self.hand += sites.bare_sites(relative, after, self.fold)
            self.hand += sites.run_sites(relative, after, self.fold)
        if relative.startswith("tests/integration/test_") and relative.endswith(".py"):
            self.hand += sites.relative_sites(relative, after, sorted(self.fold.targets))

    def plan(self, tracked: list[Path], untracked: list[Path]) -> tuple[list[Path], dict[Path, tuple[str, str]]]:
        """The untracked files the pass would change, and each tracked file's text before and after."""
        stranded = []
        for path in untracked:
            text = read_text(path)
            if text is not None and self.rewrite(str(path.relative_to(self.root)), text) != text:
                stranded.append(path)
        if stranded:
            return stranded, {}
        # Manifests first: the agreement facts hold their bytes, and the coverage policy their evidence.
        files = sorted(tracked, key=lambda path: (path.name != "spec.refs", path == self.root / COVERAGE_POLICY))
        rewrites = {}
        for path in files:
            relative = str(path.relative_to(self.root))
            before = read_text(path)
            if before is None:
                continue
            after = self.rewrite(relative, before)
            if relative == COVERAGE_POLICY:
                after = sites.retarget_exceptions(after, self.fold, self.after, self.evidence, relative, self.hand)
            self.note(relative, before, after)
            if after != before:
                rewrites[path] = (before, after)
        return [], rewrites

    def report(self, wrote: bool) -> None:
        for coordinate, source in sorted(self.fold.inherited.items()):
            self.hand.append(f"{coordinate}: cited, and under the removed {source}, which lists no such sub-point")
        if self.hand:
            print(f"{len(self.hand)} site(s) need a hand edit:")
            for line in self.hand:
                print(f"  {line}")
        if wrote:
            print("now run `grund fmt --write` so every link target and anchor follows")
