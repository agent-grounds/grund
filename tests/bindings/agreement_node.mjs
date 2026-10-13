// Persistent actual Node entrypoints, without a Rust substitute
// (§REQ-one-answer.proof.matrix, §REQ-one-answer.proof.evidence).
import {createRequire} from 'node:module';
import readline from 'node:readline';
import path from 'node:path';
import {lower} from './node/adapter_codec.mjs';
const require = createRequire(import.meta.url);
const packageRoot = process.argv[2];
const api = require(path.join(packageRoot, 'index.cjs'));
for await (const line of readline.createInterface({input: process.stdin, crlfDelay: Infinity})) {
  const request = JSON.parse(line);
  if (request.control === 'cwd') {
    process.chdir(request.cwd);
    process.stdout.write(JSON.stringify({cwd: process.cwd()}) + '\n');
    continue;
  }
  const start = performance.now();
  process.chdir(request.cwd);
  let envelope;
  try {
    const {runCautions, ...result} = await api[request.operation](...request.args);
    envelope = {failure: null, result: lower(result), run_cautions: lower(runCautions)};
  } catch (error) {
    if (!(error instanceof api.GrundError)) throw error;
    const {runCautions, ...failure} = error.failure;
    envelope = {failure: lower(failure), result: null, run_cautions: lower(runCautions)};
  }
  process.stdout.write(JSON.stringify({request, envelope, seconds: (performance.now() - start) / 1000}) + '\n');
}
