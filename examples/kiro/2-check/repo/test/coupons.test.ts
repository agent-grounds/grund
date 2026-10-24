/** Tests for §REQ-coupon-codes, one doc comment per criterion or property. */
import { describe, expect, it } from "vitest";
import fc from "fast-check";
import { applyCoupon, normalizeCode, type Coupon } from "../src/coupons";

const book: Coupon[] = [{ code: "SPRING10", discountCents: 1000, lastValidDay: "2026-12-31" }];
const today = "2026-10-22";

describe("applyCoupon", () => {
  /** §REQ-coupon-codes.1.1: an active code takes its discount off the subtotal. */
  it("takes an active code's discount off the subtotal", () => {
    expect(applyCoupon(4500, "SPRING10", book, today)).toEqual({ ok: true, subtotalCents: 3500 });
  });

  /** §REQ-coupon-codes.1.2: a discount larger than the subtotal stops at zero. */
  it("stops at zero", () => {
    expect(applyCoupon(600, "SPRING10", book, today)).toEqual({ ok: true, subtotalCents: 0 });
  });

  /** §REQ-coupon-codes.2.1: an unknown code changes nothing and says why. */
  it("refuses an unknown code", () => {
    expect(applyCoupon(4500, "WINTER", book, today)).toEqual({
      ok: false,
      subtotalCents: 4500,
      message: "This code is not valid",
    });
  });

  /**
   * §DESIGN-coupon-codes.properties.1, which validates §REQ-coupon-codes.1.1 and
   * §REQ-coupon-codes.1.2: the subtotal is never below zero.
   */
  it("never goes below zero", () => {
    fc.assert(
      fc.property(fc.nat(), fc.nat(), (subtotal, discount) => {
        const coupons = [{ code: "ANY", discountCents: discount, lastValidDay: "2026-12-31" }];
        const outcome = applyCoupon(subtotal, "ANY", coupons, today);
        expect(outcome).toEqual({ ok: true, subtotalCents: Math.max(0, subtotal - discount) });
      }),
    );
  });
});

describe("normalizeCode", () => {
  /** §DESIGN-coupon-codes.properties.2, which validates §REQ-coupon-codes.1.3. */
  it("ignores case and spacing, and is idempotent", () => {
    fc.assert(
      fc.property(fc.stringMatching(/^[A-Za-z0-9]+$/), fc.nat(3), (code, pad) => {
        const padded = " ".repeat(pad) + code.toLowerCase() + " ".repeat(pad);
        expect(normalizeCode(padded)).toBe(normalizeCode(code.toUpperCase()));
        expect(normalizeCode(normalizeCode(code))).toBe(normalizeCode(code));
      }),
    );
  });
});
