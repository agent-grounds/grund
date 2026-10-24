# Requirements Document

## Introduction

Orders above a threshold ship free. The threshold is judged on the subtotal a
shopper actually pays, after any coupon ([§REQ-coupon-codes.1.1](../coupon-codes/requirements.md#1-active-code)).

## REQ-free-shipping: Requirements

### 1. Ship large orders free

**User Story:** As a shopper, I want an order above a threshold to ship free,
so that I can avoid the shipping fee by adding to my basket.

#### .1. At or above the threshold

WHEN the discounted subtotal is 5000 cents or more, THE Checkout SHALL set the
shipping fee to zero.

#### .2. Below the threshold

WHILE the discounted subtotal is below 5000 cents, THE Checkout SHALL show how
many cents more would make the order ship free.
