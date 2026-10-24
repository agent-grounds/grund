---
inclusion: fileMatch
fileMatchPattern: "test/**/*.test.ts"
---

# STEER-testing: Testing Standards

## criteria: Every test names its criterion

Each test opens with a doc comment that cites the acceptance criterion it
proves, as `grund check` requires of every doc comment under `test/`. A test
that proves no criterion is not written.

## properties: Property tests

Each correctness property in a design gets one fast-check property test. Its
doc comment cites the property and every criterion the property validates.
