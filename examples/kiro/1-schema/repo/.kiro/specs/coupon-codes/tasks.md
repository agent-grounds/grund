# Implementation Plan: Coupon codes

## Overview

Build the lookup first, then the discount, then the refusals. Every amount is
an integer number of cents.

## Tasks

- [x] 1. Model coupons and normalise codes
  - [x] 1.1 Define the `Coupon` and `Outcome` types in `src/coupons.ts`
    - _Requirements: [§REQ-coupon-codes.1.1](requirements.md#1-active-code), [§REQ-coupon-codes.2.1](requirements.md#1-unknown-code)_
  - [x] 1.2 Implement `normalizeCode`
    - _Requirements: [§REQ-coupon-codes.1.3](requirements.md#3-case-and-spacing)_
  - [x]* 1.3 Write property test: normalising ignores case and spacing
    - **Property 2:** [§DESIGN-coupon-codes.properties.2](design.md#2-normalising-ignores-case-and-spacing)
    - **Validates:** [§REQ-coupon-codes.1.3](requirements.md#3-case-and-spacing)

- [x] 2. Apply a code to the subtotal
  - [x] 2.1 Implement `applyCoupon` for an active coupon, floored at zero
    - _Requirements: [§REQ-coupon-codes.1.1](requirements.md#1-active-code), [§REQ-coupon-codes.1.2](requirements.md#2-floor-at-zero)_
  - [x]* 2.2 Write property test: the subtotal never goes below zero
    - **Property 1:** [§DESIGN-coupon-codes.properties.1](design.md#1-the-subtotal-never-goes-below-zero)
    - **Validates:** [§REQ-coupon-codes.1.1](requirements.md#1-active-code), [§REQ-coupon-codes.1.2](requirements.md#2-floor-at-zero)

- [ ] 3. Refuse codes that cannot be used
  - [x] 3.1 Refuse a code that matches no coupon
    - _Requirements: [§REQ-coupon-codes.2.1](requirements.md#1-unknown-code)_
  - [ ] 3.2 Refuse a coupon after its last valid day
    - _Requirements: [§REQ-coupon-codes.2.2](requirements.md#2-expired-code)_

- [ ] 4. Checkpoint - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for a faster MVP.
- Each test cites the criterion it proves, so `grund refs` lists them per criterion.
