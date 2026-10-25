"""§AR-system.principle.inventory — what a path argument means and which files a run
reads are defined at one point, the `FS-scan` page, and the specs they were moved out
of, `FS-config`, `FS-workspace`, `FS-check`, `FS-list`, `FS-show`, `FS-refs` and
`FS-id`, cite it rather than restate it.
For each move this holds three things: the destination chapter holds the moved
sentence, the source section no longer does, and the source cites the destination,
so a restatement that grows back fails here. It also holds the page's chapter map,
that the page describes each command's path as it is today, `list`, `show` and `id`
resolving as `check` does now that §REQ-one-answer.differences.scope lists no row, the text its
sibling move leaves where it is, and that each frozen spec ends smaller."""

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
FS = REPO_ROOT / "docs" / "functional-spec"
SCAN = FS / "FS-scan.md"
INDEX = FS / "README.md"
# The marker, spelled so that this file cites nothing the page has not declared yet.
MARKER = "\N{SECTION SIGN}"
CITATION = MARKER + "FS-scan"
HEADING = re.compile(r"^(#{1,6}) (\S+?)(?::? )(.*)$")

CHAPTERS = (
    "terms",
    "path",
    "path.discovery", "path.check", "path.cover",
    "path.list", "path.show", "path.refs", "path.id",
    "projects",
    "projects.discovery", "projects.workspace-root", "projects.member", "projects.narrowed",
    "scope",
    "scope.include", "scope.homes", "scope.homes.why", "scope.roots",
    "scope.not-a-fence", "scope.hidden", "scope.extensions", "scope.ignore-files",
    "scope.links", "scope.links.names", "scope.links.unresolvable",
    "scope.links.owed", "scope.links.blind-spot",
    "identity",
    "identity.link-path", "identity.root-path", "identity.read-once", "identity.buffers",
    "text",
    "text.buffer", "text.buffer-only",
)

# (source file, source heading label, destination chapter it must cite,
#  fragments that moved out of it into that chapter)
MOVES = (
    ("FS-config.md", "3.5", "scope",
     ("is the set of directory names skipped at any depth",)),
    ("FS-config.md", "3.5.1", "scope.links",
     ("is followed only while its canonical target remains inside the independently checked project's canonical root",)),
    ("FS-config.md", "3.5.2", "identity.link-path",
     ("is reported at the in-tree link path, never the target's",)),
    ("FS-config.md", "3.5.2.1", "identity.root-path",
     ("is scanned and reported under the path the run was handed, never the physical one it resolves to",)),
    ("FS-config.md", "3.5.3", "scope.links.names",
     ("is excluded exactly as a real directory of that name would be",)),
    ("FS-config.md", "3.5.4", "identity.read-once",
     ("file is read once however many spellings reach it",)),
    ("FS-config.md", "3.5.5", "scope.links.unresolvable",
     ("a loop is pruned where it is met",)),
    ("FS-config.md", "3.5.6", "scope.links.owed",
     ("That report is owed only where the scan would otherwise have read through the link",)),
    ("FS-config.md", "3.5.6.1", "scope.links.blind-spot",
     ("would have been a directory to descend into had it resolved",)),
    ("FS-config.md", "3.5.7", "scope.include",
     ("never a subdirectory that merely happened to be passed as `grund`'s path argument",
      "intentionally names external content and is still scanned")),
    ("FS-config.md", "3.5.8", "scope.homes",
     ("A home that does not exist scans as nothing and earns no finding",)),
    ("FS-config.md", "3.5.9", "scope.roots",
     ("no scan root is pruned by `exclude`, an ignore file, or the hidden-directory rule",)),
    ("FS-config.md", "3.5.10", "scope.homes.why",
     ("outside `include` was never scanned, so its declarations did not exist",)),
    ("FS-config.md", "3.5.11", "scope.not-a-fence",
     ("brings into the scan is invisible rather than merely unchecked",)),
    ("FS-config.md", "3.5.12", "scope.hidden",
     ("it skips a file whose own name begins with `.` by the name that file wears",)),
    ("FS-config.md", "3.5.13", "scope.extensions",
     ("its native heading syntax still declares nothing",)),
    ("FS-config.md", "3.5.15", "scope.ignore-files",
     ("makes the scanner honor every form of ignore file the `ignore` crate recognizes",)),
    ("FS-workspace.md", "5", "projects.workspace-root",
     ("and all configured members, aggregates findings",)),
    ("FS-workspace.md", "5", "projects.narrowed",
     ("narrower than the config root loads the projects a run from `<path>` would load",)),
    ("FS-workspace.md", "5.1", "projects.member",
     ("discovers the member's own config first and, for a member that declares no `[workspace]` block of its own, validates it as an independent project",)),
    ("FS-workspace.md", "8.6.1", "path.cover",
     ("a scope narrower than the config root is one narrowed scan of the enclosing project",)),
    ("FS-workspace.md", "8.8", "projects.discovery",
     ("A scope inside a member with no config of its own is member-local all the same",)),
    ("FS-workspace.md", "8.8", "path.discovery",
     ("behaves as if the command were invoked from that path",)),
    ("FS-check.md", "1.3.6.1", "path.check",
     ("is the set of files the run reads so that citations resolve and citation edges count",
      "because a path may lie outside `include` and must still be read")),
    ("FS-list.md", "1", "path.list",
     ("is the directory or file whose entries are reported",)),
    ("FS-show.md", "1.4", "path.show",
     ("chooses the project the ID resolves in",)),
    ("FS-refs.md", "1", "path.refs",
     ("directory or file whose tree is scanned",)),
    ("FS-id.md", "1", "path.id",
     ("is the directory whose project is numbered",)),
)

