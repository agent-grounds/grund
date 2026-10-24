---
type: Metric
title: Income statement (fiscal year)
description: Headline income-statement figures for a fiscal year.
tags: [finance, income-statement]
status: stable
generated: { by: catalog-agent/1.4, at: 2026-06-20T22:53:05Z }
verified: { by: human:jdoe, at: 2026-06-25T09:00:00Z }
stale_after: 2026-12-31T00:00:00Z
sources:
  - id: fpa-handbook
    resource: https://wiki.acme.example/finance/fpa-handbook
    title: FP&A reporting handbook
---

# MET-income-statement: Income statement (fiscal year)

The income statement reports revenue and gross profit for a fiscal year.

## definition: Definition

The statement reports revenue,
[§COMP-revenue](../computations/revenue.md#comp-revenue-revenue-for-fiscal-year),
and gross profit,
[§COMP-profit](../computations/profit.md#comp-profit-gross-profit-for-fiscal-year),
for one fiscal year, as the FP&A reporting handbook lays it out.[^fpa-handbook]
Each figure is produced by its own attested computation. This concept only
narrates them, so a figure's trust is the trust of the computation it links.

[^fpa-handbook]: FP&A reporting handbook
