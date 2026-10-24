# ADR-0001: Record architecture decisions

Date: 2026-01-12

## status: Accepted

## context: Context

Carrel's first decisions were made in a hallway, and the people who made them
will not all be here next year.

## decision: Decision

We record each architecturally significant decision as an architecture decision
record, in Michael Nygard's form, one numbered file per record in `docs/adr/`.
A record is never deleted: a reversed decision is marked superseded and names
the record that replaces it.

## consequences: Consequences

A newcomer reads the records in order to learn why Carrel is built the way it
is. Writing one costs a quarter of an hour.
