# ADR-0002: Store bookings in SQLite

Date: 2026-01-19

## status: Superseded

Superseded by [§ADR-0004](0004-store-bookings-in-postgresql.md#adr-0004-store-bookings-in-postgresql).

## context: Context

The pilot covers one reading room and runs as one instance on one virtual
machine.

## decision: Decision

We keep bookings in a SQLite file beside the application.

## consequences: Consequences

There is no database server to run, and a backup is a copy of one file. A
second instance cannot share the file, so a restart closes the room.