# What the move leaves where it is: the key table's text-reading keys, which the
# languages step owns (#589), the refused `exclude` entry, which is a config refusal,
# the member run's alias error, the verdicts and diagnostic classes `check` keeps,
# and each command's own flags.
KEPT = (
    ("FS-config.md", "3.5", "comment_prefixes"),
    ("FS-config.md", "3.5", "`docstring_python` enables Python triple-quoted-string scanning"),
    ("FS-config.md", "3.5.16", "is one directory **component name**"),
    ("FS-workspace.md", "5.1", "`unknown project alias <path>` error at the citation site"),
    ("FS-check.md", "1.3.6.1", "Three verdicts move"),
    ("FS-check.md", "1.3.6.1", "Two classes of diagnostic are not about a scanned file"),
    ("FS-list.md", "1", "`--kind` selects configured citable kinds"),
    ("FS-show.md", "1.4", "is an accepted alias for scripts"),
    ("FS-refs.md", "1", "widen the section filter"),
    ("FS-id.md", "1", "minimum digit width of the number"),
)
# Sections the move leaves at their coordinate whatever their text becomes:
# 3.5.14 is the languages step's to rewrite (#589).
KEPT_SECTIONS = (("FS-config.md", "3.5.14"), ("FS-config.md", "3.5.16"))
# Text the page must not carry: the languages step's grammar, the stale copy of
# `check`'s path meaning, and `check`'s rule restated where the page cites it instead.
NOT_MOVED = (
    "comment_prefixes",
    "triple-quoted",
    "scans exactly the file or directory it is handed",
    "never the resolution",
)
# `check`'s own meaning, which no other chapter of the page may state as a rule.
CHECK_ONLY = "narrows the report"
# The page cites rather than restates these.
CITED = ("REQ-one-answer.differences.scope", "FS-workspace.6.2")
# Non-blank lines on main at 1640b6c47c; every file the move touches ends below it.
# FS-check's adds #634's 10-line chapter, §FS-check.3.17.4's wrap-test line (grund#655)
# and #756's 9 lines of checks.config-version-1: a restatement still has one line to regrow.

# FS-workspace's adds #823's 20 lines of §FS-workspace.5.1.1 and #921's 3 of §FS-workspace.2.2.2;
# FS-check's adds #836's 9 of §FS-check.2.3.1.3, #829's 17 of §FS-check.1.4.1 and §FS-check.2.2.1.4,
# and #906's 14 of §FS-check.3.5.3, §FS-check.3.5.5 and §FS-check.4.9.1; the margin is unchanged.

