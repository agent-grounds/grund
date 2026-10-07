// §FS-distribution.3.0.3 — exact command-specific projection; no message repair.
import * as api from 'grund-cli';
import { json, diagnostic, hit } from './wire-codec.mjs';
const request = JSON.parse(process.argv[3]);
const { operation, args, projection = 'detail' } = request;
const caution = warnings => {
  for (const w of warnings) process.stderr.write('warning: ' + w.message + '\n');
};
const emit = row => process.stdout.write(json(row) + '\n');
try {
  const result = await api[operation](...args);
  caution(result.runCautions);
  if (operation === 'show') process.stdout.write(result.json + '\n');
  else if (operation === 'showBatch') {
    for (const record of result.records) {
      const query = { id: record.query.id, section: record.query.section };
      if (record.ok)
        process.stdout.write('{"query":' + json(query) + ',"ok":true,"result":' +
          record.result.json + ',"error":null}\n');
      else emit({ query, ok: false, result: null, error: diagnostic(record.failure) });
    }
    process.exitCode = result.records.some(r => !r.ok) ? 1 : 0;
  } else if (operation === 'refs') {
    if (projection === 'total') emit({ sites: result.totals.sites, files: result.totals.files });
    else if (projection === 'summary') {
      for (const summary of result.summaries)
        emit({ ...(result.workspace ? { project: summary.project ?? '' } : {}),
          path: summary.path, count: summary.count, lines: summary.lines });
    } else for (const row of result.hits) emit(hit(row, result.workspace, result.kindTitle));
    if (result.note !== null) process.stderr.write('note: ' + result.note + '\n');
  } else if (operation === 'list') {
    for (const e of result.entries)
      emit({ ...(e.project === null ? {} : { project: e.project }), id: e.id,
        ...(e.section === null ? {} : { section: e.section }), kind: e.kind,
        path: e.path, line: e.line, title: e.title, stub: e.stub, defines: e.defines,
        refs: e.refs, duplicate: e.duplicate,
        ...(e.valueRoots.length ? { value_roots: e.valueRoots } : {}) });
  } else if (operation === 'listSizes') {
    for (const e of result.entries) {
      const row = { ...(e.project === null ? {} : { project: e.project }), id: e.id,
        section: e.section, kind: e.kind, path: e.path, line: e.line,
        stub: e.stub, defines: e.defines, duplicate: e.duplicate };
      for (const m of e.measurements) {
        row['lead_' + m.unit] = m.lead; row['full_' + m.unit] = m.full;
      }
      emit(row);
    }
  } else if (operation === 'cover') {
    for (const e of result.entries)
      emit({ ...(e.project === null ? {} : { project: e.project }), path: e.path,
        citations: e.citations.map(c => hit(c, c.project !== null)) });
  } else throw Error('unsupported test wire projection: ' + operation);
} catch (error) {
  if (!(error instanceof api.GrundError)) throw error;
  caution(error.failure.runCautions);
  if (error.failure.kind !== 'query') throw error; // no text-derived classification
  process.stderr.write(json(diagnostic(error.failure)) + '\n');
  process.exitCode = 1;
}
