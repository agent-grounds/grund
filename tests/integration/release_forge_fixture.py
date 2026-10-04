"""The fixture the release's pull-request list is tested on (§FS-distribution.4.6):
a throwaway repository tagged at its previous release, and a stub `gh` on `PATH`
that answers for the forge from a table the test writes, so no case reaches the
network.

The stub answers two calls and refuses everything else, so the seam is exact:

- `gh api repos/{owner}/{repo}` — the repository, `full_name` and `html_url`;
- `gh api repos/{owner}/{repo}/commits/<sha>/pulls` — the pull requests the
  commit belongs to, as the forge's JSON array: `number`, `title`, `html_url`,
  `merged_at`, `base.ref`, `merge_commit_sha`.

A leading `/` and literal `{owner}/{repo}` placeholders are accepted, and so are
`--paginate`, `-H`/`--header` and `-X`/`--method GET`. `--jq`, `-q` and
`--template` are refused: the stub hands back the JSON and the caller reads it.
A commit the table holds no answer for fails the way the forge fails for an
unknown commit, and a commit the table marks failed fails as a server error.
Every call is logged, one JSON argument list per line.

It is not named `test_*`, so `unittest discover` imports it only from the
modules that use it."""

import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from subprocess import CompletedProcess

from changelog_gate_fixture import REPO_ROOT, GitFixture, environment

SCRIPT_PATH = REPO_ROOT / "scripts" / "prepare_changelog_release.py"
REPOSITORY = {"full_name": "agent-grounds/grund", "html_url": "https://github.com/agent-grounds/grund"}

STUB = r'''#!{python}
import json, os, re, sys

args = sys.argv[1:]
with open(os.environ["GH_STUB_LOG"], "a", encoding="utf-8") as log:
    log.write(json.dumps(args) + "\n")
if not args or args[0] != "api":
    sys.exit(f"gh stub: only `gh api` is answered, not {{args!r}}")
paths, rest = [], iter(args[1:])
for arg in rest:
    if arg in ("--jq", "-q", "--template", "-t"):
        sys.exit(f"gh stub: {{arg}} is not evaluated; read the JSON the call returns")
    if arg in ("-H", "--header", "-X", "--method"):
        value = next(rest, "")
        if arg in ("-X", "--method") and value.upper() != "GET":
            sys.exit(f"gh stub: only GET is answered, not {{value}}")
    elif arg == "--paginate":
        continue
    elif arg.startswith("-"):
        sys.exit(f"gh stub: unexpected flag {{arg}}")
    else:
        paths.append(arg)
if len(paths) != 1:
    sys.exit(f"gh stub: expected one endpoint, got {{paths!r}}")
with open(os.environ["GH_STUB_ANSWERS"], encoding="utf-8") as table:
    answers = json.load(table)
path = paths[0].lstrip("/")
if re.fullmatch(r"repos/[^/]+/[^/]+", path):
    print(json.dumps(answers["repo"]))
    sys.exit(0)
commit = re.fullmatch(r"repos/[^/]+/[^/]+/commits/([0-9a-f]{{7,40}})/pulls", path)
if commit is None:
    sys.exit(f"gh stub: no answer for endpoint {{path}}")
sha = commit.group(1)
failed = [full for full in answers["fail"] if full.startswith(sha)]
if failed:
    sys.exit(f"gh: {{answers['fail'][failed[0]]}} (HTTP 500)")
known = [full for full in answers["commits"] if full.startswith(sha)]
if len(known) != 1:
    sys.exit(f"gh: No commit found for SHA: {{sha}} (HTTP 422)")
print(json.dumps(answers["commits"][known[0]]))
'''


def pull_request(number, title, merge_commit, *, base="main", merged=True, url=None):
    """One pull request as `commits/<sha>/pulls` returns it."""
    return {
        "number": number,
        "title": title,
        "html_url": url or f"{REPOSITORY['html_url']}/pull/{number}",
        "merged_at": "2026-09-01T12:00:00Z" if merged else None,
        "base": {"ref": base, "repo": dict(REPOSITORY)},
        "merge_commit_sha": merge_commit,
        "state": "closed" if merged else "open",
    }


