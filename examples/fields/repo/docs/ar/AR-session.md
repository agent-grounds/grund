# AR-session: The session cache

Beside §AR-store, as §DF-cache decided; it never says where it sits.

## terms: Terms

- **session**: a signed-in user's token.

## 1. Design

An in-memory map, evicted hourly.
