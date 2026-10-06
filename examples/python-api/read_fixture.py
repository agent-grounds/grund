"""Runnable check, iteration and show (§FS-distribution.3.3.7)."""

from pathlib import Path
from grund import check, show

repo = Path(__file__).resolve().parents[2] / "tests/e2e/cases/json-report/repo"
result = check(repo)
assert [(finding.code, finding.line) for finding in result.report] == [("dangling", 3)]
for finding in result.report:
    print(finding.code, finding.line)
assert "FS-999-missing" in show("FS-001-alpha", root=repo, mode="brief").body