# #932 adds 4 lines of §FS-check.3.5.3.1; retain the one-line margin.
# #1010 adds 8 lines of §FS-workspace.6.1.7.5; the margin is unchanged.
BASELINES = {"FS-config.md": 765, "FS-workspace.md": 1194, "FS-check.md": 983}


def _flat(text):
    return " ".join(text.split()).lower()


def _sections(path):
    """Map each heading label to the text under it, up to the next heading."""
    sections, label, lines, fence = {}, None, [], None
    for line in path.read_text(encoding="utf-8").splitlines():
        opener = re.match(r"^ {0,3}(`{3,}|~{3,})", line)
        if fence:
            if opener and opener.group(1)[0] == fence[0] and len(opener.group(1)) >= len(fence):
                fence = None
        elif opener:
            fence = opener.group(1)
        match = None if fence or opener else HEADING.match(line)
        if match:
            if label is not None:
                sections.setdefault(label, "\n".join(lines))
            label, lines = match.group(2).rstrip(":").rstrip("."), []
        elif label is not None:
            lines.append(line)
    if label is not None:
        sections.setdefault(label, "\n".join(lines))
    return sections


def _cites(text, chapter):
    return re.search(re.escape(CITATION) + r"\." + re.escape(chapter) + r"(?![\w.-])", text)


class ScanMoveTests(unittest.TestCase):
    def scan(self):
        self.assertTrue(SCAN.is_file(), f"{SCAN.relative_to(REPO_ROOT)} is not there")
        return _sections(SCAN)

    def test_every_chapter_the_page_promises_is_declared(self):
        declared = self.scan()
        self.assertIn("FS-scan", declared)
        self.assertEqual([], [c for c in CHAPTERS if c not in declared], "chapters missing")
        self.assertRegex(INDEX.read_text(encoding="utf-8"), re.escape(f"[{CITATION}](FS-scan.md#"))

    def test_the_page_has_named_chapters_and_one_terms_chapter(self):
        text = self.scan() and SCAN.read_text(encoding="utf-8")
        tops = re.findall(r"^## (\S+?):? ", text, re.M)
        self.assertEqual([], [t for t in tops if t[0].isdigit()], "numbered top-level chapters")
        self.assertEqual(1, len(re.findall(r"^## terms: Terms$", text, re.M)), "Terms chapters")

    def test_each_moved_sentence_lives_at_its_destination_only(self):
        declared = _sections(SCAN) if SCAN.is_file() else {}
        for source, label, chapter, fragments in MOVES:
            body = _sections(FS / source).get(label)
            self.assertIsNotNone(body, f"{source} {label} is not there")
            self.assertTrue(_cites(body, chapter), f"{source} {label} does not cite FS-scan.{chapter}")
            for fragment in fragments:
                self.assertNotIn(_flat(fragment), _flat(body), f"{source} {label} still restates {chapter}")
                self.assertIn(_flat(fragment), _flat(declared.get(chapter, "")),
                              f"FS-scan.{chapter} does not hold the moved sentence")

    def test_the_page_describes_todays_paths_and_cites_what_it_does_not_restate(self):
        declared = self.scan()
        page = SCAN.read_text(encoding="utf-8")
        for target in CITED:
            self.assertIn(MARKER + target, page, f"FS-scan does not cite {target}")
        for fragment in NOT_MOVED:
            self.assertNotIn(_flat(fragment), _flat(page), f"FS-scan states {fragment!r}")
        stated = [label for label, body in declared.items()
                  if label != "path.check" and _flat(CHECK_ONLY) in _flat(body)]
        self.assertEqual([], stated, "a path narrowing the report is check's meaning, not a rule")

    def test_every_frozen_spec_ends_smaller(self):
        for name, baseline in BASELINES.items():
            lines = (FS / name).read_text(encoding="utf-8").splitlines()
            self.assertLess(sum(1 for line in lines if line.strip()), baseline, name)

    def test_what_the_move_leaves_behind_stays(self):
        for source, label, fragment in KEPT:
            self.assertIn(_flat(fragment), _flat(_sections(FS / source)[label]), f"{source} {label}")
        for source, label in KEPT_SECTIONS:
            self.assertIn(label, _sections(FS / source), f"{source} {label}")


if __name__ == "__main__":
    unittest.main()
