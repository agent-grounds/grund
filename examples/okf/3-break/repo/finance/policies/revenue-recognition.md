---
type: Policy
title: Revenue recognition policy
description: When a sale counts as revenue, and in which fiscal year.
resource: https://wiki.acme.example/finance/revenue-recognition
tags: [finance, policy, revenue]
status: stable
generated: { by: human:jdoe, at: 2026-04-02T00:00:00Z }
verified: { by: human:jdoe, at: 2026-06-25T09:00:00Z }
---

# POL-revenue-recognition: Revenue recognition policy

A sale is revenue once it is delivered and can no longer be returned, and it is
booked to the fiscal year in which that happens.

## 1. Recognition

A line is recognized when its order is delivered and its 30-day return window
has closed.

## 2. Amount

The recognized amount is the line's net amount. Shipping and tax are collected
for others and are never revenue.

## 3. Fiscal year

A line is booked to the fiscal year containing the day it is recognized, which
is the day its return window closes.
