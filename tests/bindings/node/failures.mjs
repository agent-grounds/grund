// §FS-distribution.3.2.2.1 — failures are Promise data, never parsed messages.
import assert from 'node:assert/strict';
import { mkdtemp, writeFile, mkdir, cp, readFile } from 'node:fs/promises';
import * as api from 'grund-cli';
const root = process.argv[2];
async function failure(name, args, kind, code) {
  let pending;
  assert.doesNotThrow(() => { pending = api[name](...args); }, name);
  assert.ok(pending instanceof Promise, name);
  await assert.rejects(pending, error => {
    assert.ok(error instanceof api.GrundError);
    const f = error.failure;
    assert.equal(f.kind, kind);
    if (code) assert.equal(f.code, code);
    for (const field of ['path', 'line', 'column', 'partial'])
      assert.ok(Object.hasOwn(f, field), field);
    for (const field of ['sites', 'authority', 'causes', 'runCautions'])
      assert.ok(Array.isArray(f[field]), field);
    assert.equal(f.operation, name);
    return true;
  });
}
const invalid = [
  ['check', [root, { invented: true }]], ['check', [root, { suggestions: 1 }]],
  ['scan', []], ['scan', [new URL('file:///repo')]],
  ['show', ['\ud800', { root }]], ['show', ['FS-001-alpha', { root, mode: 'wide' }]],
  ['showBatch', [['FS-001-alpha', 7], { root }]],
  ['refs', ['FS-001-alpha', { root, descendants: 'yes' }]],
  ['list', [{ root, kinds: 'FS' }]], ['listSizes', [{ root, units: [] }]],
  ['listSizes', [{ root, units: ['bytes', 'bytes'] }]],
  ['listSizes', [{ root, top: 0 }]], ['cover', [{ root: null }]],
  ['fmt', [{ root, write: 'yes' }]], ['proposeId', ['FS', 'snow', { root, width: NaN }]],
  ['proposeId', ['FS', 'snow', { root, width: Number.MAX_SAFE_INTEGER + 1 }]],
  ['init', [root, { agents: [] }]], ['completeIds', [{ root, sections: null }]],
  ['effectiveConfig', [{ root, values: {} }]], ['validateConfig', [{ root: 42 }]],
  ['fetch', ['FS-001-alpha\0', { root }]],
  ['integrations', [{ conversation: 'link' }]],
  ['integrations', [{ write: true, agent: 'claude' }]],
  ['referenceStyle', [Buffer.from(root)]], ['agentSetupInstructions', [{}]],
];
for (const [name, args] of invalid) await failure(name, args, 'input');
await failure('check', [root, { onlyRule: true }], 'input', 'conflicting-options');
await failure('show', ['FS-999-missing', { root }], 'query', 'not-found');
await assert.rejects(api.show('FS-999-missing', { root }), error => {
  assert.equal(error.failure.partial, null, 'ordinary query failure has no partial output');
  assert.equal(error.failure.message, 'ID not found: FS-999-missing');
  return true;
});
await failure('show', ['FS-001-alpha.9', { root, section: '1' }], 'query');
await failure('show', ['FS-001-alpha', { root, section: '99' }], 'query', 'missing-section');
await failure('list', [{ root, kinds: ['NOTAKIND'] }], 'operation');
await failure('list', [{ root, projects: ['unknown'] }], 'operation');
await failure('list', [{ root, selector: 'FS-999-missing' }], 'query');
await failure('proposeId', ['NOTAKIND', 'Snow', { root }], 'operation');
await failure('proposeId', ['FS', '', { root }], 'query');
await failure('fetch', ['FS-001-alpha', { root }], 'operation');
const invalidRoot = await mkdtemp(root + '-invalid-');
await writeFile(invalidRoot + '/grund.toml', 'grund_config_version = "wrong"\n');
for (const name of ['effectiveConfig', 'validateConfig'])
  await failure(name, [{ root: invalidRoot }], 'config', 'invalid-config');
// §FS-distribution.3.2.1: root discovery cautions survive later member validation failure.
const workspace = await mkdtemp(root + '-validation-');
await mkdir(workspace + '/.agents');
await mkdir(workspace + '/child');
await writeFile(workspace + '/grund.toml',
  'grund_config_version = 1\n[workspace]\nmembers = ["child"]\n');
