// §FS-distribution.3.2.1 — explicit fetch writes exactly the core-owned snapshot bytes.
// The fixture's fetcher is a POSIX script grund runs directly (§FS-fetch.2), so the
// Python test running this file skips it off unix, as the CLI's case runner does.
import assert from 'node:assert/strict';
import { cp, readdir, readFile, lstat, mkdtemp } from 'node:fs/promises';
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
const fetch = await mkdtemp(root + '-mutation-');
await cp(cases + 'fetch-workspace-folder/repo', fetch, { recursive: true });
const fetched = await api.fetch('alpha/TICKET-1234', { root: fetch });
assert.equal(fetched.id, 'alpha/TICKET-1234');
assert.deepEqual(await bytes(fetch), await bytes(cases + 'fetch-workspace-folder/expected.repo'));
