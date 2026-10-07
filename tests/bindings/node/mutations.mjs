// §FS-distribution.3.2.1 — exact core-owned bytes, previews and refusal safeguards.
import assert from 'node:assert/strict';
import { cp, readdir, readFile, lstat, mkdtemp, symlink, mkdir, realpath, stat, rm }
  from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import path from 'node:path';
import * as api from 'grund-cli';
const root = process.argv[2];
const cases = process.env.GRUND_TEST_REPO + '/tests/e2e/cases/';
async function bytes(base) {
  const rows = {};
  async function walk(dir) {
    for (const name of (await readdir(dir)).sort()) {
      const file = path.join(dir, name);
      if ((await lstat(file)).isDirectory()) await walk(file);
      else rows[path.relative(base, file)] = (await readFile(file)).toString('base64');
    }
  }
  await walk(base);
  return rows;
}
async function fixture(name) {
  const target = await mkdtemp(root + '-mutation-');
  await cp(cases + name + '/repo', target, { recursive: true });
  return target;
}
// §FS-distribution.3.2.3.1: arrange the core's complete VCS-ancestry precondition.
async function vcsMarkers(target) {
  const found = [];
  for (let dir = await realpath(target); ; dir = path.dirname(dir)) {
    for (const marker of ['.git', '.hg', '.jj', '.svn']) {
      const file = path.join(dir, marker);
      try {
        await stat(file);
        found.push(file);
      } catch (error) {
        if (!['ENOENT', 'ENOTDIR'].includes(error.code)) throw error;
      }
    }
    if (path.dirname(dir) === dir) return found;
  }
}
async function unversionedFixture() {
  const obstructions = [];
  const candidates = new Set([
    path.dirname(root), process.platform === 'linux' ? '/dev/shm' : tmpdir(),
  ]);
  for (const base of candidates) {
    let target;
    try {
      const ancestors = await vcsMarkers(base);
      if (ancestors.length) throw Error('VCS markers: ' + ancestors.join(', '));
      target = await mkdtemp(path.join(base, 'grund-node-no-vcs-'));
      const markers = await vcsMarkers(target);
      if (markers.length) throw Error('VCS markers: ' + markers.join(', '));
      return target;
    } catch (error) {
      obstructions.push(base + ': ' + error.message);
      if (target) await rm(target, { recursive: true, force: true });
    }
  }
  throw Error('Cannot arrange an unversioned guard fixture: ' + obstructions.join('; '));
}
const formatted = await fixture('fmt-shorthand-write');
const before = await bytes(formatted);
await api.fmt({ root: formatted });
assert.deepEqual(await bytes(formatted), before, 'default fmt is a preview');
await api.fmt({ root: formatted, write: true });
assert.deepEqual(await bytes(formatted), await bytes(cases + 'fmt-shorthand-write/expected.repo'));
assert.equal((await api.fmt({ root: formatted, write: true })).changes.length, 0);
const protectedConfig = await fixture('init-force-keeps-existing-config');
const initial = await bytes(protectedConfig);
await api.init(protectedConfig, { dryRun: true, noVcs: true });
assert.deepEqual(await bytes(protectedConfig), initial);
await api.init(protectedConfig, { check: true, noVcs: true });
assert.deepEqual(await bytes(protectedConfig), initial, 'check implies dryRun');
await api.init(protectedConfig, { force: true, noVcs: true });
const after = await bytes(protectedConfig);
assert.equal(after['grund.toml'], initial['grund.toml'], 'force never rewrites existing config');
assert.deepEqual(after, await bytes(cases + 'init-force-keeps-existing-config/expected.repo'));
// §FS-init.2.3.8.3: invalid config refuses before writes; existing prose accepts append.
const appending = await fixture('init-conflict-no-force');
const appendBefore = await bytes(appending);
const prose = await readFile(appending + '/AGENTS.md', 'utf8');
const appended = await api.init(appending, { noVcs: true, name: 'append-fixture' });
assert.ok(appended.events.some(e => e.path === 'AGENTS.md' && e.verb === 'appended'));
assert.ok((await readFile(appending + '/AGENTS.md', 'utf8')).startsWith(prose));
const nodeAppend = await bytes(appending);
// §FS-distribution.3.2.3.1: compare exact bytes at the identical restored CLI target.
await rm(appending, { recursive: true });
await mkdir(appending);
await cp(cases + 'init-conflict-no-force/repo', appending, { recursive: true });
assert.deepEqual(await bytes(appending), appendBefore);
assert.ok(process.env.GRUND_TEST_CLI, 'same-source CLI byte oracle is required');
const cliAppend = spawnSync(process.env.GRUND_TEST_CLI,
  ['init', appending, '--no-vcs', '--name', 'append-fixture'], { encoding: 'utf8' });
