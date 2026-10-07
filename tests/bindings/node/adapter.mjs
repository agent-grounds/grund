// §FS-distribution.3.0.3 — test-only Node side of the complete-data protocol.
import * as api from 'grund-cli';
const request = JSON.parse(await new Promise(resolve => {
  let input = '';
  process.stdin.setEncoding('utf8');
  process.stdin.on('data', part => { input += part; });
  process.stdin.on('end', () => resolve(input));
}));
// Fixed host fields only: authored IDs, config/schema keys and details are opaque.
const names = Object.fromEntries([
  'runCautions', 'selectedReport', 'hadScanErrors', 'outputFormat', 'kindTitle',
  'expectedExit', 'scanErrors', 'enclosingDeclaration', 'enclosingSection',
  'sectionSeparator', 'valueRoots', 'refusedWrites', 'e2eCaseDir',
  'fileHoldsSingleDeclaration', 'pendingChanges', 'scanReadsFile', 'fsHome',
  'headingName', 'headingMarker', 'configFile', 'manualSteps', 'agentOverrides',
  'conversationTarget', 'installKind', 'configTarget', 'resolverTarget',
  'escapedCitations', 'scannedFiles', 'walkedDirs', 'fileStructures',
  'valueBindings', 'invalidValueDeclarations', 'invalidValueBindings',
  'sectionHeadingsOutsideDeclarations', 'unmarkedHeadings', 'nearMissHeadings',
  'headingLevel', 'duplicateSections', 'e2eCase', 'bodyStart', 'bodyEnd',
  'bodyHasContent', 'valueValid', 'localSection', 'shorthandRewritable',
  'numericRun', 'inlineSite', 'memberSlice', 'keyColumn', 'keyText',
  'sourceKind', 'suggestedPath', 'sourceSlice', 'valueRoot',
  'bindingNamespace', 'bindingSection', 'startLine', 'endLine',
  'isDocComment', 'isDocstring', 'lineCount', 'canonicalRelativePath',
  'firstLine', 'lastLine', 'maxColumns', 'hasNote', 'layoutViolations',
  'docComments', 'totalLines',
].map(name => [name, name.replace(/[A-Z]/g, c => '_' + c.toLowerCase())]));
function lower(value, opaque = false) {
  if (Array.isArray(value)) return value.map(item => lower(item, opaque));
  if (value === null || typeof value !== 'object') return value;
  return Object.fromEntries(Object.entries(value).map(([key, item]) => {
    if (!opaque && /[A-Z]/.test(key) && !names[key])
      throw Error('unmapped fixed host field: ' + key);
    return [opaque ? key : names[key] ?? key,
      lower(item, opaque || key === 'values' || key === 'details')];
  }));
}
function quoted(text) {
  return '"' + text.replace(/["\\\x00-\x1f]/g, char => ({
    '"': '\\"', '\\': '\\\\', '\n': '\\n', '\r': '\\r', '\t': '\\t',
  })[char] ?? '\\u' + char.charCodeAt(0).toString(16).padStart(4, '0')) + '"';
}
function canonical(value) {
  if (typeof value === 'string') return quoted(value);
  if (Array.isArray(value)) return '[' + value.map(canonical).join(',') + ']';
  if (value !== null && typeof value === 'object')
    return '{' + Object.keys(value).sort((a, b) => Buffer.compare(Buffer.from(a), Buffer.from(b)))
      .map(key => quoted(key) + ':' + canonical(value[key])).join(',') + '}';
  return JSON.stringify(value);
}
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
