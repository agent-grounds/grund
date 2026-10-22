---
type: Attested Computation
title: Gross profit for fiscal year
description: Gross profit by segment for a fiscal year, per the cost allocation standard.
tags: [finance, profit]
status: stable
runtime: bigquery
parameters:
  - { name: year, type: integer, required: true }
  - { name: segment, type: string, required: true }
executor:
  resource: /references/skills/run-on-bq.md
  receipt: [job_id, executed_sql, result]
attester:
  resource: /references/attesters/sql-equality.py
generated: { by: catalog-agent/1.4, at: 2026-06-14T14:00:00Z }
verified: { by: process:finance-nightly, at: 2026-06-12T08:00:00Z }
stale_after: 2026-06-15T00:00:00Z
sources:
  - id: cost-alloc
    resource: /policies/cost-allocation.md
    title: Cost allocation standard
---

# COMP-profit: Gross profit for fiscal year

Gross profit for one fiscal year and one segment: recognized revenue less the cost allocated to it.

## computation: Computation

```sql
SELECT SUM(amount) - SUM(allocated_cost) AS gross_profit
FROM finance.recognized_revenue
WHERE fiscal_year = @year
  AND segment = @segment
```

The query reads
[§TBL-recognized-revenue](../tables/recognized-revenue.md#tbl-recognized-revenue-recognized-revenue),
whose `allocated_cost` is the cost the allocation standard assigns to each line.[^cost-alloc]
It runs through
[§SKILL-run-on-bq](../references/skills/run-on-bq.md#skill-run-on-bq-run-an-attested-computation-on-bigquery),
like every computation in this bundle.

[^cost-alloc]: Cost allocation standard, [§POL-cost-allocation.1](../policies/cost-allocation.md#1-allocated-cost)
