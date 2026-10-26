# ADR-0003: Release an unclaimed seat 15 minutes into its slot

Date: 2026-02-02

## status: Accepted

## context: Context

In the pilot a third of the bookings were never claimed, and the seats stood
empty while students queued at the desk.

## decision: Decision

A booking with no claim 15 minutes after its slot begins is cancelled, and the
seat is offered again. A claim made inside the 15 minutes always wins over the
release ([§QS-3](../arc42/10_quality_requirements.md#qs-3-a-claim-at-the-last-moment)).

## consequences: Consequences

Seats are used. A student who arrives late loses the seat, and is mailed to say
so.
