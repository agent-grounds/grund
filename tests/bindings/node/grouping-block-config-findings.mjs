// §FS-check.4.11.3: at a workspace root, each include_root = false block the workspace
// expansion reaches has its config findings named: in check's report.warnings and in
// validateConfig's runCautions, as a member's are.
import assert from 'node:assert/strict';
import { cp, mkdtemp } from 'node:fs/promises';
import * as api from 'grund-cli';
const root = process.argv[2];
const cases = process.env.GRUND_TEST_REPO + '/tests/e2e/cases/';
async function copy(name) {
  const dir = await mkdtemp(root + '-' + name + '-');
  await cp(cases + name + '/repo', dir, { recursive: true });
  return dir;
}
const project = ({ code, path, message }) => ({ code, path, message });
const block = [{ code: 'config-version-1', path: null,
  message: 'group/grund.toml is version 1; migrate it with grund config migrate --to 2 group' }];
const checked = await api.check(await copy('workspace-grouping-block-version-1-check'));
assert.deepEqual(checked.report.warnings.map(project), block);
assert.deepEqual(checked.report.errors, []);
const validated = await api.validateConfig(
  { root: await copy('workspace-grouping-block-version-1-validate') });
assert.deepEqual(validated.runCautions.map(project), block);