assert.ifError(cliAppend.error);
assert.equal(cliAppend.status, 0, cliAppend.stderr);
assert.deepEqual(nodeAppend, await bytes(appending));
const refusing = await fixture('init-invalid-config-is-an-error');
const refusedBefore = await bytes(refusing);
await assert.rejects(api.init(refusing, { noVcs: true }), error => error instanceof api.GrundError);
assert.deepEqual(await bytes(refusing), refusedBefore, 'whole-set preflight before writes');
const value = await fixture('fmt-embedded-value-refusal-stable');
const valueBefore = await bytes(value);
const refusal = await api.fmt({ root: value, write: true });
assert.ok(Array.isArray(refusal.refusedWrites));
assert.deepEqual(refusal.changes, []);
assert.deepEqual(await bytes(value), valueBefore);
const external = await fixture('fmt-index-external-file-symlink');
const outsideBefore = await readFile(external + '/outside/index.md');
await symlink(external + '/outside/index.md', external + '/project/docs/fs/README.md');
const symlinkRefusal = await api.fmt({ root: external + '/project', write: true });
assert.ok(symlinkRefusal.refusedWrites.length > 0);
assert.deepEqual(await readFile(external + '/outside/index.md'), outsideBefore);
await mkdir(process.env.HOME, { recursive: true });
const homeBefore = await bytes(process.env.HOME);
await assert.rejects(api.init(process.env.HOME, { noVcs: true }),
                     error => error instanceof api.GrundError);
assert.deepEqual(await bytes(process.env.HOME), homeBefore, 'home guard is preserved');
const nonVcs = await unversionedFixture();
try {
  assert.deepEqual(await vcsMarkers(nonVcs), []);
  const nonVcsBefore = await bytes(nonVcs);
  assert.deepEqual(nonVcsBefore, {});
  await assert.rejects(api.init(nonVcs), error => {
    assert.ok(error instanceof api.GrundError);
    assert.equal(error.failure.kind, 'operation');
    assert.equal(error.failure.code, 'init');
    assert.match(error.failure.message, /is not inside a version-controlled tree/);
    return true;
  });
  assert.deepEqual(await bytes(nonVcs), nonVcsBefore, 'VCS guard before side effects');
} finally {
  await rm(nonVcs, { recursive: true, force: true });
}
// The explicit fetch write runs a POSIX fetcher, so it is fetch.mjs's own skippable test.
// User-global integration install is isolated in a subprocess HOME/XDG fixture.
const detection = await api.integrations();
assert.equal(detection.mode, 'detection');
const artifact = await api.integrations({ client: 'vscode' });
assert.equal(artifact.mode, 'artifact');
assert.ok(artifact.artifacts.length > 0);
const installed = await api.integrations({ client: 'vscode', write: true });
assert.equal(installed.mode, 'install');
assert.ok(Array.isArray(installed.events));
const repeated = await api.integrations({ client: 'vscode', write: true });
assert.deepEqual(repeated.preferences, installed.preferences);
