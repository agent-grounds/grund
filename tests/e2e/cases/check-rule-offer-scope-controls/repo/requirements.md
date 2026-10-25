# Requirements

## FS-store: A user stores a secret

### requirements: Requirements

The secret is encrypted before it is written (§REQ-no-plaintext).

### design: Design

One row per secret version.

## FS-rotate: A user rotates a secret

### requirements: Requirements

Old versions stay readable for seven days; nothing is cited here.

## FS-search: A user searches their secrets by name

### requirements: Requirements

Search never matches on secret values, only on names.

### notes: Notes

Indexing names does not touch §REQ-no-plaintext; this citation is outside the requirements chapter.

## FS-audit: Admins read the access log

Every read of a secret is listed with the reader and the time, which meets
§REQ-audit-trail. FS-audit has no requirements chapter.

### design: Design

An append-only table, partitioned by month.
