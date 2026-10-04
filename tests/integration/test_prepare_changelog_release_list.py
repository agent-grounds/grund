"""§FS-distribution.4.6 — the release lists the pull requests merged since the
previous tag itself: `prepare` asks the forge about every commit in the range
(§FS-distribution.4.6.1), writes one escaped, linked line per merged pull request,
newest first by git (§FS-distribution.4.6.2), adds the compatibility notices the
decisions gained since the tag (§FS-distribution.4.6.4), rotates the former inline
release into its archive with a counted summary (§FS-distribution.4.5), and
otherwise refuses and leaves the tree as it was (§FS-distribution.4.6.3).

Each case runs the script the way the release workflows do, in a throwaway
repository, against the stub forge of `release_forge_fixture.py`. The
compatibility notices on their own are `test_prepare_changelog_release_notices.py`."""

import json
import unittest

from release_forge_fixture import ForgeRepository, pull_request

VERSION, DATE = "0.3.0", "2026-10-05"

# The previous release, inline in the earlier Keep-a-Changelog shape, as 0.15.0 is today.
PREVIOUS = """## 2. [0.2.0] — 2026-05-17

### Added

- [\u00a7FS-x](functional-spec/FS-x.md#a): something new. (PR #4)
- Another addition. (PR #5)

### Fixed

- A fix. (PR #6)

"""
CHANGELOG = f"""# Changelog

Intro.

## 1. Conventions

How the sections are written.

{PREVIOUS}## 3. Older releases

- [0.1.0](changelog/0.1.0.md) — 2026-05-14: first published release.
"""
DECISION = """# DF-new: cover fails a broken workspace

**Status:** Accepted

## 1. Decision

`cover` fails a workspace it cannot expand.

## release-note: Release note

- **`cover` now fails a broken workspace.** Who this breaks: a CI step that runs it
  on such a workspace. See [\u00a7DF-new](#df-new-cover-fails-a-broken-workspace) and
  [\u00a7REQ-x.1](../../requirements/REQ-x.md#1-y).
"""
BODY = """- [Correct the verdict of \\`cover\\`](https://github.com/agent-grounds/grund/pull/13) (PR #13)
- [Reword the guide](https://github.com/agent-grounds/grund/pull/12) (PR #12)
- [Fix the \\[&sect;FS-x\\] \\*glob\\*](https://github.com/agent-grounds/grund/pull/11) (PR #11)
- [Add the parser](https://github.com/agent-grounds/grund/pull/10) (PR #10)

### Compatibility notices

- **`cover` now fails a broken workspace.** Who this breaks: a CI step that runs it
  on such a workspace. See [\u00a7DF-new](decisions/functional/DF-new.md#df-new-cover-fails-a-broken-workspace) and
  [\u00a7REQ-x.1](requirements/REQ-x.md#1-y).
"""
RELEASED = f"""# Changelog

Intro.

## 1. Conventions

How the sections are written.

## 2. [{VERSION}] — {DATE}

{BODY}
## 3. Older releases

- [0.2.0](changelog/0.2.0.md) — 2026-05-17: 3 pull requests, 0 compatibility notices.
- [0.1.0](changelog/0.1.0.md) — 2026-05-14: first published release.
"""
ARCHIVED = """# 0.2.0 — 2026-05-17

### Added

- [\u00a7FS-x](../functional-spec/FS-x.md#a): something new. (PR #4)
- Another addition. (PR #5)

### Fixed

- A fix. (PR #6)

"""


class ReleaseHistory(ForgeRepository):
    """The history every list case starts from: tagged `v0.2.0`, then the
    `-dev` opener, four rebase-merged pull requests - one of two commits, one
    touching docs only, one adding a compatibility notice - and one commit
    pushed straight to `main`."""

    def _history(self, changelog: str = CHANGELOG) -> None:
        self._repository({"docs/changelog.md": changelog, "Cargo.toml": 'version = "0.2.0"\n'})
        self.opener = self._land("Open 0.2.1-dev for development", {"Cargo.toml": 'version = "0.2.1-dev"\n'})
        self._answer(self.opener)
        (self.parser,) = self._merged(10, "Add the parser", ("Add the parser", {"src/parse.rs": "fn parse() {}\n"}))
        self.glob = self._merged(
            11,
            "Fix  the [\u00a7FS-x]\n *glob*",
            ("Match the glob", {"src/glob.rs": "fn glob() {}\n"}),
            ("Test the glob", {"tests/glob.rs": "fn test() {}\n"}),
        )
        # The forge also names an open pull request and one merged into another base for this commit.
        self._answer(
            self.parser,
            pull_request(10, "Add the parser", self.parser),
            pull_request(98, "Backport the parser", self.parser, base="release-0.2"),
            pull_request(99, "Someone else's draft", "0" * 40, merged=False),
        )
        self.stray = self._land("Push straight to main", {"src/stray.rs": "fn stray() {}\n"})
        self._answer(self.stray)
        self._merged(12, "Reword the guide", ("Reword the guide", {"docs/guide.md": "Reworded.\n"}))
        self._merged(
            13,
            "Correct the verdict of `cover`",
            ("Correct cover", {"docs/decisions/functional/DF-new.md": DECISION, "src/cover.rs": "fn cover() {}\n"}),
        )


