# Runtime View

## Booking a seat

1. The student picks a seat and a slot; the Booking API inserts the booking.
2. If another instance inserted one for the same seat and slot first, the
   unique index refuses the second insert, and the student is offered the
   nearest free seat ([§QS-1](10_quality_requirements.md#qs-1-two-students-book-one-seat-at-once)).

## Releasing an unclaimed seat

Every minute the release job cancels each booking whose slot began more than
15 minutes ago and has no claim ([§QS-3](10_quality_requirements.md#qs-3-a-claim-at-the-last-moment)).