class ForgeRepository(GitFixture):
    """A repository whose base commit holds `files` and is tagged `tag`, a stub
    forge whose answers the test fills in per commit, and the release script run
    the way the release workflows run it: from the repository root."""

    def _repository(self, files: dict[str, str], tag: str | None = "v0.2.0") -> Path:
        scratch = tempfile.TemporaryDirectory()
        self.addCleanup(scratch.cleanup)
        self.scratch = Path(scratch.name)
        self.repo = self.scratch / "repo"
        self._git(self.scratch, "init", "-q", "-b", "main", str(self.repo))
        self.day = 0
        self.answers = {"repo": dict(REPOSITORY), "commits": {}, "fail": {}}
        self.base = self._land("The base", files)
        if tag is not None:
            self._git(self.repo, "tag", tag)
        return self.repo

    def _land(self, message: str, files: dict[str, str | None]) -> str:
        """Write each file, or delete it where the text is `None`, and commit the
        result on its own day, so every history is the same bytes."""
        for relative, text in files.items():
            path = self.repo / relative
            if text is None:
                path.unlink()
            else:
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(text.encode("utf-8"))
        self.day += 1
        when = f"2026-09-{self.day:02d}T12:00:00+00:00"
        identity = ("-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "-c", "commit.gpgsign=false")
        dated = {"GIT_AUTHOR_DATE": when, "GIT_COMMITTER_DATE": when, "GIT_CONFIG_GLOBAL": str(self.repo / ".absent")}
        for arguments in (("add", "-A"), (*identity, "commit", "-q", "--allow-empty", "-m", message)):
            subprocess.run(["git", "-C", str(self.repo), *arguments], check=True, capture_output=True, env=environment(dated))
        return self._git(self.repo, "rev-parse", "HEAD")

    def _answer(self, commit: str, *pulls: dict) -> None:
        self.answers["commits"][commit] = list(pulls)

    def _merged(self, number: int, title: str, *messages_and_files) -> list[str]:
        """Land one pull request as its commits, rebase-merged, and answer each of
        them with it; its merge commit is the last. Returns the commits."""
        commits = [self._land(message, files) for message, files in messages_and_files]
        for commit in commits:
            self._answer(commit, pull_request(number, title, commits[-1]))
        return commits

    def _bin(self, with_gh: bool) -> Path:
        """A `PATH` holding git and, unless told otherwise, the stub `gh` - and
        nothing else, so a `gh` installed on the machine cannot answer instead."""
        directory = self.scratch / ("bin" if with_gh else "bin-without-gh")
        if not directory.is_dir():
            directory.mkdir()
            git = shutil.which("git")
            assert git is not None, "the release tests need git"
            if sys.platform != "win32":  # on Windows git.exe stays where its siblings are
                os.symlink(git, directory / "git")
            if with_gh:
                stub = directory / "gh"
                stub.write_text(STUB.format(python=sys.executable), encoding="utf-8")
                stub.chmod(0o755)
                if sys.platform == "win32":
                    # `which` finds a .cmd through PATHEXT; a shebang means nothing here.
                    (directory / "gh.cmd").write_text(
                        f'@"{sys.executable}" "{stub}" %*\r\n', encoding="utf-8"
                    )
        return directory

    def _path(self, with_gh: bool) -> str:
        directory = self._bin(with_gh)
        if sys.platform == "win32":
            return os.pathsep.join([str(directory), str(Path(shutil.which("git")).parent)])
        return str(directory)

    def _script(self, *arguments: str, gh: bool = True, cwd: Path | None = None) -> CompletedProcess:
        table, self.log = self.scratch / "answers.json", self.scratch / "gh.log"
        table.write_text(json.dumps(self.answers), encoding="utf-8")
        return subprocess.run(
            [sys.executable, str(SCRIPT_PATH), *arguments],
            cwd=cwd or self.repo,
            capture_output=True,
            encoding="utf-8",
            errors="replace",
            env=environment(
                {
                    "PATH": self._path(gh),
                    "PYTHONUTF8": "1",
                    "GH_STUB_ANSWERS": str(table),
                    "GH_STUB_LOG": str(self.log),
                    "GIT_CONFIG_GLOBAL": str(self.scratch / ".gitconfig-absent"),
                    "GIT_CONFIG_NOSYSTEM": "1",
                }
            ),
        )

    def _read(self, relative: str = "docs/changelog.md") -> str:
        return (self.repo / relative).read_text(encoding="utf-8")

    def assertSucceeded(self, result: CompletedProcess) -> None:
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)

    def assertRefusedUntouched(self, result: CompletedProcess, *needles: str, repo: Path | None = None) -> None:
        """Refused, non-zero, naming its case, with every file as it was
        (§FS-distribution.4.6.3)."""
        repo = repo or self.repo
        self.assertNotEqual(0, result.returncode, result.stdout + result.stderr)
        for needle in needles:
            self.assertIn(needle, result.stderr)
        self.assertEqual("", self._git(repo, "status", "--porcelain", "--untracked-files=all"))
