// §FS-distribution.3.0.3 — test-only Node side of the complete-data protocol.
import * as api from 'grund-cli';
import {lower, canonical} from './adapter_codec.mjs';
const request = JSON.parse(await new Promise(resolve => {
  let input = '';
  process.stdin.setEncoding('utf8');
  process.stdin.on('data', part => { input += part; });
  process.stdin.on('end', () => resolve(input));
}));
let envelope;
try {
  const { runCautions, ...result } = await api[request.operation](...request.args);
  envelope = { failure: null, result: lower(result), run_cautions: lower(runCautions) };
} catch (error) {
  if (!(error instanceof api.GrundError)) throw error;
  const { runCautions, ...failure } = error.failure;
  envelope = { failure: lower(failure), result: null, run_cautions: lower(runCautions) };
}
process.stdout.write(canonical(envelope) + '\n');
