# REQ-coupons: Coupon codes

## 1. Apply a coupon code

### .1. Active code

WHEN a shopper submits an active code, THE Checkout SHALL reduce the subtotal.

## 2. Refuse a code that cannot be used

### .1. Unknown code

IF a code matches no coupon, THEN THE Checkout SHALL show "This code is not valid".

### .2. Expired code

IF a code is past its last valid day, THEN THE Checkout SHALL show "This code has expired".
