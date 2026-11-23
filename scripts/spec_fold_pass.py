"""The record-driven fold pass over one checkout: what it writes, and what it leaves for a person.

The middle of `move_spec_sections.py` (§REQ-spec-section-names.permanence): given the
records and the section maps before and after, it decides each tracked file's new text,
surface by surface through `spec_fold_sites`, line by line through `spec_fold_lines`, and
gathers every site that needs a hand edit. It writes nothing; the script writes what it
returns, after the refusals it owns.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path

import spec_fold_coverage as coverage
import spec_fold_lines as lines
import spec_fold_map as folds
import spec_fold_records as recorded
import spec_fold_sites as sites


# The surfaces of a fold that hold a coordinate no scanner reads, by path.
SCHEMA = "schemas/grund.schema.json"
LEDGER = "tests/integration/citation_ledger.toml"
POINTER_GUARD = "tests/integration/test_pointer_guard.py"
COVERAGE_POLICY = "tests/integration/functional_spec_coverage_policy.rs"
AGREEMENT_FACTS = "tests/integration/command_agreement_facts/"
# A repeat is a record's, an input's or a golden's here, never the pass's to collapse.
KEEPS_REPEATS = ("docs/changelog.md", "docs/changelog/", AGREEMENT_FACTS, "tests/e2e/cases/")


def read_text(path: Path) -> str | None:
    try:
        return path.read_text(encoding="utf-8")
    except (UnicodeDecodeError, FileNotFoundError, IsADirectoryError):
        return None


@dataclass
class Log:
    """What one run reports, by kind."""

    hand: list[str] = field(default_factory=list)
    collapsed: list[str] = field(default_factory=list)
    self_cited: list[str] = field(default_factory=list)
    mixed: list[str] = field(default_factory=list)
    promised: list[str] = field(default_factory=list)


class FoldPass:
    """One run of the record-driven pass over one checkout."""

    def __init__(self, root: Path, records: list, before: folds.Spec, after: folds.Spec, done: set = frozenset()):
        self.root, self.before, self.after = root, before, after
        self.fold, _ = folds.fold_map(records, done)
        self.fold.known |= set(before.declarations)
        self.stale = folds.stale_anchors(before, after, self.fold)
        live = recorded.active(records, done)
        self.log = Log(hand=[line for record in live for line in record.hand])
        self.overrides = [override for record in live for override in record.overrides]
        self.promises = [promise for record in live for promise in record.promises]
        self.evidence: set[str] = set()
        self.agreement: dict[str, tuple[bytes, bytes]] = {}
        self.policy_read = False

    def rewrite(self, relative: str, text: str, log: Log) -> str:
        """The text the pass writes for one file, every surface but the coverage policy."""
        fold = self.fold
        if relative.rsplit("/", 1)[-1] == "spec.refs":
            return sites.rewrite_refs(text, fold)
        original = text
        quoted = lines.quoted(relative, text)
        masked = lines.mask(text, quoted) if quoted else None
        if quoted and masked is None:
            log.hand.append(f"{relative}: holds a character the pass masks fenced blocks with; "
                            "its fenced blocks were rewritten, so check them by hand")
        text = masked or text
        start = sites.released_part(relative, text)
        if start is None:
            text = sites.retarget_marked(text, fold)
        else:
            text = sites.retarget_marked(text[:start], fold) + sites.escape_released(text[start:], fold)
        if relative == SCHEMA:
            text = sites.retarget_schema(text, fold, self.after, self.stale, relative, log.hand)
        text = sites.repoint_links(text, relative, self.stale, log.hand)
        if relative == LEDGER:
            text = sites.retarget_ledger(text, fold)
        if relative == POINTER_GUARD:
            text = sites.retarget_pointers(text, fold, relative, log.hand)
        if relative.startswith("tests/integration/test_") and relative.endswith(".py"):
            text = sites.retarget_chapters(text, fold, self.before, relative, log.hand)
        case = relative.removeprefix(AGREEMENT_FACTS).removesuffix(".json")
        if relative.startswith(AGREEMENT_FACTS) and case in self.agreement:
            patched = sites.patch_agreement(text, *self.agreement[case])
            if patched is None:
                log.hand.append(f"{relative}: its `spec.refs` bytes are not the manifest's; regenerate them")
            text = patched or text
        if masked:
            text = lines.unmask(text)
            log.hand += lines.quoted_sites(relative, original, quoted, fold, self.stale)
        if not relative.startswith(KEEPS_REPEATS):
            text, collapsed, left = lines.collapse(relative, original, text)
            log.collapsed += collapsed
            log.hand += left
        log.self_cited += lines.self_citations(relative, original, text)
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
            self.log.hand += sites.bare_sites(relative, after, self.fold)
            self.log.hand += sites.run_sites(relative, after, self.fold)
        if relative.startswith("tests/integration/test_") and relative.endswith(".py"):
            self.log.hand += sites.relative_sites(relative, after, self.fold, self.before)
        if after != before and (mixed := lines.mixed(relative, before, after)):
            self.log.mixed.append(mixed)

    def plan(self, tracked: list[Path], untracked: list[Path]) -> tuple[list[Path], dict[Path, tuple[str, str]]]:
        """The untracked files the pass would change, and each tracked file's text before and after."""
        stranded = []
        for path in untracked:
            text = read_text(path)
            if text is not None and self.rewrite(str(path.relative_to(self.root)), text, Log()) != text:
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
            after = self.rewrite(relative, before, self.log)
            if relative == COVERAGE_POLICY:
                self.policy_read = True
                after = coverage.retarget_exceptions(after, self.fold, self.after, self.evidence, relative, self.log.hand)
                after = coverage.lead_promises(after, self.promises, self.fold, self.after, self.evidence,
                                               relative, self.log.hand, self.log.promised)
            self.note(relative, before, after)
            if after != before:
                rewrites[path] = (before, after)
        return [], rewrites

    def overridden(self) -> list[str]:
        """Each override of a record, with what the map wrote at its site."""
        found = []
        for override in self.overrides:
            if not override.file:
                found.append(f"{override.where}: overrides: {override.text}")
                continue
            had = override.old in self.fold.inherited
            mapped = self.fold.lookup(folds.LEGACY.sub(r"\1-", override.old)) or self.fold.lookup(override.old)
            if not had:
                self.fold.inherited.pop(override.old, None)
            wrote = f"the map wrote `{mapped}`" if mapped else "no record removes it"
            reason = f"  # {override.reason}" if override.reason else ""
            found.append(f"{override.file}:{override.line}: write `{override.new}` for `{override.old}`; "
                         f"{wrote} ({override.where}){reason}")
        return found

    def report(self, wrote: bool) -> None:
        log = self.log
        if self.promises and not self.policy_read:
            log.hand += [f"{COVERAGE_POLICY}: not tracked; add the lead promise `{p.id}` by hand" for p in self.promises]
        hand = [line for line in log.hand if ": pins: " in line] + self.overridden()
        hand += [line for line in log.hand if ": pins: " not in line]
        hand += [f"{coordinate}: cited, and under the removed {source}, which lists no such sub-point"
                 for coordinate, source in sorted(self.fold.inherited.items())]
        blocks = (
            (hand, "site(s) need a hand edit"),
            (log.self_cited, "self-citation(s) the retarget made; delete or reword each"),
            (log.collapsed, "repeated citation(s) collapsed on lines the pass changed"),
            (log.mixed, "file(s) whose code and #[cfg(test)] code both changed; split each across the commits"),
            (log.promised, f"lead promise(s) added to LEAD_PROMISES in {COVERAGE_POLICY}, for the coverage gate"),
        )
        for found, title in blocks:
            if found:
                print(f"{len(found)} {title}:")
                for line in found:
                    print(f"  {line}")
        if wrote:
            print("now run `grund fmt --write` so every link target and anchor follows")
