# Implementation Plan: Free shipping

## Overview

Not started. The fee depends on the discounted subtotal, so this plan follows
the coupon plan.

## Tasks

- [ ] 1. Implement `shippingFee` in `src/shipping.ts`
  - _Requirements: [§REQ-free-shipping.1.1](requirements.md#1-at-or-above-the-threshold), [§REQ-free-shipping.1.2](requirements.md#2-below-the-threshold)_
  - [ ]* 1.1 Write property test: free exactly from the threshold up
    - **Property 1:** [§DESIGN-free-shipping.properties.1](design.md#1-free-exactly-from-the-threshold-up)
    - **Validates:** [§REQ-free-shipping.1.1](requirements.md#1-at-or-above-the-threshold)

- [ ] 2. Checkpoint - Ensure all tests pass, ask the user if questions arise.
