# Requirements

# FS-reserve: An order reserves stock before payment

Serves [§GOAL-no-oversell](docs/goals.md#goal-no-oversell-we-never-sell-stock-we-do-not-have).

## 1. Hold time

A reservation holds its items for fifteen minutes.

# FS-restock: A delivery restocks its items

A delivery adds its items back to stock.

## 1. Waiting reservations

Restocking releases waiting reservations first ([§FS-reserve.1](requirements.md#1-hold-time)).
