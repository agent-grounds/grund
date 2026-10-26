# DESIGN-coupon-codes: Design Document: Coupon codes

## overview: Overview

Applying a code is one pure function from a subtotal, a submitted code, the
coupon book and today's date to an outcome. It meets [§REQ-coupon-codes](requirements.md#req-coupon-codes-requirements), and it
keeps money in integer cents ([§STEER-tech.money](../../steering/tech.md#money-money)).

## architecture: Architecture

The checkout page calls `applyCoupon` and renders its outcome. The coupon book
is passed in rather than read from storage, so the function has no side
effects. It lives in `src/coupons.ts`, as the structure steering lays out
([§STEER-structure.layout](../../steering/structure.md#layout-layout)).

## components: Components and Interfaces

### components.normalize: normalizeCode

`normalizeCode(code: string): string` trims the code and folds it to upper
case, so that every lookup compares normalised codes ([§REQ-coupon-codes.1.3](requirements.md#3-case-and-spacing)).

### components.apply: applyCoupon

`applyCoupon(subtotalCents, code, book, today): Outcome` looks the code up and
takes the coupon's discount off the subtotal, never below zero
([§REQ-coupon-codes.1.1](requirements.md#1-active-code), [§REQ-coupon-codes.1.2](requirements.md#2-floor-at-zero)).

## data-models: Data Models

```ts
interface Coupon { code: string; discountCents: number; lastValidDay: string }
type Outcome =
  | { ok: true; subtotalCents: number }
  | { ok: false; subtotalCents: number; message: string };
```

## properties: Correctness Properties

### .1: The subtotal never goes below zero

*For any* subtotal and any active coupon, the subtotal `applyCoupon` returns is
the subtotal less the discount, or zero when the discount is larger.

**Validates:** [§REQ-coupon-codes.1.1](requirements.md#1-active-code), [§REQ-coupon-codes.1.2](requirements.md#2-floor-at-zero)

### .2: Normalising ignores case and spacing

*For any* code, `normalizeCode` gives the same result for the code with its
letters in any case and any spaces around it, and normalising twice changes
nothing.

**Validates:** [§REQ-coupon-codes.1.3](requirements.md#3-case-and-spacing)

## errors: Error Handling

A refused code is an outcome, not an exception: `ok` is false, the subtotal is
the one passed in, and `message` is the text the requirement fixes
([§REQ-coupon-codes.2.1](requirements.md#1-unknown-code), [§REQ-coupon-codes.2.2](requirements.md#2-expired-code)).

## testing: Testing Strategy

Each criterion gets an example test and each property a fast-check property
test, as the testing steering asks ([§STEER-testing.criteria](../../steering/testing.md#criteria-every-test-names-its-criterion),
[§STEER-testing.properties](../../steering/testing.md#properties-property-tests)).
