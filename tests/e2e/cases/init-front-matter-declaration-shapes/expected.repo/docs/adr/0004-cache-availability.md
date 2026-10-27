---
status: acepted
date: 2026-02-03
decision-makers: [Bo Lind]
reviewer: Ana Ruiz
---

# ADR-0004-cache-availability: Cache room availability in Redis

## context: Context and Problem Statement

The availability page queries every booking on each load.

## decision: Decision Outcome

Each room's availability is cached in Redis for 60 seconds.
