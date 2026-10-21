"""Offline selection/preparation history for §FS-distribution.4.4.2.

Extends the shared forge fixture, including its cleanup policy, rather than
copying its git or gh setup. No command pushes or contacts a real forge.
"""

import subprocess
import sys

from changelog_gate_fixture import REPO_ROOT, environment
from release_forge_fixture import ForgeRepository

SELECTOR = REPO_ROOT / "scripts" / "release_version.py"
CANDIDATE = REPO_ROOT / "scripts" / "distribution" / "candidate.py"
GUARD = REPO_ROOT / "scripts" / "check_release_ramps.py"
VERSION_FIXTURE = "tests/e2e/cases/cli-version/expected.stdout"
MEMBERS = ("grund-core", "grund-cli", "grund-lsp", "grund-py")
CHANGELOG = """# Changelog

## 1. Conventions

The release lists its pull requests.

## 2. [0.21.3] — 2026-09-01

- Previous release.

## 3. Older releases

"""


def version_files(version):
    """A small complete version closure, including literal and inherited versions."""
    python = version.replace("-dev", ".dev0")
    members = ", ".join(f'"crates/{name}"' for name in MEMBERS)
    files = {
        "Cargo.toml": f'[workspace]\nmembers = [{members}]\n[workspace.package]\nversion = "{version}"\n',
        "Cargo.lock": "version = 3\n" + "".join(
            f'\n[[package]]\nname = "{name}"\nversion = "{version}"\n' for name in MEMBERS),
        "crates/grund-py/pyproject.toml": f'[project]\nname = "grund"\nversion = "{python}"\n',
        VERSION_FIXTURE: f"grund {version}\n",
    }
    for name in MEMBERS:
        own = 'version.workspace = true' if name == "grund-py" else f'version = "{version}"'
        files[f"crates/{name}/Cargo.toml"] = f'[package]\nname = "{name}"\n{own}\n'
        if name != "grund-core":
            files[f"crates/{name}/Cargo.toml"] += (
                f'[dependencies]\ngrund-core = {{ path = "../grund-core", version = "{version}" }}\n')
    return files


class SelectionRepository(ForgeRepository):
    def setUp(self):
        self._repository({**version_files("0.21.3"), "docs/changelog.md": CHANGELOG,
                          "scripts/release-floor.txt": "0.21.0\n"}, tag="v0.21.3")
        opener = self._land("Open 0.21.4-dev for development", version_files("0.21.4-dev"))
        self._answer(opener)  # A known direct commit, associated with no pull request.
        (self.tip,) = self._merged(804, "Allow the first stable release",
                                 ("Add a feature", {"crates/grund-core/src/lib.rs": "// A feature.\n"}))
        self._main_tip(self.tip)

    def _main_tip(self, sha):
        # A fetched remote-tracking ref, without a network or a remote writer.
        self._git(self.repo, "update-ref", "refs/remotes/origin/main", sha)

    def _run(self, script, *args):
        return subprocess.run([sys.executable, str(script), *args], cwd=self.repo,
                              capture_output=True, text=True, encoding="utf-8",
                              env=environment({"PATH": self._path(False), "PYTHONUTF8": "1"}))

    def _select(self, version="1.0.0", expected="tip", ref="refs/heads/main", dispatch="tip"):
        args = ["--ref", ref, "--dispatch-sha", self.tip if dispatch == "tip" else dispatch]
        if version is not None:
            args += ["--version", version]
        if expected is not None:
            args += ["--expected-main-sha", self.tip if expected == "tip" else expected]
        return self._run(SELECTOR, *args)

    def _outputs(self, result, next_version, current="0.21.3"):
        self.assertSucceeded(result)
        outputs = dict(line.split("=", 1) for line in result.stdout.splitlines())
        self.assertEqual({"current": current, "current_tag": f"v{current}",
                          "next": next_version, "next_tag": f"v{next_version}"}, outputs)
        return outputs

    def _snapshot(self):
        """Tracked/untracked bytes, refs and HEAD: refusal may change none of them."""
        files = {str(p.relative_to(self.repo)): p.read_bytes() for p in self.repo.rglob("*")
                 if p.is_file() and ".git" not in p.relative_to(self.repo).parts}
        return (files, self._git(self.repo, "show-ref"), self._git(self.repo, "rev-parse", "HEAD"))

    def _refused(self, before, result, *needles):
        self.assertNotEqual(0, result.returncode, result.stdout + result.stderr)
        self.assertNotIn("can't open file", result.stderr,
                         "a missing selector is not evidence of a policy refusal")
        self.assertNotIn("Traceback", result.stderr, "a crash is not a policy refusal")
        self.assertEqual("", result.stdout, "a refusal must not emit usable version outputs")
        for needle in needles:
            self.assertIn(needle, result.stderr)
        self.assertEqual(before, self._snapshot(), "selection changed files or refs on refusal")

    def _candidate(self, *args):
        result = self._run(CANDIDATE, *args, "--root", str(self.repo))
        self.assertSucceeded(result)
        return result.stdout.strip()

    def _prepare(self, outputs):
        """The existing preparation seam, driven by the real selector's outputs.

        No Cargo dependency resolution/build is needed for this closed fixture:
        candidate set-version rewrites its workspace lock entries as in production.
        Existing workflow tests pin cargo update and the candidate run/push sequence.
        """
        released = outputs["next"]
        self.assertSucceeded(self._run(GUARD, released, "--root", str(self.repo)))
        self._candidate("set-version", released)
        (self.repo / VERSION_FIXTURE).write_text(f"grund {released}\n", encoding="utf-8")
        self.assertSucceeded(self._script("prepare", released, "--date", "2026-10-21"))
        self._git(self.repo, "checkout", "-b", f"release-candidate/v{released}-804")
        release_sha = self._commit(self.repo, f"Release v{released}")
        major, minor, patch = map(int, released.split("."))
        dev = f"{major}.{minor}.{patch + 1}-dev"
        self._git(self.repo, "checkout", "-b", f"release-candidate/v{released}-804-dev")
        self._candidate("set-version", dev)
        (self.repo / VERSION_FIXTURE).write_text(f"grund {dev}\n", encoding="utf-8")
        dev_sha = self._commit(self.repo, f"Open {dev} for development")
        return release_sha, dev_sha
