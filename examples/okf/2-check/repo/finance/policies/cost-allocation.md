---
type: Policy
title: Cost allocation standard
description: Which costs a revenue line carries, and how segments are drawn.
resource: https://wiki.acme.example/finance/cost-allocation
tags: [finance, policy, cost]
status: stable
generated: { by: human:jdoe, at: 2026-02-01T10:00:00Z }
verified: { by: human:jdoe, at: 2026-06-10T09:00:00Z }
---

# POL-cost-allocation: Cost allocation standard

Every recognized line carries the full cost of delivering it, so gross profit
can be reconciled to the general ledger.

## 1. Allocated cost

A line's allocated cost is its product cost plus its share of fulfillment,
outbound shipping and payment fees, allocated monthly.

## 2. Segments

A segment is the product category a line's item belongs to on the day it was
sold. Its cost falls in the fiscal year its revenue does, which is the year the
revenue policy books it to:
[§POL-revenue-recognition.3](revenue-recognition.md#3-fiscal-year).
