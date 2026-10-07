// §FS-distribution.3.0.3 — frozen check wire projection, separate from host data.
import * as api from 'grund-cli';
function escape(text) {
  return '"' + text.replace(/["\\\x00-\x1f\x7f-\x9f]/g, char => ({
    '"': '\\"', '\\': '\\\\', '\n': '\\n', '\r': '\\r', '\t': '\\t',
  })[char] ?? '\\u' + char.charCodeAt(0).toString(16).padStart(4, '0')) + '"';
}
function json(value) {
  if (typeof value === 'string') return escape(value);
  if (Array.isArray(value)) return '[' + value.map(json).join(',') + ']';
  if (value !== null && typeof value === 'object')
    return '{' + Object.entries(value).map(([k, v]) => escape(k) + ':' + json(v)).join(',') + '}';
  return JSON.stringify(value);
}
const result = await api.check(process.argv[2], JSON.parse(process.argv[3] || '{}'));
for (const caution of result.runCautions) process.stderr.write('warning: ' + caution.message + '\n');
const findings = [...result.selectedReport.warnings, ...result.selectedReport.errors,
                  ...result.selectedReport.suggestions];
findings.sort((a, b) =>
  (a.path === null ? (b.path === null ? 0 : -1) : b.path === null ? 1 :
    Buffer.compare(Buffer.from(a.path), Buffer.from(b.path))) ||
  (a.line ?? 0) - (b.line ?? 0) ||
  Buffer.compare(Buffer.from(a.message), Buffer.from(b.message)));
for (const finding of findings) {
  const wire = {
    ...(finding.channel ? { channel: finding.channel } : { severity: finding.severity }),
    path: finding.path, line: finding.line, code: finding.code, message: finding.message,
    sites: finding.sites.length ? finding.sites.map(s => ({ path: s.path, line: s.line })) : null,
    authority: finding.authority.length ? finding.authority : null,
  };
  (finding.line === null ? process.stderr : process.stdout).write(json(wire) + '\n');
}
process.exitCode = result.selectedReport.errors.length || result.hadScanErrors ? 1 : 0;