class EndToEndTests(ReleaseHistory, unittest.TestCase):
    def test_prepare_writes_the_release_the_forge_and_the_tree_describe(self) -> None:
        self._history()
        self.assertSucceeded(self._script("prepare", VERSION, "--date", DATE))
        self.assertEqual(RELEASED, self._read())
        self.assertEqual(ARCHIVED, self._read("docs/changelog/0.2.0.md"))

    def test_preview_prints_the_body_prepare_writes_and_writes_nothing(self) -> None:
        self._history()
        preview = self._script("preview")
        self.assertSucceeded(preview)
        self.assertEqual(BODY.strip("\n"), preview.stdout.strip("\n"))
        self.assertEqual("", self._git(self.repo, "status", "--porcelain", "--untracked-files=all"))

    def test_the_release_notes_are_the_body_prepare_wrote(self) -> None:
        self._history()
        self.assertSucceeded(self._script("prepare", VERSION, "--date", DATE))
        notes = self.scratch / "notes.md"
        self.assertSucceeded(self._script("notes", VERSION, "--output", str(notes)))
        self.assertEqual(BODY.strip("\n"), notes.read_text(encoding="utf-8").strip("\n"))

    def test_every_commit_in_the_range_is_asked_about(self) -> None:
        self._history()
        self.assertSucceeded(self._script("prepare", VERSION, "--date", DATE))
        asked = {
            argument.split("/commits/")[1].split("/")[0]
            for line in self.log.read_text(encoding="utf-8").splitlines()
            for argument in json.loads(line)
            if "/commits/" in argument
        }
        in_range = self._git(self.repo, "rev-list", "v0.2.0..HEAD").split()
        self.assertEqual(len(in_range), len(asked), asked)
        for commit in in_range:
            self.assertTrue(any(commit.startswith(sha) for sha in asked), commit)


class ListTests(ReleaseHistory, unittest.TestCase):
    def _section(self) -> str:
        changelog = self._read()
        return changelog.split(f"## 2. [{VERSION}] — {DATE}\n\n", 1)[1].split("\n## 3. Older releases", 1)[0]

    def _lines(self) -> list[str]:
        return [line for line in self._section().split("\n### ", 1)[0].splitlines() if line]

    def test_the_list_is_newest_first_by_git_not_by_the_forge_s_timestamps(self) -> None:
        self._history()
        # The forge claims #10 merged last; git says it landed first.
        for commit, pulls in self.answers["commits"].items():
            for pull in pulls:
                pull["merged_at"] = f"2026-09-{40 - pull['number'] % 30:02d}T00:00:00Z" if pull["merged_at"] else None
        self.assertSucceeded(self._script("prepare", VERSION, "--date", DATE))
        self.assertEqual(["#13", "#12", "#11", "#10"], [line.rsplit("(PR ", 1)[1].rstrip(")") for line in self._lines()])

    def test_a_pull_request_is_placed_by_its_last_commit_and_listed_once(self) -> None:
        self._history()
        self.assertSucceeded(self._script("prepare", VERSION, "--date", DATE))
        self.assertEqual(1, sum("(PR #11)" in line for line in self._lines()))

    def test_only_merged_pull_requests_into_main_from_the_range_are_listed(self) -> None:
        self._history()
        self._merged(14, "Already released", ("Late commit", {"src/late.rs": "fn late() {}\n"}))
        late = self._git(self.repo, "rev-parse", "HEAD")
        self._answer(late, pull_request(14, "Already released", self.base))
        self.assertSucceeded(self._script("prepare", VERSION, "--date", DATE))
        for absent in ("(PR #98)", "(PR #99)", "(PR #14)"):
            with self.subTest(absent=absent):
                self.assertNotIn(absent, self._section())

    def test_docs_only_pull_requests_are_listed(self) -> None:
        self._history()
        self.assertSucceeded(self._script("prepare", VERSION, "--date", DATE))
        self.assertIn("- [Reword the guide](https://github.com/agent-grounds/grund/pull/12) (PR #12)", self._lines())

    def test_the_title_is_collapsed_and_escaped(self) -> None:
        self._repository({"docs/changelog.md": CHANGELOG})
        title = "a\\b `c` *d* _e_ [f] <g> & \u00a7h\t\n  i "
        self._merged(20, title, ("Escape", {"src/e.rs": "fn e() {}\n"}))
        self.assertSucceeded(self._script("prepare", VERSION, "--date", DATE))
        self.assertEqual(
            ["- [a\\\\b \\`c\\` \\*d\\* \\_e\\_ \\[f\\] \\<g\\> \\& &sect;h i](https://github.com/agent-grounds/grund/pull/20) (PR #20)"],
            self._lines(),
        )

    def test_a_commit_with_no_pull_request_is_warned_about_once_and_the_dev_opener_is_not(self) -> None:
        self._history()
        result = self._script("prepare", VERSION, "--date", DATE)
        self.assertSucceeded(result)
        warnings = [line for line in result.stderr.splitlines() if "warning:" in line]
        self.assertEqual(1, len(warnings), result.stderr)
        self.assertIn(self.stray[:7], warnings[0])
        self.assertIn("Push straight to main", warnings[0])
        self.assertFalse(any(self.opener[:7] in line for line in result.stderr.splitlines()), result.stderr)

    def test_a_higher_tag_on_another_branch_is_not_the_previous_release(self) -> None:
        self._history()
        identity = ("-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid")
        elsewhere = self._git(self.repo, *identity, "commit-tree", "-m", "Elsewhere", f"{self.base}^{{tree}}")
        self._git(self.repo, "tag", "v0.9.0", elsewhere)
        self.assertSucceeded(self._script("prepare", VERSION, "--date", DATE))
        self.assertEqual(RELEASED, self._read())


