# DF-kind-fields: a kind states its fields by section name, and a present optional field is still checked

**Status:** Accepted
**Date:** 2026-10-11

## 1. Context

The v2 format publishes `fields`, `closed` and `form` on a kind ([§DISC-core-concerns.7](../../discussions/proposals/2026-09-30-core-concerns.md#7-the-v2-format)), and agent-grounds/grund#458 builds them ([§FS-fields](../../functional-spec/FS-fields.md#fs-fields-a-v2-kind-states-the-shape-of-its-declarations-and-check-holds-every-declaration-to-it)). The design record fixes the spelling and leaves several choices to the ticket: how a field is matched, what `after` means when a predecessor is absent, which strength each violation takes, how `form` relates to v1's `values`, `rules` and `value_chapter`, and what happens where a field and an existing chapter-presence rule see the same defect ([§FS-rules.3.1](../../functional-spec/FS-rules.md#31-chapter-presence)). The author approved the proposal that settles them on agent-grounds/grund#458; this record keeps what each choice rejected.

## 2. Decision

### 2.1 A field is matched by its section name, and its title and values byte for byte

A field is the direct section whose name is the field's key, never one matched by its display name ([§FS-fields.identity.name](../../functional-spec/FS-fields.md#identityname-a-field-is-the-direct-section-of-its-name)). The coordinate then never moves when a status changes, which a citation of `DF-x.status` needs. `title` and `one_of` compare byte for byte ([§FS-fields.identity.match](../../functional-spec/FS-fields.md#identitymatch-titles-and-values-match-byte-for-byte)). Case-insensitive matching, which chapter rules use, was rejected: a status is data that scripts read from `grund show --format json`, and preventing three spellings of `Accepted` is why a closed list exists.

### 2.2 `after` means directly after, and an absent member is skipped

`after` places a field immediately after its predecessor, an absent member is skipped, and each field is judged against its nearest present predecessor so one error never cascades ([§FS-fields.order](../../functional-spec/FS-fields.md#order-order)). "Anywhere after" was rejected: with `placement` first, `after = "placement"` would then check nothing. Treating an absent required predecessor as a second error for every field behind it was rejected because it reports one omission as many.

### 2.3 `may` waives absence only

A present `may` field is held to its placement, title and values, as an error ([§FS-fields.strengths.may](../../functional-spec/FS-fields.md#strengthsmay-a-present-optional-field-is-still-held-to-its-shape)). Leaving a present optional field unchecked was rejected, because a `one_of` that nothing enforces is a declaration that lies. A separate strength key for shape was rejected because the published format has none; an author who wants the shape soft writes `presence = "warn"`.

### 2.4 The value chapter moves into the field list

A `content = "values"` field is v1's `value_chapter`, and v1's key lowers to a `may` field with nothing else, so the chapter has one place in the record and `form` carries no chapter of its own ([§FS-fields.content.values](../../functional-spec/FS-fields.md#contentvalues-a-values-field-is-the-value-chapter), [§FS-fields.forms](../../functional-spec/FS-fields.md#forms-form-lowers-to-the-version-1-contracts)). Mirroring the chapter both in `form` and in the field list was rejected: two records of one fact is the parallel interpretation path the ticket rules out. The Rust `Form` type loses its chapter payload; if that record shipped in a release before this change, the change is taken under the pre-1.0 licence of [§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise).

### 2.5 A field and a rule are paired at the merge, not deduplicated before evaluation

Where a field expresses a presence rule and both see the same absence in one declaration on the same channel, the field's finding stands and the rule's `chapter-cardinality` is dropped ([§FS-rules.6](../../functional-spec/FS-rules.md#6-semantic-deduplication)). Deduplicating the two constraints before evaluation, as `[citations]` and rules are, was rejected: a rule counts chapters by title and a field finds one by name, so dropping the rule up front would lose the defects only the rule sees, such as two chapters titled Terms under different names.

## 3. Consequences

A v1 file reads exactly as before. A v2 file that writes fields gets findings it asked for, so no existing verdict moves and no release note is owed. A repository that adds a field next to an existing presence rule sees the shared absence under the field's code instead of the rule's, and `docs/user-facing/rules.md` says so.
