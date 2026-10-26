---
inclusion: always
---

# STEER-tech: Technology Stack

## stack: Stack

- TypeScript 5 on Node.js 20, as ES modules.
- Vitest for tests, and fast-check for property tests.

## money: Money

Every amount is an integer number of cents. A price is never a float, and no
arithmetic on money divides, so no amount is ever rounded.

## commands: Common commands

- `npm test` runs every test once.
- `npx grund check` checks the specs, the code and the tests against each other.
