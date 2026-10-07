// §FS-distribution.3.0.3 — the CLI renderer's control escapes and key order.
export function json(value) {
  if (typeof value === 'string')
    return '"' + value.replace(/["\\\x00-\x1f\x7f-\x9f]/g, char => ({
      '"': '\\"', '\\': '\\\\', '\n': '\\n', '\r': '\\r', '\t': '\\t',
    })[char] ?? '\\u' + char.charCodeAt(0).toString(16).padStart(4, '0')) + '"';
  if (Array.isArray(value)) return '[' + value.map(json).join(',') + ']';
  if (value !== null && typeof value === 'object')
    return '{' + Object.entries(value).map(([key, item]) => json(key) + ':' + json(item)).join(',') + '}';
  return JSON.stringify(value);
}
export function diagnostic(f) {
  return { severity: 'error', path: null, line: null, code: f.code,
    message: f.message,
    sites: f.sites.length ? f.sites.map(s => ({ path: s.path, line: s.line })) : null,
    authority: null };
}
export function hit(h, project, kindTitle = null) {
  return { ...(project ? { project: h.project ?? '' } : {}),
    path: h.path, line: h.line, column: h.column, id: h.id, section: h.section,
    marker: h.marker, text: h.text, enclosing_declaration: h.enclosingDeclaration,
    enclosing_section: h.enclosingSection,
    ...(kindTitle === null ? {} : { kind_title: kindTitle }) };
}
