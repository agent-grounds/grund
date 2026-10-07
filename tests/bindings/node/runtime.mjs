// §FS-distribution.3.2.3.1 — deterministic test-only native barriers and faults.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { Worker } from 'node:worker_threads';
import { readFile, writeFile, mkdtemp } from 'node:fs/promises';
import * as api from 'grund-cli';
const root = process.argv[2];
const require = createRequire(import.meta.url);
// This export exists only in the feature-gated test build, never a shipped addon.
const native = require('./node_modules/grund-cli/native/grund.node');
assert.ok(native.__test, 'feature-gated test-node-runtime harness is required');
const seam = native.__test;
seam.installOtherHook(); // arrange the independent Rust hook before first invoke
const baseline = seam.resourceCounts();
seam.holdNext('check');
let ticks = 0;
const timer = setInterval(() => ticks++, 2);
const checking = api.check(root);
await seam.waitStarted();
const atStart = ticks;
await new Promise(resolve => setTimeout(resolve, 30));
assert.ok(ticks > atStart, 'timer advanced while native compute was active');
seam.release();
await checking;
clearInterval(timer);
for (const stage of ['convert', 'compute', 'rayon', 'resolve', 'reject']) {
  seam.panicNext(stage);
  await assert.rejects(api.check(root), error => {
    assert.equal(error.failure.kind, 'native');
    assert.equal(error.failure.code, 'native-panic');
    return true;
  });
  await api.check(root);
}
seam.failWorkerNext();
await assert.rejects(api.check(root), error => error.failure.kind === 'worker');
const beforeHook = seam.hookCalls();
seam.panicNext('rayon');
await assert.rejects(api.check(root), error => error.failure.kind === 'native');
assert.equal(seam.hookCalls(), beforeHook, 'guarded Rayon panic is silent');
seam.panicOutsideGuard();
assert.equal(seam.hookCalls(), beforeHook + 1, 'unrelated panic chains prior hook');
for (const outcome of ['success', 'error', 'panic']) {
  seam.holdNextWriter(outcome);
  const writing = api.fmt({ root, write: true });
  // Readiness means the lease is acquired, before any filesystem side effect.
  await seam.waitStarted();
  for (const contender of [
    () => api.fmt({ root, write: true }),
    () => api.init(root + '-other-root', { noVcs: true }),
    () => api.fetch('FS-001-alpha', { root }),
    () => api.integrations({ client: 'vscode', write: true }),
  ]) {
    await assert.rejects(contender(), error => {
      assert.equal(error.failure.kind, 'busy');
      assert.equal(error.failure.code, 'writer-busy');
      return true;
    });
  }
  // No writer lease on read-only/preview calls, even while a writer holds it.
  await api.check(root);
  await api.fmt({ root });
  seam.release();
  if (outcome === 'success') await writing;
  else await assert.rejects(writing, error => error instanceof api.GrundError);
  await api.fmt({ root, write: true });
}
// Native test builds may inject I/O failures after an owned write, without rollback.
const initTarget = await mkdtemp(root + '-partial-init-');
seam.failWriteAfter('init', 1);
await assert.rejects(api.init(initTarget, { noVcs: true, docs: true }), error => {
  assert.equal(error.failure.kind, 'io');
  assert.equal(error.failure.partial.operation, 'init');
  assert.ok(error.failure.partial.result.events.length > 0);
  return true;
});
const beta = root + '/docs/functional-spec/FS-002-beta.md';
await writeFile(beta, (await readFile(beta, 'utf8')) + '\nReferences \u00a7FS-001-alpha.\n');
const betaBefore = await readFile(beta);
const pendingFmt = await api.fmt({ root, crossRefs: true });
assert.ok(pendingFmt.changes.some(change => change.path === 'docs/functional-spec/FS-002-beta.md'),
          'a real cross-reference rewrite is pending before the write fault');
assert.deepEqual(await readFile(beta), betaBefore, 'preview leaves pending bytes intact');
seam.failWriteAfter('fmt', 1);
await assert.rejects(api.fmt({ root, write: true, crossRefs: true }), error => {
  assert.equal(error.failure.kind, 'io');
  assert.equal(error.failure.partial.operation, 'fmt');
  assert.ok(error.failure.partial.result.changes.length > 0);
  return true;
});
await api.fmt({ root, write: true });
// Terminate an environment during held native work; Rust jobs survive safely.
seam.holdNextWriter('success');
const worker = new Worker(`
  const { parentPort, workerData } = require('node:worker_threads');
  const api = require(workerData.package);
  api.fmt({root: workerData.root, write: true}).then(() => parentPort.postMessage('done'));
`, { eval: true, workerData: {
  root, package: require.resolve('grund-cli'),
}});
await seam.waitStarted();
const termination = worker.terminate();
seam.release();
await termination;
await seam.waitIdle();
await api.fmt({ root, write: true });
assert.deepEqual(seam.resourceCounts(), baseline, 'jobs, environments and leases released');
