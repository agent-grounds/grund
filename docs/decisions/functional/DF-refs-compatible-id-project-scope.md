# DF-refs-compatible-id-project-scope: a refs path selects sites while the project resolves a compatible ID

**Date:** 2026-11-02

## status: Accepted

## scope: Context and decision

Issue #1126 reproduces a contradiction in the existing read contract:
`refs FS-BadHaskell` accepts a declared near miss, while the same query with a
file or directory path outside its declaration refuses it as `invalid-id`.
[§FS-config.3.2.6](../../functional-spec/FS-config.md#326-resolving-an-off-grammar-citation)
already requires resolving the declared token for compatibility, and
[§FS-scan.path.refs](../../functional-spec/FS-scan.md#pathrefs-refs-scans-the-paths-tree)
makes the path select sites. Resolve against the selected project's catalog in
every case, retaining exactly the sites the path selects. This applies at the
shared API boundary and therefore to the CLI, Python and Node.

## compatibility: Compatibility

This corrects an erroneous query refusal; no repository needs an edit and no
passing repository becomes failing. The read contract already promises success
under [§FS-config.3.2.6](../../functional-spec/FS-config.md#326-resolving-an-off-grammar-citation).
There is no old form to retire through the deprecation path
([§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path))
and nothing for a mechanical migration to rewrite
([§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations)).
This decision does not invoke section 5 of
[§REQ-backwards-compatibility](../../requirements/REQ-backwards-compatibility.md#req-backwards-compatibility-an-upgrade-never-changes-a-verdict-quietly):
it restores a promised read rather than introducing a failing repository
verdict. The release note below records the observable exit change. No finding
replaces the refusal: the query succeeds. Undeclared malformed operands still
fail; another project's declaration cannot authorize one. Authoring grammar and
`check`'s near-miss finding keep their existing rules.

## release-note: Release note

- [§FS-scan.path.refs](../../functional-spec/FS-scan.md#pathrefs-refs-scans-the-paths-tree): `refs` now accepts a declared off-grammar ID for read compatibility even when its explicit file or directory path excludes the declaration. Such queries change from exit `1` with `invalid-id` to exit `0` with only the selected citation sites, possibly none. Python and Node calls return the corresponding successful result. Callers that relied on the erroneous refusal should consume the successful result; no repository migration is needed. Undeclared malformed IDs remain refused. Corrects #1126.
