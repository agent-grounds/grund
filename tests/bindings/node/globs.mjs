// §FS-coordinate-globs.bindings — `globs` selects as repeated --glob flags do, `note` carries
// the CLI's note, and a refused pattern is a query failure.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { join } from 'node:path';
import * as api from 'grund-cli';
const cases = join(process.env.GRUND_TEST_REPO, 'tests', 'e2e', 'cases');
const repo = name => join(cases, name, 'repo');
const golden = (name, stream) => readFile(join(cases, name, `expected.${stream}`), 'utf8');
const jsonRows = text => text.split('\n').filter(Boolean).map(line => JSON.parse(line));
const pick = (row, fields) => Object.fromEntries(fields.map(field => [field, row[field]]));

{
  const name = 'list-glob-point-and-subtree-json';
  const fields = ['id', 'section', 'kind', 'path', 'line', 'title'];
  const result = await api.list({ root: repo(name), globs: ['FS-config.requirements{,.**}'] });
  assert.deepEqual(result.entries.map(entry => pick(entry, fields)),
    jsonRows(await golden(name, 'stdout')).map(row => pick(row, fields)));
  assert.equal(result.note, null);
}
{
  const name = 'list-glob-size';
  const result = await api.listSizes({ root: repo(name), globs: ['FS-*.terms'], units: ['lines'] });
  const coordinates = (await golden(name, 'stdout')).split('\n').filter(Boolean)
    .map(line => line.split(/\s+/)[0]);
  assert.deepEqual(result.entries.map(entry => `${entry.id}.${entry.section}`), coordinates);
}
{
  const name = 'show-batch-glob-json';
  const result = await api.showBatch(null, { root: repo(name), globs: ['FS-config.requirements{,.**}'] });
  assert.deepEqual(result.records.map(record => [record.query.id, record.query.section ?? null]),
    jsonRows(await golden(name, 'stdout')).map(row => [row.query.id, row.query.section]));
}
{
  const name = 'list-glob-note-suggests-the-kind';
  const note = (await golden(name, 'stderr')).replace(/^note: /, '').replace(/\n$/, '');
  const root = repo(name);
  for (const pending of [api.list({ root, globs: ['fs-*'] }), api.listSizes({ root, globs: ['fs-*'] }),
    api.showBatch(null, { root, globs: ['fs-*'] })])
    assert.equal((await pending).note, note);
  assert.equal((await api.list({ root })).note, null);
}
{
  const name = 'list-glob-refused-star-run-ends-the-pattern';
  const message = (await golden(name, 'stderr')).replace(/^error: /, '').replace(/\n$/, '');
  const root = repo(name);
  await assert.rejects(api.list({ root, globs: ['FS-**'] }), error => {
    assert.ok(error instanceof api.GrundError);
    const f = error.failure;
    assert.deepEqual([f.kind, f.code, f.message, f.column],
      ['query', 'invalid-coordinate-glob', message, 4]);
    return true;
  });
  await assert.rejects(api.showBatch(['FS-config'], { root, globs: ['FS-*'] }), error => {
    assert.deepEqual([error.failure.kind, error.failure.code], ['input', 'conflicting-options']);
    return true;
  });
}
