// §FS-distribution.3.2.2 — strict NodeNext consumers and readonly records.
import * as api from 'grund-cli';
const root = './repo';
const check = await api.check(root, { suggestions: true, only: ['dangling'] });
const code: string = check.report.errors[0]!.code;
const column: number | null = check.report.errors[0]!.column;
// @ts-expect-error result fields are readonly
check.hadScanErrors = false;
// @ts-expect-error readonly finding collection
check.report.errors.push(check.report.errors[0]!);
// @ts-expect-error unknown options
api.check(root, { imagined: true });
// @ts-expect-error wrong enum
api.show('FS-example', { mode: 'wide' });
// @ts-expect-error explicit scan operand is mandatory
api.scan();
// @ts-expect-error nullable outputs do not admit null roots
api.effectiveConfig({ root: null });
await api.scan(root);
await api.show('FS-example', { root, section: null, mode: 'brief', format: 'json' });
await api.showBatch([{ id: 'FS-example', section: null }], { root });
await api.showBatch(null, { root });
await api.refs('FS-example', { root, descendants: true });
await api.list({ root, kinds: ['FS'], projects: [], unused: true, selector: null });
await api.listSizes({ root, units: ['bytes', 'words'], top: 1 });
await api.cover({ root });
await api.fmt({ root, write: false, marker: true, crossRefs: false });
await api.proposeId('FS', 'Example', { root, width: 3 });
await api.init(root, { name: null, description: null, docs: false, force: false,
  dryRun: true, check: false, noVcs: true, agents: ['canonical'] });
await api.completeIds({ root, prefix: 'FS-', sections: true });
await api.validateConfig({ root });
await api.fetch('FS-example', { root });
await api.integrations({ client: 'vscode', write: true,
  conversation: 'link', conversationTarget: 'file', agent: null });
await api.referenceStyle(root + '/document.md');
await api.agentSetupInstructions();
try { await api.show('FS-missing', { root }); }
catch (error) {
  if (error instanceof api.GrundError) {
    const kind: api.Failure['kind'] = error.failure.kind;
    const partial = error.failure.partial;
    if (partial?.operation === 'fmt') {
      const label: string = partial.result.changes[0]!.label;
      void label;
    }
    void kind;
  }
}
void code; void column;
