# DESIGN-free-shipping: Design Document: Free shipping

## overview: Overview

One pure function, `shippingFee(subtotalCents)`, decides the fee from the
discounted subtotal that `applyCoupon` returns ([§DESIGN-coupon-codes.components.apply](../coupon-codes/design.md#componentsapply-applycoupon)).
It meets [§REQ-free-shipping](requirements.md#req-free-shipping-requirements).

## components: Components and Interfaces

### components.fee: shippingFee

`shippingFee(subtotalCents: number): { feeCents: number; centsToFree: number }`
returns a zero fee at or above the threshold ([§REQ-free-shipping.1.1](requirements.md#1-at-or-above-the-threshold)), and below
it the cents still needed ([§REQ-free-shipping.1.2](requirements.md#2-below-the-threshold)).

## properties: Correctness Properties

### .1: Free exactly from the threshold up

*For any* subtotal, the fee is zero if and only if the subtotal is at least the
threshold.

**Validates:** [§REQ-free-shipping.1.1](requirements.md#1-at-or-above-the-threshold)

## testing: Testing Strategy

The property gets a fast-check test, and each criterion an example test at the
threshold and one cent below it ([§STEER-testing.properties](../../steering/testing.md#properties-property-tests)).
