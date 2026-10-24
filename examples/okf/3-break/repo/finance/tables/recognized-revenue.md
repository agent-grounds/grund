---
type: BigQuery Table
title: Recognized revenue
description: One row per recognized revenue line, with the cost allocated to it.
resource: https://console.cloud.google.com/bigquery?p=acme&d=finance&t=recognized_revenue
tags: [finance, revenue, warehouse]
generated: { by: catalog-agent/1.4, at: 2026-06-20T22:53:05Z }
verified: { by: process:schema-sync, at: 2026-06-30T02:00:00Z }
sources:
  - id: warehouse-schema
    resource: https://wiki.acme.example/data/finance-schema
    title: Finance warehouse schema
---

# TBL-recognized-revenue: Recognized revenue

One row per order line, written once the line is recognized as revenue.

## schema: Schema

| Column | Type | Description |
|---|---|---|
| `line_id` | STRING | The order line, unique across channels.[^warehouse-schema] |
| `fiscal_year` | INTEGER | The fiscal year the line is recognized in. |
| `segment` | STRING | The product category, as of the sale. |
| `amount` | NUMERIC | Net amount in US dollars; no shipping, no tax. |
| `allocated_cost` | NUMERIC | Full cost of the line, in US dollars. |

## examples: Examples

```sql
SELECT segment, SUM(amount) AS revenue
FROM finance.recognized_revenue
WHERE fiscal_year = 2026
GROUP BY segment
```

[^warehouse-schema]: Finance warehouse schema
