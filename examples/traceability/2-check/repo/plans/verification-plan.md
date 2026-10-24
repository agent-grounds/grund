# LC-1 charge controller: verification plan, trace criteria

The trace is complete when every level is cited by the level below it. The
schema already makes each level cite the one above. These rules are the other
direction, and `grund check` holds the tree to each of them:

- [§RULE-001](verification-plan.md#rule-001-each-stk-must-be-cited-by-at-least-one-sys)
- [§RULE-002](verification-plan.md#rule-002-each-sys-must-be-cited-by-at-least-one-swr)
- [§RULE-003](verification-plan.md#rule-003-each-swr-must-be-cited-by-at-least-one-des)
- [§RULE-004](verification-plan.md#rule-004-each-swr-must-be-cited-by-at-least-one-tc-or-ana)

## RULE-001: Each STK must be cited by at least one SYS.

A need that no system requirement answers is a need the product ignores.

## RULE-002: Each SYS must be cited by at least one SWR.

A system requirement no software requirement refines is implemented by no one.

## RULE-003: Each SWR must be cited by at least one DES.

A software requirement the design does not realize has no code behind it.

## RULE-004: Each SWR must be cited by at least one TC or ANA.

Every software requirement is verified, by a test case or by an analysis.
