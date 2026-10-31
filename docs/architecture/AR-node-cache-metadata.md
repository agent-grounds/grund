# AR-node-cache-metadata: Offline publication resolves the features actually built

The dependency-only Node publication of [§AR-ci.3.4.1.5](AR-ci.md#3415-a-dependency-only-publication-view-with-the-live-root-intact) needs package metadata
without fetching unused optional source archives. The source-build rehearsal
of [§FS-distribution.3.2.3.3](../functional-spec/FS-distribution.md#3233-source-buildlocal-package-ownership) need not build every optional workspace feature.

## placement: Between live targets and publication selection

```text
owner checkout, live units ─► [ offline metadata ] ─► source-qualified snapshot selection
```

`scripts/node_cache_publication.py` reads each live target's owner checkout and
compiled units before selecting the snapshot. This point specifies its metadata
inputs; [§AR-ci.3.4.1.5](AR-ci.md#3415-a-dependency-only-publication-view-with-the-live-root-intact) continues to define artifact identity, publication contents,
ownership, restore semantics and failure tolerance. This is CI infrastructure
outside the shipped engine ([§AR-system.5](README.md#5-what-holds-the-shape)); it reads Cargo inputs and does not
change the source-build rehearsal.

## terms: Terms

Leans on [§FS-terms.terms.8](../functional-spec/FS-terms.md#terms8-the-architectures-own-words) (meter) and [§FS-repository-maintenance.terms](../functional-spec/FS-repository-maintenance.md#terms-terms) (checkout root).

An **observed feature** is a workspace feature in a compiled unit's Cargo
fingerprint, associated with that workspace package by its exact target source
input. An **unused optional source** has a lock/index entry but no downloaded
archive because the gate has not needed to compile it.

## resolution: Locked and offline, with observed workspace features

First read workspace metadata without dependencies, using `--locked --offline`.
Join compiled units to its opaque package IDs by the recorded target source
inputs, using the same unit identity mechanism as artifact selection. Read those
units' feature fingerprints across profiles and nested targets. Resolve full
package metadata with default features plus the observed features still declared
by their workspace packages, for the host and every built platform.

Do not activate unused optional features just to classify artifacts. In
particular, the Node gate need not fetch the CLI's unused benchmark dependency
chain. Keep `--locked --offline` on both metadata reads. The final resolved Cargo
package IDs and source locations remain the artifact classifier's inputs;
feature selection does not introduce a package-name ownership heuristic.

## failures: Cargo diagnostics explain an unready publication

If metadata fails, include Cargo's stderr, exit status, command and owner checkout
in the warning. Escape workflow annotation data so multiline diagnostics remain
visible without becoming workflow commands. Preserve the failure behavior of
[§AR-ci.3.4.1.5](AR-ci.md#3415-a-dependency-only-publication-view-with-the-live-root-intact): live bytes survive, the previous snapshot is retired and no ready
output is emitted for an incomplete publication.

## regression: An unused archive stays absent and an enabled dependency stays warm

The real-Cargo publication fixtures in `test_node_binding_acceptance.py` include
an indexed, locked optional registry package without a source archive and a
compiled, feature-enabled local dependency. Publication must become ready without
acquiring the unused source, retain the enabled dependency and reuse it after
restore. Existing source collisions, stale workspace exclusion, live-byte and
ownership assertions remain in place at both workflow copies and toolchains.
Making the absent package required must then warn with Cargo's diagnostic, retire
the saved view and leave live bytes intact. These checks remain in the existing
Node selection; their cost counts in [§AR-ci.requirements.1.verification](AR-ci.md#requirements1verification-a-cost-correction-measures-comparable-full-gates).
