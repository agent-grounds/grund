# Requirements Document

## Introduction

Shoppers at Quillmarsh Pantry receive coupon codes by email and on printed
inserts. This feature lets a shopper enter one code at checkout and see the
discounted price before paying, as the product steering asks
([§STEER-product.goals](../../steering/product.md#goals-what-the-shop-optimises-for)).

## Glossary

- **Checkout**: the page that totals an order and takes payment.
- **Coupon**: a code, the discount it grants in cents, and the last day it is valid.
- **Subtotal**: the order's total before shipping, in cents.

## REQ-coupon-codes: Requirements

### 1. Apply a coupon code

**User Story:** As a shopper, I want to enter a coupon code at checkout, so that
I pay the discounted price the promotion advertised.

#### .1. Active code

WHEN a shopper submits the code of an active coupon, THE Checkout SHALL reduce
the subtotal by the coupon's discount.

#### .2. Floor at zero

WHEN a coupon's discount exceeds the subtotal, THE Checkout SHALL reduce the
subtotal to zero and no further.

#### .3. Case and spacing

THE Checkout SHALL match a submitted code to a coupon regardless of letter case
and of spaces before or after it.

### 2. Refuse a code that cannot be used

**User Story:** As the shop's operator, I want unknown and expired codes
refused, so that no order is discounted by a promotion that does not exist or
has ended.

#### .1. Unknown code

IF a shopper submits a code that matches no coupon, THEN THE Checkout SHALL
leave the subtotal unchanged and show "This code is not valid".

#### .2. Expired code

IF a shopper submits a coupon's code after its last valid day, THEN THE
Checkout SHALL leave the subtotal unchanged and show "This code has expired".
