// §FS-distribution.3.2.3.2 — test-only optional payload resolution.
const assert = require('node:assert/strict');
const Module = require('node:module');
const original = Module._load;
const descriptor = JSON.parse(process.env.GRUND_TEST_PLATFORM_PAYLOAD);
let requests = 0;
Module._load = function(request, parent, main) {
  if (/grund/.test(request) && request !== 'grund-cli' &&
      !request.startsWith('.') && !require('node:path').isAbsolute(request)) {
    requests++;
    return request.endsWith('/package.json')
      ? { version: descriptor.metadata.packageVersion } : descriptor;
  }
  return original.call(this, request, parent, main);
};
const api = require('grund-cli');
(async () => {
  if (process.env.GRUND_TEST_EXPECT_LOAD === 'yes')
    await assert.rejects(api.check(process.argv[2]), e => e.failure.kind === 'load');
  else assert.ok((await api.check(process.argv[2])).report.errors.some(f => f.code === 'dangling'));
  if (process.env.GRUND_TEST_EXPECT_LOCAL === 'yes')
    assert.equal(requests, 0, 'local payload has precedence');
  else assert.ok(requests > 0, 'selected an optional platform payload');
})().catch(error => { process.stderr.write(error.stack); process.exitCode = 1; });
