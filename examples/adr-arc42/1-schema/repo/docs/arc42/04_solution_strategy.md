# Solution Strategy

- One Python web service, run as two instances behind the load balancer, so a
  restart never closes the rooms ([§QG-3](01_introduction_and_goals.md#qg-3-one-part-time-administrator-can-run-it)).
- Bookings live in PostgreSQL, whose unique index on seat and slot is what
  makes a double booking impossible ([§ADR-0004](../adr/0004-store-bookings-in-postgresql.md#adr-0004-store-bookings-in-postgresql), [§QG-1](01_introduction_and_goals.md#qg-1-a-seat-is-never-booked-twice)).
- A seat nobody claims goes back on offer 15 minutes into its slot ([§ADR-0003](../adr/0003-release-unclaimed-seats.md#adr-0003-release-an-unclaimed-seat-15-minutes-into-its-slot)).
- Pages are rendered on the server and kept small, so a booking is fast on a
  phone ([§QS-2](10_quality_requirements.md#qs-2-a-booking-on-the-walk-to-the-library)).
- Each decision is an architecture decision record in `docs/adr/` ([§ADR-0001](../adr/0001-record-architecture-decisions.md#adr-0001-record-architecture-decisions)),
  and chapter 9 lists them all.
