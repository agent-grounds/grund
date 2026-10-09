# REFLECTION-jvm-reflection: JVM reflection

Argus resolves a reflective call only where the Lean engine (argus.spec.md §5.1) can establish its target.

## 1. Scope

The call graph (argus.spec.md §4.5) and the system model (system-model.md §2.1) bound what is resolved.

## 2. Limits

A target outside the closed world is reported, as graph-analysis.md §10 requires; see §2 above.