class ArchiveSummaryTests(ReleaseHistory, unittest.TestCase):
    """§FS-distribution.4.5: the archive link counts what the archived release held."""

    def test_a_release_in_the_list_shape_is_counted_as_pull_requests_and_notices(self) -> None:
        listed = (
            "## 2. [0.2.0] — 2026-05-17\n\n"
            "- [One](https://github.com/agent-grounds/grund/pull/4) (PR #4)\n"
            "- [Two](https://github.com/agent-grounds/grund/pull/5) (PR #5)\n\n"
            "### Compatibility notices\n\n- **A notice.** Who this breaks: nobody.\n\n"
        )
        self._history(CHANGELOG.replace(PREVIOUS, listed))
        self.assertSucceeded(self._script("prepare", VERSION, "--date", DATE))
        self.assertIn("- [0.2.0](changelog/0.2.0.md) — 2026-05-17: 2 pull requests, 1 compatibility notice.\n", self._read())

    def test_a_count_of_one_is_singular(self) -> None:
        single = "## 2. [0.2.0] — 2026-05-17\n\n- [One](https://github.com/agent-grounds/grund/pull/4) (PR #4)\n\n"
        self._history(CHANGELOG.replace(PREVIOUS, single))
        self.assertSucceeded(self._script("prepare", VERSION, "--date", DATE))
        self.assertIn("- [0.2.0](changelog/0.2.0.md) — 2026-05-17: 1 pull request, 0 compatibility notices.\n", self._read())


class RefusalTests(ReleaseHistory, unittest.TestCase):
    """§FS-distribution.4.6.3: each refusal names its case and writes nothing."""

    def test_a_missing_gh_is_refused(self) -> None:
        self._history()
        self.assertRefusedUntouched(self._script("prepare", VERSION, "--date", DATE, gh=False), "gh")

    def test_a_forge_call_that_fails_is_refused(self) -> None:
        self._history()
        self.answers["fail"][self.glob[0]] = "Server Error"
        self.assertRefusedUntouched(self._script("prepare", VERSION, "--date", DATE), self.glob[0][:7])

    def test_a_commit_the_forge_does_not_answer_for_is_refused(self) -> None:
        self._history()
        del self.answers["commits"][self.stray]
        self.assertRefusedUntouched(self._script("prepare", VERSION, "--date", DATE), self.stray[:7])

    def test_a_shallow_clone_is_refused(self) -> None:
        self._history()
        clone = self.scratch / "shallow"
        self._git(self.scratch, "clone", "-q", "--depth", "2", "--no-local", f"file://{self.repo}", str(clone))
        self._git(clone, "fetch", "-q", "--depth", "2", "origin", "tag", "v0.2.0")
        result = self._script("prepare", VERSION, "--date", DATE, cwd=clone)
        self.assertRefusedUntouched(result, "shallow", repo=clone)

    def test_a_tag_that_does_not_name_the_inline_release_is_refused(self) -> None:
        self._history(CHANGELOG.replace("[0.2.0] — 2026-05-17", "[0.1.9] — 2026-05-17"))
        self.assertRefusedUntouched(self._script("prepare", VERSION, "--date", DATE), "v0.2.0", "0.1.9")

    def test_a_url_that_is_not_this_repository_s_pull_request_is_refused(self) -> None:
        self._history()
        elsewhere = "https://github.com/someone/else/pull/12"
        for pulls in self.answers["commits"].values():
            for pull in pulls:
                if pull["number"] == 12:
                    pull["html_url"] = elsewhere
        self.assertRefusedUntouched(self._script("prepare", VERSION, "--date", DATE), elsewhere)

    def test_a_range_with_no_pull_request_is_refused(self) -> None:
        self._repository({"docs/changelog.md": CHANGELOG})
        self._answer(self._land("Open 0.2.1-dev for development", {"Cargo.toml": 'version = "0.2.1-dev"\n'}))
        self._answer(self._land("Push straight to main", {"src/stray.rs": "fn stray() {}\n"}))
        self.assertRefusedUntouched(self._script("prepare", VERSION, "--date", DATE), "no pull request")


if __name__ == "__main__":
    unittest.main()
