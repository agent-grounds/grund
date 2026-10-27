# DOC-solver: Solver

Each solver step advances time by `1.0e-3 s` (§CONST-time-step).

This adjacent literal is deliberately not bound: 0.001 (§CONST-time-step.1).

The stability analysis still assumes `0.002 s` (§CONST-time-step).

## 1. Courant limit <!-- grund:value -->
### 1.1. 0.5

## 2. Embedded use

Each step damps velocity by `0.020` (§CONST-damping.1).

The substep cap is `8` (§CONST-max-substeps.1), and the integration scheme is
`velocity-verlet` (§CONST-integrator.1).

The independently marked Courant limit agrees: `0.50` (§DOC-solver.1.1).

Bound at its root, the same limit is one literal: `0.50` (§DOC-solver.1).

The chapter-declared gravity agrees too: `9.81` (§DOC-solver.values.gravity.1).

As one quantity, amount and unit together: `9.81 m/s²` (§DOC-solver.values.gravity).

## values: Values

### values.gravity: Gravitational acceleration

#### values.gravity.1: 9.81
#### values.gravity.2: m/s²
