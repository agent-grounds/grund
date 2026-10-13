/**
 * AR-javascript: JavaScript router
 *
 * ## 1. Contract
 *
 * Routes requests from JavaScript.
 */
export function route() {
  // Implements §FS-entry.1.
  // See §FS-gone-javascript.
  const one = "data §api/FS-str-javascript";
  const many = `first line
data §api/FS-multi-javascript
`;
  return [one, many];
}
