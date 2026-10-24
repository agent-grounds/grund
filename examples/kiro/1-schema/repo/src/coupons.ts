/**
 * Coupon codes at checkout, built as §DESIGN-coupon-codes.components describes.
 */

/** A coupon, as §DESIGN-coupon-codes.data-models has it: its discount in integer cents (§STEER-tech.money). */
export interface Coupon {
  code: string;
  discountCents: number;
  lastValidDay: string;
}

/** What applying a code gives the checkout: a new subtotal, or a refusal (§DESIGN-coupon-codes.errors). */
export type Outcome =
  | { ok: true; subtotalCents: number }
  | { ok: false; subtotalCents: number; message: string };

/** Trims a code and folds its case, so codes compare as §REQ-coupon-codes.1.3 asks. */
export function normalizeCode(code: string): string {
  return code.trim().toUpperCase();
}

/**
 * Applies one submitted code to a subtotal. An active code takes its discount off
 * (§REQ-coupon-codes.1.1), never below zero (§REQ-coupon-codes.1.2); a code that
 * matches no coupon leaves the subtotal as it was (§REQ-coupon-codes.2.1).
 */
export function applyCoupon(
  subtotalCents: number,
  code: string,
  book: readonly Coupon[],
  today: string,
): Outcome {
  const wanted = normalizeCode(code);
  const coupon = book.find((c) => normalizeCode(c.code) === wanted);
  if (coupon === undefined) {
    return { ok: false, subtotalCents, message: "This code is not valid" };
  }
  return { ok: true, subtotalCents: Math.max(0, subtotalCents - coupon.discountCents) };
}
