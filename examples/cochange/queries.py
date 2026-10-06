"""Complete released Grund query facts for §FS-cochange-recipe.snapshots."""
from pathlib import Path
import subprocess

from common import Refusal, error, matches, parse_json


class Facts:
    """Positive full-coordinate resolution precedes root matching (§FS-cochange-recipe.evidence)."""
    def __init__(self, snapshot, root, tracked, policy, executable, errors, findings):
        """Complete queries even on empty comparisons (§FS-cochange-recipe.snapshots)."""
        self.snapshot, self.root = snapshot, root
        self.policy, self.executable, self.errors = policy, executable, errors
        self.findings = findings
        self.cover, self.catalog, self.resolved = {}, {}, {}
        self.check()
        self.read_catalog()
        self.read_cover()
        for path in sorted(tracked):
            if matches(path, policy['source_paths'] + policy['test_paths']) and path not in self.cover:
                raise Refusal('scan-scope', f'{path}: no cover row; include the classified file '
                              'in the snapshot scan configuration.', path, query='cover')
        self.resolve()

    def clean(self, text):
        """Keep scratch identities out of reports (§FS-cochange-recipe.output)."""
        return text.replace(str(self.snapshot), '<snapshot>')

    def path(self, value, query):
        """Normalize output to Git paths (§FS-cochange-recipe.snapshots)."""
        if not isinstance(value, str):
            raise Refusal('query', f'{query}: missing path field; require complete output.', query=query)
        path = Path(value)
        if not path.is_absolute():
            path = self.root / path
        # Lexical identity retains the symlink's own tracked name.
        import os
        path = Path(os.path.abspath(path))
        if not path.is_relative_to(self.snapshot):
            raise Refusal('unsupported-input', f'{query}: fact path leaves Git snapshot.', query=query)
        return path.relative_to(self.snapshot).as_posix()

    def query(self, name, extra=(), stdin=None):
        """One scan per surface with complete NDJSON (§FS-cochange-recipe.snapshots)."""
        args = ([name, '--batch', '--format=json', '--brief', '--path', str(self.root)]
                if name == 'show' else [name, str(self.root), '--format=json'])
        try:
            result = subprocess.run([self.executable, *args, *extra], input=stdin,
                                    capture_output=True, text=True, cwd=self.root)
        except (OSError, UnicodeError) as exc:
            raise Refusal('check' if name == 'check' else 'query',
                          f'{name}: {self.clean(str(exc))}; correct query executable/output.', query=name)
        try:
            rows = [parse_json(line) for line in result.stdout.splitlines() if line.strip()]
            if any(not isinstance(row, dict) for row in rows):
                raise ValueError('non-object record')
        except ValueError as exc:
            raise Refusal('check' if name == 'check' else 'query',
                          f'{name}: malformed NDJSON ({exc}); require complete query output.', query=name)
        if result.returncode and name not in ('check', 'show'):
            raise Refusal('query', f'{name} exited {result.returncode}: '
                          f'{self.clean(result.stderr.strip())}; fix query/scan input.', query=name)
        return result, rows

    def check(self):
        """Independent check errors always survive waivers (§FS-cochange-recipe.snapshots)."""
        result, rows = self.query('check')
        for row in rows:
            if (any(key not in row for key in ('severity', 'path', 'line', 'code', 'message', 'sites', 'authority')) or
                    not isinstance(row.get('message'), str) or not isinstance(row.get('code'), str) or
                    row.get('severity') not in ('error', 'warning', 'suggestion')):
                raise Refusal('check', 'check: incomplete finding record; require complete output.', query='check')
            # Preserve warnings as facts without changing their verdict. The recipe's
            # error list remains the independent refusal/obligation channel.
            import json
            finding = parse_json(self.clean(json.dumps(row)))
            if finding.get('path'):
                finding['path'] = self.path(row['path'], 'check')
            self.findings.append(dict(snapshot=self.snapshot.name, finding=finding))
            if row['severity'] == 'error':
                self.errors.append(error('check', self.clean(row['message']),
                    self.path(row['path'], 'check') if row.get('path') else None, query='check'))
        if result.returncode:
            self.errors.append(error('check', f'check exited {result.returncode}: '
                f'{self.clean(result.stderr.strip()) or "standing errors above"}; correct the snapshot.', query='check'))

    def read_catalog(self):
        """Catalog identities retain project names (§FS-cochange-recipe.evidence)."""
        _, rows = self.query('list')
        for row in rows:
            if (not isinstance(row.get('id'), str) or not isinstance(row.get('kind'), str) or
                    not isinstance(row.get('line'), int) or not isinstance(row.get('duplicate'), bool)):
                raise Refusal('query', 'list: incomplete catalog identity; require complete output.', query='list')
            identity = row['id']
            project, bare = identity.rsplit('/', 1) if '/' in identity else (None, identity)
            if row.get('project') != project or identity in self.catalog or row['duplicate']:
                raise Refusal('query', f'list: duplicate or inconsistent catalog identity {identity}.', query='list')
            self.catalog[identity] = dict(project=project, id=bare, kind=row['kind'],
                                          path=self.path(row.get('path'), 'list'))

    def read_cover(self):
        """Missing scope differs from zero citations (§FS-cochange-recipe.snapshots)."""
        _, rows = self.query('cover')
        for row in rows:
            path = self.path(row.get('path'), 'cover')
            citations = row.get('citations')
            if (path in self.cover or not isinstance(citations, list) or
                    row.get('project') is not None and not isinstance(row['project'], str)):
                raise Refusal('query', f'cover: duplicate/incomplete row for {path}.', path, query='cover')
            coordinates = []
            for citation in citations:
                required = ('path', 'line', 'column', 'id', 'section', 'marker', 'text',
                            'enclosing_declaration', 'enclosing_section')
                if (not isinstance(citation, dict) or any(key not in citation for key in required) or
                        not isinstance(citation['id'], str) or not isinstance(citation['line'], int) or
                        not isinstance(citation['column'], int) or not isinstance(citation['marker'], bool) or
                        not isinstance(citation['text'], str) or
                        any(citation[key] is not None and not isinstance(citation[key], str)
                            for key in ('section', 'enclosing_declaration', 'enclosing_section')) or
                        self.path(citation['path'], 'cover') != path or
                        citation.get('project') != row.get('project')):
                    raise Refusal('query', f'cover: malformed citation in {path}.', path, query='cover')
                identity = citation['id']
                # Qualified cover IDs already use the loaded workspace registry; local IDs
                # use the citing row's project, rather than another member's same bare ID.
                if '/' not in identity and row.get('project') is not None:
                    identity = row['project'] + '/' + identity
                coordinates.append((identity, citation['section']))
            self.cover[path] = coordinates

    def resolve(self):
        """Ordered complete coordinate envelopes (§FS-cochange-recipe.snapshots)."""
        coordinates = sorted(set((identity, None) for identity in self.catalog) |
                             {coordinate for row in self.cover.values() for coordinate in row},
                             key=lambda pair: (pair[0], pair[1] or ''))
        import json
        queries = [dict(id=identity, section=section) for identity, section in coordinates]
        result, rows = self.query('show', stdin=''.join(json.dumps(q) + '\n' for q in queries))
        if len(rows) != len(queries):
            raise Refusal('query', 'show: incomplete or duplicate batch envelopes; '
                          'require one ordered result per query.', query='show')
        for query, envelope in zip(queries, rows):
            if (set(envelope) != {'query', 'ok', 'result', 'error'} or
                    envelope['query'] != query or not isinstance(envelope['ok'], bool)):
                raise Refusal('query', 'show: malformed/mismatched batch envelope.', query='show')
            if not envelope['ok']:
                detail = envelope['error']
                if not isinstance(detail, dict) or not isinstance(detail.get('message'), str):
                    raise Refusal('query', 'show: missing failed-result detail.', query='show')
                raise Refusal('resolution', f'{query["id"]}: {self.clean(detail["message"])}; '
                              'correct the full citation coordinate.', query='show')
            target = envelope['result']
            catalog = self.catalog.get(query['id'])
            if (not isinstance(target, dict) or not catalog or envelope['error'] is not None or
                    target.get('id') != catalog['id'] or target.get('section') != query['section'] or
                    not isinstance(target.get('body'), str) or not isinstance(target.get('line'), int) or
                    self.path(target.get('path'), 'show') != catalog['path']):
                raise Refusal('query', f'show: missing/mismatched result for {query["id"]}.', query='show')
            self.resolved[(query['id'], query['section'])] = catalog
        if result.returncode:
            raise Refusal('query', f'show exited {result.returncode}; require a successful complete batch.', query='show')

    def targets(self, path):
        """Use only direct, resolved eligible roots (§FS-cochange-recipe.evidence)."""
        identities = {}
        for coordinate in self.cover[path]:
            target = self.resolved[coordinate]
            if target['kind'] in self.policy['eligible_kinds']:
                identities[(target['project'], target['id'])] = target
        return identities
