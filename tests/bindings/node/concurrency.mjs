// §FS-distribution.3.2.3.1 — copied requests, independent roots and read purity.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { cp, readFile, writeFile, readdir, lstat } from 'node:fs/promises';
import path from 'node:path';
import * as api from 'grund-cli';
const root = process.argv[2];
const other = root + '-other';
await cp(root, other, { recursive: true });
const doc = other + '/docs/functional-spec/FS-001-alpha.md';
await writeFile(doc, (await readFile(doc, 'utf8')).replace('FS-999-missing', 'FS-002-beta') +
  '\n雪 §FS-002-beta\n');
async function digest(base) {
  const rows = [];
  async function walk(dir) {
    for (const name of (await readdir(dir)).sort()) {
      const file = path.join(dir, name);
      if ((await lstat(file)).isDirectory()) await walk(file);
      else rows.push([path.relative(base, file), (await readFile(file)).toString('base64')]);
    }
  }
  await walk(base);
  return createHash('sha256').update(JSON.stringify(rows)).digest('hex');
}
const before = await digest(root);
const cwd = process.cwd();
const isolated = await Promise.all([api.check(root), api.check(other)]);
assert.equal(isolated[1].report.errors.length, 0, 'clean check resolves');
const unicodeRefs = await api.refs('FS-002-beta', { root: other });
assert.equal(unicodeRefs.hits.find(h => h.text === '§FS-002-beta').column, 5,
             'one-based byte column after a three-byte character and space');
try {
  process.chdir(path.dirname(root));
  const captured = api.check(path.basename(root));
  process.chdir(other);
  assert.deepEqual(await captured, isolated[0], 'relative root captured before await');
} finally { process.chdir(cwd); }
for (let repetition = 0; repetition < 8; repetition++) {
  const options = { only: ['dangling'], ignore: [] };
  const pending = api.check(root, options);
  options.only.length = 0;
  options.ignore.push('dangling');
  const [left, right, selected] = await Promise.all([api.check(root), api.check(other), pending]);
  assert.deepEqual(left, isolated[0]);
  assert.deepEqual(right, isolated[1]);
  assert.ok(selected.selectedReport.errors.some(f => f.code === 'dangling'));
  assert.equal(process.cwd(), cwd);
}
await Promise.all([
  api.scan(root), api.show('FS-001-alpha', { root }), api.refs('FS-002-beta', { root }),
  api.list({ root }), api.listSizes({ root }), api.cover({ root }), api.fmt({ root }),
  api.proposeId('FS', 'Read only', { root }), api.completeIds({ root }),
  api.effectiveConfig({ root }), api.validateConfig({ root }),
]);
assert.equal(await digest(root), before);
const offline = root + '-offline';
await cp(process.env.GRUND_TEST_REPO + '/tests/e2e/cases/show-missing-snapshot-offline/repo',
         offline, { recursive: true });
// Turn any execution of the configured fetcher into a visible forbidden side effect.
await writeFile(offline + '/scripts/fetch-ticket',
  '#!/bin/sh\nprintf executed > "' + offline + '/FETCH_WAS_RUN"\nexit 93\n',
  { mode: 0o755 });
const offlineBefore = await digest(offline);
await assert.rejects(api.show('TICKET-17', { root: offline }));
await api.check(offline);
await api.list({ root: offline });
await api.completeIds({ root: offline });
assert.equal(await digest(offline), offlineBefore);
// Core workspace discovery is shared across root/member calls, without JS cwd changes.
const workspace = root + '-workspace';
await cp(process.env.GRUND_TEST_REPO + '/tests/e2e/cases/workspace-refs-ambiguous-section-json/repo',
         workspace, { recursive: true });
const configs = await Promise.all([
  api.effectiveConfig({ root: workspace }),
  api.effectiveConfig({ root: workspace + '/apps/api' }),
]);
assert.ok(configs.every(c => typeof c.root === 'string'));
assert.notEqual(configs[0].root, configs[1].root);
assert.equal(process.cwd(), cwd);
