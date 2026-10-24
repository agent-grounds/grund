# Risks and Technical Debts

- The door readers are old and fail in cold weather. A reader that is down
  takes no claims, so [§ADR-0003](../adr/0003-release-unclaimed-seats.md#adr-0003-release-an-unclaimed-seat-15-minutes-into-its-slot) releases seats that students are sitting in.
  [§ADR-0005](../adr/0005-claim-seats-from-the-phone.md#adr-0005-claim-a-seat-from-the-phone) proposes claiming from the phone instead.
- The pilot's bookings sit in SQLite until the one-off migration of [§ADR-0004](../adr/0004-store-bookings-in-postgresql.md#adr-0004-store-bookings-in-postgresql)
  has run.
