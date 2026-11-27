# DF-closed-citation-grounding-warns: Warn when grounding asks for citations a closed home permits nowhere

**Date:** 2026-11-27

## status: Accepted

## context: Context

A scanned non-citable home closed to every citation ([§FS-config.3.9.4.1](../../functional-spec/FS-config.md#3941-a-row-closed-to-every-citation)) can still ask for grounding. That setting cannot be satisfied: a citation breaks the direction rule, and an inline declaration cannot ground a file in this home ([§FS-check.3.6.2](../../functional-spec/FS-check.md#362-the-unit)). For `must`, every file fails `check`; `warn` and `should` keep their softer channels. The config loads without explaining the conflict.

## decision: Decision

Emit the permanent `closed-citation-grounding` warning defined by [§FS-check.checks.closed-citation-grounding](../../functional-spec/FS-check.md#checksclosed-citation-grounding-grounding-asks-for-citations-a-closed-row-permits-nowhere), naming both effective settings, their provenance and both repairs. Read the existing closed-row and grounding predicates. Keep loading, checker findings, managed-block bytes and versions, and exit codes unchanged. Config-grounded rows retain config provenance under the flag. This follows the warning without a deadline in [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path).

## alternatives: Rejected alternatives

- Refusing the config at load invalidates configurations that load today and cannot describe the run's grounding flag.
- Rewording `ungrounded` changes stable message bytes ([§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered)), and still leaves the maintainer diagnosing the conflict one file at a time.

## consequences: Consequences

`check`, `config validate` and `config show` gain stderr warnings; an empty closed home can warn in place of `success`. Migration compares config path, row, strength and unit across version-specific wording ([§FS-config-migrate.5.2](../../functional-spec/FS-config-migrate.md#52-the-report-gate)). Queries and `init` stay silent. No Citation directions bullet or managed-block version changes.

## release-note: Release note

- [§FS-check.checks.closed-citation-grounding](../../functional-spec/FS-check.md#checksclosed-citation-grounding-grounding-asks-for-citations-a-closed-row-permits-nowhere): **Grounding on a closed non-citable home is cautioned.** `grund check`, `grund config validate` and `grund config show` now print one `closed-citation-grounding` warning per scanned non-citable row closed to all citation targets while any grounding rung reaches it, naming both settings and how to repair either. JSON check adds a structured stderr warning with null `path`, `line` and `sites`. For an empty home this replaces `success`; all exit codes and existing findings stay unchanged. Scripts expecting silent stderr or `success` can use `check --ignore closed-citation-grounding`; validate and show have no selector. The warning never becomes an error. Migration preserves the warning's facts across versions.
