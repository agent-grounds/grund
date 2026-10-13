/**
 * AR-typescript: TypeScript router
 *
 * ## 1. Contract
 *
 * Routes requests from TypeScript.
 */
export function route(): string[] {
  // Implements §FS-entry.1.
  // See §FS-gone-typescript.
  const one: string = "data §api/FS-str-typescript";
  const many: string = `first line
data §api/FS-multi-typescript
`;
  return [one, many];
}
