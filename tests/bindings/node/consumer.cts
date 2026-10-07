// §FS-distribution.3.2.3.2 — typed require conditions select the CJS declaration.
import api = require('grund-cli');
const result: Promise<api.CheckResult> = api.check('./repo');
// @ts-expect-error enum checking must also hold in CJS
api.listSizes({ units: ['characters'] });
// @ts-expect-error data never exposes a live native iterator
result.snapshot;
async function consume() {
  const report = await result;
  const errors: readonly api.Finding[] = report.report.errors;
  // @ts-expect-error readonly TS result
  report.runCautions = [];
  return errors;
}
void consume;
