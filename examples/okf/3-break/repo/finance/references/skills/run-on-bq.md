---
type: Skill
title: Run an attested computation on BigQuery
description: Binds a computation's parameters, runs the job, and returns its receipt.
tags: [skill, executor, bigquery]
status: stable
generated: { by: human:jdoe, at: 2026-06-20T10:00:00Z }
---

# SKILL-run-on-bq: Run an attested computation on BigQuery

The executor of every computation in this bundle whose `runtime` is `bigquery`.

## steps: Steps

1. Read the SQL fence under the computation's `Computation` heading, or the file
   its `computation` key names.
2. Bind each value the caller supplied as a named query parameter. Never write a
   value into the SQL.
3. Run the job as standard SQL and wait for it to finish.
4. Return the receipt. Never edit the SQL to make a failing run succeed.

## receipt: Receipt

| Field | What it holds |
|---|---|
| `job_id` | The BigQuery job that ran. |
| `executed_sql` | The SQL the job ran, its parameters still written as `@name`. |
| `result` | The first row of the result. |
