// §FS-config.4.1.1, §FS-check.4.11.3: at a workspace root, validateConfig's runCautions carry
// the root's config findings and each expanded project's, in check's order and report base.
import assert from 'node:assert/strict';
import { cp, mkdtemp } from 'node:fs/promises';
import * as api from 'grund-cli';
const root = process.argv[2];
const cases = process.env.GRUND_TEST_REPO + '/tests/e2e/cases/';
async function findings(name) {
  const copy = await mkdtemp(root + '-' + name + '-');
  await cp(cases + name + '/repo', copy, { recursive: true });
  return (await api.validateConfig({ root: copy })).runCautions
    .map(({ code, path, message }) => ({ code, path, message }));
}
assert.deepEqual(await findings('config-validate-workspace-member-findings'), [
  { code: 'deprecated-config-location', path: null,
    message: 'packages/on-agents/.agents/grund.toml is a deprecated config location'
      + ' — move it to packages/on-agents/grund.toml' },
  { code: 'redundant-config', path: null,
    message: 'packages/on-pair/.agents/grund.toml is ignored'
      + ' — packages/on-pair/grund.toml takes precedence; delete one' },
  { code: 'config-version-1', path: null,
    message: 'packages/on-v1/grund.toml is version 1;'
      + ' migrate it with grund config migrate --to 2 packages/on-v1' },
]);
assert.deepEqual(await findings('config-validate-workspace-ok'), [
  { code: 'config-version-1', path: null,
    message: 'grund.toml is version 1; migrate it with grund config migrate --to 2' },
  { code: 'config-version-1', path: null,
    message: 'packages/alpha/grund.toml is version 1;'
      + ' migrate it with grund config migrate --to 2 packages/alpha' },
]);
