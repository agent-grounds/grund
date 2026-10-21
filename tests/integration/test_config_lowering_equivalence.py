"""§AR-config.4 — moving the v1 reader and lowering it into the records changes
no byte a user sees. For every project under `tests/e2e/cases/` and `examples/`
that carries a `grund.toml` (bare or under `.agents/`), `grund config show` and
`grund config validate` are run from the repository root, and their exit code,
their stdout and their stderr are held to the captures in
`config_lowering_captures.jsonl`.

The captures were taken from the binary built at the commit that specified
agent-grounds/grund#453, before any reader code moved. stdout is held by its
SHA-256 (the first 16 hex digits), because the effective configs together run to
megabytes. stderr is held verbatim, because it is where a config error's text and
its `path:line` anchor live, which is the seam a move of validation is most
likely to break (§FS-config.4.3).

Issue #817's wording-only exception changes only the final repair clause of
validate[2] for the two v2 records
tests/e2e/cases/workspace-include-root-false-unread-check/repo and
tests/e2e/cases/workspace-include-root-false-unread-check/repo/group, as
§FS-check.4.10.5 requires. The owner-authorized #817 follow-up applies the same
version-neutral clause to the 25 v1 records at capture lines 933, 1010-1011,
1013-1014, 1021, 1025, 1027-1028, 1030-1031, 1033-1034, 1051, 1053-1054,
1187-1188, 1190-1191, 1196, 1200, 1204, 1208 and 1212. Only those unread-workspace
repair clauses in validate[2] change; all other captured bytes, statuses and
digests remain the #453 reference.

Issue #881 ends the one-grammar window (§RM-one-grammar-flip), after which the
canonical config of §DF-directions-render.3 no longer loads: it pinned `api/AR`
in a project with no `[workspace]`. The fixture changed rather than the reader,
gaining a `[workspace]` whose member `api` configures `AR`, so the three records
of that config, at capture lines 34, 549 and 550, hold its show digest and an
empty validate stderr. Those bytes are the same before and after the flip.

The captures are not regenerated to make this test pass. `--capture` exists to
take them, once, from a binary whose behaviour is the reference.
"""

import concurrent.futures
import functools
import hashlib
import json
import os
import re
import subprocess
import sys
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CAPTURES = Path(__file__).resolve().parent / "config_lowering_captures.jsonl"
TREES = ("tests/e2e/cases", "examples")
COMMANDS = ("show", "validate")
EXE = "grund.exe" if os.name == "nt" else "grund"


@functools.lru_cache(maxsize=None)
def _grund():
    """The binary under test, built from this checkout once per process."""
    target = Path(os.environ.get("CARGO_TARGET_DIR") or REPO_ROOT / "target")
    cargo = os.environ.get("CARGO", "cargo")
    subprocess.run([cargo, "build", "-p", "grund", "--locked"], cwd=REPO_ROOT, check=True)
    built = target / "debug" / EXE
    if not built.is_file():
        raise AssertionError(f"no grund binary at {built} after cargo build -p grund")
    return str(built)


def projects():
    """Every project directory a fixture config governs, repository-relative."""
    found = set()
    for tree in TREES:
        for config in (REPO_ROOT / tree).rglob("grund.toml"):
            project = config.parent.parent if config.parent.name == ".agents" else config.parent
            found.add(project.relative_to(REPO_ROOT).as_posix())
    return sorted(found)


# The version 1 caution (§FS-check.checks.config-version-1) came after the captures and
# is not the lowering's; it is dropped from what is compared.
CAUTION = re.compile(
    r"^warning: \S+ is version 1; migrate it with grund config migrate --to 2( \S+)?\n", re.M
)


def _run(project, command):
    result = subprocess.run(
        [_grund(), "config", command, project],
        cwd=REPO_ROOT,
        capture_output=True,
    )
    return [
        result.returncode,
        hashlib.sha256(result.stdout).hexdigest()[:16],
        CAUTION.sub("", result.stderr.decode("utf-8", "replace")),
    ]


def observe():
    """`project -> {command: [exit, stdout digest, stderr]}` for the tree as it is."""
    names = projects()
    _grund()
    jobs = [(project, command) for project in names for command in COMMANDS]
    with concurrent.futures.ThreadPoolExecutor(os.cpu_count() or 4) as pool:
        results = list(pool.map(lambda job: _run(*job), jobs))
    observed = {project: {} for project in names}
    for (project, command), result in zip(jobs, results):
        observed[project][command] = result
    return observed


def captured():
    observed = {}
    for line in CAPTURES.read_text(encoding="utf-8").splitlines():
        if line:
            row = json.loads(line)
            observed[row["project"]] = {command: row[command] for command in COMMANDS}
    return observed


def write_captures():
    lines = [
        json.dumps({"project": project, **results}, ensure_ascii=False, separators=(",", ":"))
        for project, results in sorted(observe().items())
    ]
    CAPTURES.write_text("\n".join(lines) + "\n", encoding="utf-8")


# The captures were taken on Linux. The lowering is the same code everywhere,
# but a fixture's symlinks and case-folding are not, so the bytes are held where
# they were taken.
@unittest.skipUnless(sys.platform.startswith("linux"), "captures are Linux bytes")
class ConfigLoweringEquivalenceTests(unittest.TestCase):
    maxDiff = None

    def test_every_captured_project_is_still_a_fixture(self):
        """A capture whose fixture has gone holds nothing. A fixture added after
        the captures were taken is not held here; its own e2e case holds it."""
        self.assertGreater(len(captured()), 1000, "the captures no longer parse")
        self.assertEqual([], sorted(set(captured()) - set(projects())))

    def test_config_show_and_validate_print_the_captured_bytes(self):
        expected = captured()
        moved = []
        for project, results in sorted(observe().items()):
            for command in COMMANDS:
                if project in expected and results[command] != expected[project][command]:
                    was, now = expected[project][command], results[command]
                    moved.append(f"grund config {command} {project}: was {was!r}, now {now!r}")
        self.assertEqual([], moved)


if __name__ == "__main__":
    if sys.argv[1:] == ["--capture"]:
        write_captures()
    else:
        unittest.main()
