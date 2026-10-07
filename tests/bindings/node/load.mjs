// §FS-distribution.3.2.3.2 — imports survive absent/corrupt native payloads.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import * as api from 'grund-cli';
const cjs = createRequire(import.meta.url)('grund-cli');
assert.equal(api.GrundError, cjs.GrundError);
await assert.rejects(api.check(process.argv[2]), error => {
  assert.ok(error instanceof api.GrundError);
  assert.equal(error.failure.kind, 'load');
  assert.equal(error.failure.code, 'native-load');
  assert.ok(Array.isArray(error.failure.causes));
  assert.ok(Object.keys(error.failure.details).length > 0);
  return true;
});
