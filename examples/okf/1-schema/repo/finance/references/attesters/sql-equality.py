"""SQL-equality attester for the computations §SKILL-run-on-bq runs.

It reads the receipt §SKILL-run-on-bq.receipt describes and returns a verdict
without asking a model: a run is attested when the SQL the job executed is the
computation's own SQL and the value about to be shown is the job's result.
"""


def canonical(sql: str) -> str:
    """Drop `--` comments and fold whitespace, leaving literals as written."""
    lines = (line.split("--", 1)[0] for line in sql.splitlines())
    return " ".join(" ".join(lines).split())


def attest(computation_sql: str, receipt: dict, displayed: object) -> dict:
    """Return the verdict for one run of a computation."""
    if canonical(receipt["executed_sql"]) != canonical(computation_sql):
        return {"verdict": "fail", "why": "the job did not run the sanctioned SQL"}
    if receipt["result"] != displayed:
        return {"verdict": "fail", "why": "the value shown is not the job's result"}
    return {"verdict": "ok", "job_id": receipt["job_id"]}