await writeFile(workspace + '/.agents/grund.toml', 'grund_config_version = 1\n');
await writeFile(workspace + '/child/grund.toml', 'grund_config_version = "bad"\n');
const earlier = (await api.effectiveConfig({ root: workspace })).runCautions;
assert.ok(earlier.some(f => f.code === 'redundant-config'));
await assert.rejects(api.validateConfig({ root: workspace }), error => {
  assert.equal(error.failure.kind, 'config');
  assert.equal(error.failure.code, 'invalid-config');
  assert.equal(error.failure.path, 'child/grund.toml');
  assert.equal(error.failure.line, 1);
  assert.equal(error.failure.partial, null);
  assert.deepEqual(error.failure.runCautions, earlier);
  return true;
});
// Duplicate sites must survive query failure as data.
const duplicate = await mkdtemp(root + '-duplicate-');
await mkdir(duplicate + '/docs/functional-spec', { recursive: true });
await writeFile(duplicate + '/grund.toml', 'grund_config_version = 1\n');
for (const name of ['a', 'b'])
  await writeFile(duplicate + '/docs/functional-spec/' + name + '.md', '# FS-001-alpha: Alpha\n');
await assert.rejects(api.show('FS-001-alpha', { root: duplicate }), error => {
  assert.equal(error.failure.kind, 'query');
  assert.equal(error.failure.code, 'ambiguous');
  assert.equal(error.failure.sites.length, 2);
  assert.deepEqual(error.failure.authority, []);
  return true;
});
// Invalid UTF-8 is a deterministic per-file scan error, independent of user privileges.
const partial = await mkdtemp(root + '-partial-');
await cp(root, partial, { recursive: true });
await writeFile(partial + '/docs/functional-spec/FS-003-broken.md', Buffer.from([0xff, 0xfe]));
const incomplete = await api.check(partial);
assert.equal(incomplete.hadScanErrors, true, 'incomplete checks still resolve');
assert.ok(incomplete.report.errors.some(f => f.code === 'io'));
await assert.rejects(api.scan(partial), error => {
  assert.equal(error.failure.kind, 'io');
  assert.equal(error.failure.partial.operation, 'scan');
  assert.ok(error.failure.partial.result.declarations.length > 0);
  return true;
});
await assert.rejects(api.fmt({ root: partial, write: true, crossRefs: true }), error => {
  assert.equal(error.failure.kind, 'io');
  assert.equal(error.failure.partial.operation, 'fmt');
  assert.ok(error.failure.partial.result.scanErrors.length > 0);
  assert.deepEqual(error.failure.partial.result.changes, []);
  return true;
});
// §FS-distribution.3.2.2.2: a real completed integration write precedes resolver refusal.
// Rehearsal supplies a fresh isolated HOME/XDG tree for this subprocess.
await mkdir(process.env.HOME + '/.local', { recursive: true });
await writeFile(process.env.HOME + '/.local/bin', 'blocked resolver directory');
const kitty = process.env.XDG_CONFIG_HOME + '/kitty/kitty.conf';
// §FS-errors.4: an event path is spelled with `/` on every OS, Windows included.
const kittyEvent = kitty.replaceAll('\\', '/');
await assert.rejects(api.integrations({ client: 'kitty', write: true }), error => {
  const f = error.failure;
  assert.equal(f.kind, 'io');
  assert.equal(f.code, 'filesystem');
  assert.equal(f.partial.operation, 'integrations');
  const result = f.partial.result;
  assert.equal(result.mode, 'install');
  assert.ok(result.events.some(e => e.path === kittyEvent && e.verb === 'appended'));
  for (const event of result.events) assert.deepEqual(Object.keys(event).sort(), ['path', 'verb']);
  assert.deepEqual(result.notes, []);
  assert.deepEqual(result.manualSteps, []);
  assert.deepEqual(result.preferences,
    { conversation: 'plain', conversationTarget: 'file', agentOverrides: {} });
  return true;
});
assert.ok((await readFile(kitty, 'utf8')).includes('# >>> grund integrations'),
          'completed artifact remains');
