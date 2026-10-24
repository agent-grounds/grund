# STEER-structure: Project Structure

## layout: Layout

- `src/` holds one module per checkout concern, named after it: `coupons.ts`.
- `test/` holds one test file per module: `coupons.test.ts`.
- `.kiro/specs/<feature>/` holds each feature's spec.

## traceability: Traceability

Specs, code and tests cite each other by grund ID, and `grund check` holds them
to the schema below. A spec's requirements are `REQ-<feature>` and its design
`DESIGN-<feature>`. Criterion 1.2 of the coupon spec is `REQ-coupon-codes.1.2`,
and a citation of it is that ID with the section sign in front. Steering cites
no spec, because it outlives every one of them. Each spec has a design before
it has tasks ([§RULE-design](traceability.md#rule-design-each-req-must-be-cited-by-at-least-one-design)).

#[[file:grund.toml]]
