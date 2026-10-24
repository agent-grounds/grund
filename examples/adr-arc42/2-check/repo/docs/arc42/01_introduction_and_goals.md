# Introduction and Goals

Carrel books seats in the four reading rooms of the Merrowfield University
Library. A student picks a seat and a two-hour slot from a phone or a library
terminal, claims it by tapping a card at the room's door, and loses it to the
next student if they never arrive.

## Requirements Overview

- A student books, moves or cancels a seat for any slot in the next seven days.
- A seat that is booked but not claimed goes back on offer.
- Reading-room staff see each room's occupancy and can close a room for an
  event, which cancels its bookings and tells the students holding them.

## Quality Goals

### QG-1: A seat is never booked twice

Two students arriving with a booking for one seat is the failure the desk
cannot repair, because one of them has to leave. Priority 1.

### QG-2: A booking takes under a minute on a phone

Most bookings are made on the walk to the library. Priority 2.

### QG-3: One part-time administrator can run it

Library IT is one person, two days a week. Priority 3.

## Stakeholders

| Role/Name | Contact | Expectations |
|---|---|---|
| Students | the student council | a seat they booked is free when they arrive |
| Reading-room staff | Priya Lund | occupancy at a glance; closing a room in one step |
| Library IT | Tomás Brandt | nothing to do on a normal day |
| Architect | Ines Varga | every decision traceable to a quality goal |
