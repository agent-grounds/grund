# ADR-0004: Store bookings in PostgreSQL

Date: 2026-03-09

## status: Accepted

## context: Context

Carrel now serves four rooms, and a restart that closes them all is no longer
acceptable. [§ADR-0002](0002-store-bookings-in-sqlite.md#adr-0002-store-bookings-in-sqlite) kept bookings in a SQLite file that only one instance can
use.

## decision: Decision

We keep bookings in PostgreSQL on its own machine, shared by two application
instances. A unique index on seat and slot refuses a second booking of one seat
([§QG-1](../arc42/01_introduction_and_goals.md#qg-1-a-seat-is-never-booked-twice), [§QS-1](../arc42/10_quality_requirements.md#qs-1-two-students-book-one-seat-at-once)). A one-off script moves the pilot's bookings across.

## consequences: Consequences

A double booking is impossible whichever instance takes it. There is one more
machine to patch and back up ([§QS-4](../arc42/10_quality_requirements.md#qs-4-the-database-machine-is-lost-overnight)).
