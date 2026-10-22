---
type: Attested Computation
title: Revenue for fiscal year
description: Recognized revenue for a fiscal year, per Finance's definition.
tags: [finance, revenue]
status: stable
runtime: bigquery
parameters:
  - { name: year, type: integer, required: true }
executor:
  resource: /references/skills/run-on-bq.md
  receipt: [job_id, executed_sql, result]
attester:
  resource: /references/attesters/sql-equality.py
generated: { by: catalog-agent/1.4, at: 2026-06-28T14:00:00Z }
verified: { by: human:jdoe, at: 2026-06-25T09:00:00Z }
stale_after: 2026-12-31T00:00:00Z
sources:
  - id: rev-policy
    resource: /policies/revenue-recognition.md
    title: Revenue recognition policy
    author: team:finance-fpa
    last_modified: 2026-04-02T00:00:00Z
  - id: exec-rev-dash
    resource: https://dashboards.acme.example/exec-revenue
    title: Executive revenue dashboard
    author: team:finance-fpa
    usage_count: 5000
    last_modified: 2026-06-18T00:00:00Z
usage_window: { from: 2026-06-01T00:00:00Z, to: 2026-06-30T00:00:00Z }
---

# COMP-revenue: Revenue for fiscal year

Recognized revenue for one fiscal year. The year is the only value a caller binds.

## computation: Computation

```sql
SELECT SUM(amount) AS revenue
FROM finance.recognized_revenue
WHERE fiscal_year = @year
```

The query reads
[§TBL-recognized-revenue](../tables/recognized-revenue.md#tbl-recognized-revenue-recognized-revenue),
which holds a line only once the recognition policy counts it as revenue.[^rev-policy]
It runs through
[§SKILL-run-on-bq](../references/skills/run-on-bq.md#skill-run-on-bq-run-an-attested-computation-on-bigquery),
and the SQL-equality attester compares the receipt's `executed_sql` with this
fence. The executive revenue dashboard reports the same figure.[^exec-rev-dash]

[^rev-policy]: Revenue recognition policy, [§POL-revenue-recognition.1](../policies/revenue-recognition.md#1-recognition)
[^exec-rev-dash]: Executive revenue dashboard
