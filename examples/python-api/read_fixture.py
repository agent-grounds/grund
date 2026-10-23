"""Runnable check, iteration and show (§FS-distribution.3.3.7)."""

from pathlib import Path
from grund import check, show

repo = Path(__file__).resolve().parents[2] / "tests/e2e/cases/json-report/repo"
result = check(repo)
# Read errors independently of the version-1 caution (§FS-check.checks.config-version-1).
assert [(finding.code, finding.line) for finding in result.report.errors] == [("dangling", 3)]
for finding in result.report.errors:
    print(finding.code, finding.line)
assert "FS-999-missing" in show("FS-001-alpha", root=repo, mode="brief").body

# §FS-errors.4.1: the pathless caution precedes the located warning.
warning_repo = repo.parent.parent / "cross-project-citation-without-workspace/repo"
warning_codes = [finding.code for finding in check(warning_repo).report.warnings]
assert warning_codes == ["config-version-1", "unused"]
print("warnings: " + ", ".join(warning_codes))
