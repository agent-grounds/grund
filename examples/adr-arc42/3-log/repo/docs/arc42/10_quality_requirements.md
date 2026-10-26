# Quality Requirements

## Quality Requirements Overview

| Quality | Requirement |
|---|---|
| Reliability | no seat is ever held by two bookings ([§QG-1](01_introduction_and_goals.md#qg-1-a-seat-is-never-booked-twice)) |
| Usability | a booking on a phone takes under a minute ([§QG-2](01_introduction_and_goals.md#qg-2-a-booking-takes-under-a-minute-on-a-phone)) |
| Operability | one part-time administrator runs it ([§QG-3](01_introduction_and_goals.md#qg-3-one-part-time-administrator-can-run-it)) |
| Accessibility | the booking pages meet WCAG 2.2 at level AA |

## Quality Scenarios

Each scenario names its stimulus, what Carrel does in response, and the measure
that decides whether it held. Every quality goal has at least one
([§RULE-goals-measured](../rules.md#rule-goals-measured-each-qg-must-be-cited-by-at-least-one-qs)).

### QS-1: Two students book one seat at once

Makes [§QG-1](01_introduction_and_goals.md#qg-1-a-seat-is-never-booked-twice) measurable.

- **Stimulus:** two students book seat 12 of the east room for the 14:00 slot
  in the same second, through different instances.
- **Response:** one booking is stored, and the other student is offered the
  nearest free seat.
- **Measure:** no double booking in a test of 1,000 simultaneous pairs.

### QS-2: A booking on the walk to the library

Makes [§QG-2](01_introduction_and_goals.md#qg-2-a-booking-takes-under-a-minute-on-a-phone) measurable.

- **Stimulus:** a student on a mobile connection opens Carrel and books the
  first free seat.
- **Response:** the booking is confirmed.
- **Measure:** under 60 seconds from opening the page, over a 3G connection.

### QS-3: A claim at the last moment

Makes [§QG-1](01_introduction_and_goals.md#qg-1-a-seat-is-never-booked-twice) measurable.

- **Stimulus:** a student taps their card 14 minutes and 59 seconds into the
  slot, while the release job runs.
- **Response:** the claim wins, and the seat is not released.
- **Measure:** no claimed seat is released in a test of 1,000 such races.

### QS-4: The database machine is lost overnight

Makes [§QG-3](01_introduction_and_goals.md#qg-3-one-part-time-administrator-can-run-it) measurable.

- **Stimulus:** `carrel-db` is gone in the morning.
- **Response:** the administrator restores the nightly dump on a new machine.
- **Measure:** the rooms reopen within one working hour, following one page of
  instructions.
