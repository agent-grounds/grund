# Context and Scope

## Business Context

| Neighbour | What crosses the boundary |
|---|---|
| Student | bookings, moves and cancellations; a claim, by tapping a card at the door |
| Reading-room staff | occupancy; closing a room |
| University sign-on | who the student is |

## Technical Context

Phones and library terminals reach Carrel over HTTPS through the university's
load balancer. Each room's door reader posts a claim to Carrel when a card is
tapped. Carrel sends its notices through the university's mail relay.
