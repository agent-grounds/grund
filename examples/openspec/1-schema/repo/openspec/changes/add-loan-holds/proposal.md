# Proposal

## Why

A member who finds the copy they want on loan can only come back and hope. The
desk keeps a paper list of who asked first, and a renewal can keep a book away
from that list for 63 days.

## What Changes

- Members can place a hold on a copy that is on loan; a returned copy is kept
  for the first member in its hold queue.
- A renewal is refused while another member holds the copy, which changes
  [§SPEC-loans.requirements.renew-a-loan](../../specs/loans/spec.md#renew-a-loan-requirement-renew-a-loan).
- "Borrow a Book" becomes "Check Out a Book", the desk's own word, now that a
  hold is also a way to borrow: [§SPEC-loans.requirements.borrow-a-book](../../specs/loans/spec.md#borrow-a-book-requirement-borrow-a-book).

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `loans`: adds holds, refuses renewals of held copies, and renames a
  requirement ([§SPEC-loans](../../specs/loans/spec.md#spec-loans-loans-specification)).

## Impact

- `src/larkspur/loans.py` gains a hold queue per copy; `renew` reads it.
- Notices are unchanged in this change; a "hold ready" email is the next one.
