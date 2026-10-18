// §FS-coordinate-globs.bindings — `refs` takes `globs` or `from` with a null ID, `cover` takes
// `globs` and keeps every entry, `note` carries the CLI's note, and selectors conflict.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { join } from 'node:path';
import * as api from 'grund-cli';
const cases = join(process.env.GRUND_TEST_REPO, 'tests', 'e2e', 'cases');
const repo = name => join(cases, name, 'repo');
const golden = (name, stream) => readFile(join(cases, name, `expected.${stream}`), 'utf8');
const jsonRows = text => text.split('\n').filter(Boolean).map(line => JSON.parse(line));
const record = row => ({ project: row.project ?? null, path: row.path, line: row.line,
  column: row.column, id: row.id, section: row.section, marker: row.marker, text: row.text,
  enclosingDeclaration: row.enclosing_declaration, enclosingSection: row.enclosing_section });
const hit = h => ({ project: h.project, path: h.path, line: h.line, column: h.column, id: h.id,
  section: h.section, marker: h.marker, text: h.text,
  enclosingDeclaration: h.enclosingDeclaration, enclosingSection: h.enclosingSection });
const rejectsAs = (pending, kind, code) => assert.rejects(pending, error => {
  assert.ok(error instanceof api.GrundError);
  assert.deepEqual([error.failure.kind, error.failure.code], [kind, code]);
  return true;
});

for (const [name, options] of [
  ['refs-glob-union-once-json', { globs: ['FS-login.1{,.**}', 'FS-login.1.*'] }],
  ['workspace-refs-glob-qualified-json', { globs: ['target/FS-login{,.**}'] }],
  ['refs-from-json', { from: 'FS-login.1' }],
  ['workspace-refs-from-keeps-the-alias-json', { from: 'citer/FS-audit' }],
]) {
  const result = await api.refs(null, { root: repo(name), ...options });
  assert.deepEqual(result.hits.map(hit), jsonRows(await golden(name, 'stdout')).map(record), name);
  assert.deepEqual([result.kindTitle, result.note], [null, null], name);
}
{
  const name = 'refs-glob-total-json';
  const result = await api.refs(null, { root: repo(name), globs: ['FS-login{,.**}'] });
  assert.deepEqual([result.totals], jsonRows(await golden(name, 'stdout')));
}
{
  const name = 'cover-glob-json';
  const result = await api.cover({ root: repo(name), globs: ['FS-ledger.*'] });
  assert.deepEqual(result.entries.map(entry => [entry.path, entry.citations.map(hit)]),
    jsonRows(await golden(name, 'stdout')).map(row => [row.path, row.citations.map(record)]));
  assert.equal(result.note, null);
}
{
  const name = 'refs-glob-note';
  const note = (await golden(name, 'stderr')).replace(/^note: /, '').replace(/\n$/, '');
  const root = repo(name);
  assert.equal((await api.refs(null, { root, globs: ['POLICY-*'] })).note, note);
  assert.equal((await api.cover({ root, globs: ['POLICY-*'] })).note, note);
  assert.equal((await api.cover({ root })).note, null);
}
{
  const name = 'refs-glob-refused-before-discovery';
  const message = (await golden(name, 'stderr')).replace(/^error: /, '').replace(/\n$/, '');
  const root = repo(name);
  // Thunks, so each call starts under its own handler and no rejection waits unhandled.
  for (const pending of [() => api.refs(null, { root, globs: ['FS-{a,b'] }),
    () => api.cover({ root, globs: ['FS-{a,b'] })])
    await assert.rejects(pending, error => {
      const f = error.failure;
      assert.deepEqual([f.kind, f.code, f.message, f.column],
        ['query', 'invalid-coordinate-glob', message, 4]);
      return true;
    });
}
{
  const name = 'refs-from-not-found-json';
  const [refusal] = jsonRows(await golden(name, 'stderr'));
  await assert.rejects(api.refs(null, { root: repo(name), from: 'FS-nope' }), error => {
    assert.deepEqual([error.failure.kind, error.failure.code, error.failure.message],
      ['query', refusal.code, refusal.message]);
    return true;
  });
}
{
  const root = repo('refs-from-json');
  for (const [id, options] of [
    ['FS-login', { globs: ['FS-*'] }], ['FS-login', { from: 'FS-login' }],
    [null, { globs: ['FS-*'], from: 'FS-login' }], [null, { globs: ['FS-*'], section: '1' }],
    [null, { globs: ['FS-*'], descendants: true }], [null, { from: 'FS-login', section: '1' }],
    [null, { from: 'FS-login', descendants: true }],
  ])
    await rejectsAs(api.refs(id, { root, ...options }), 'input', 'conflicting-options');
  await rejectsAs(api.refs(null, { root }), 'input', 'invalid-argument');
  await rejectsAs(api.refs(null, { root, globs: [] }), 'input', 'invalid-argument');
  assert.deepEqual((await api.refs('FS-login', { root, globs: [] })).hits,
    (await api.refs('FS-login', { root })).hits);
}
