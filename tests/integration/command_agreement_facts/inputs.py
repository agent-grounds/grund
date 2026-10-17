"""Pin a manifest's `inputFiles` to its fixture and keep its layout stable.

Reads only the input case directory's bytes, directories and symlink targets,
the inventory the loader compares (§REQ-one-answer.proof.evidence); it imports
nothing that answers a question, and leaves every authored fact as written
(§AR-system.principle). Usage: `python3 inputs.py <case>...`.
"""

import json
import os
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
CASES = HERE.parents[1] / 'e2e/cases'


def inventory(root):
    files = {}
    for directory, dirs, names in os.walk(root):
        for name in sorted(dirs + names):
            path = Path(directory) / name
            relative = path.relative_to(root).as_posix()
            if path.is_symlink():
                files[relative] = {'symlink': os.readlink(path).replace('\\', '/')}
            elif path.is_dir():
                files[relative] = {'directory': True}
            else:
                files[relative] = {'bytes': list(path.read_bytes())}
        dirs[:] = [d for d in dirs if not (Path(directory) / d).is_symlink()]
    return dict(sorted(files.items()))


def compact(value):
    return json.dumps(value, ensure_ascii=False, separators=(',', ':'))


def block(value, indent):
    """One authored entry per line: a list of objects, or an object's keys."""
    pad = ' ' * indent
    if isinstance(value, list) and value and all(isinstance(v, dict) for v in value):
        return '[\n' + ',\n'.join(pad + '  ' + compact(v) for v in value) + '\n' + pad + ']'
    if isinstance(value, dict) and value:
        return '{\n' + ',\n'.join(pad + '  ' + compact(k) + ': ' + compact(v)
                                  for k, v in value.items()) + '\n' + pad + '}'
    return compact(value)


def render(document):
    lines = []
    for key, value in document.items():
        if key == 'facts':
            body = ',\n'.join(f'    {compact(k)}: {block(v, 4)}' for k, v in value.items())
            lines.append(f'  "facts": {{\n{body}\n  }}')
        else:
            lines.append(f'  {compact(key)}: {block(value, 2)}')
    return '{\n' + ',\n'.join(lines) + '\n}\n'


def main(names):
    for name in names:
        manifest = HERE / f'{name}.json'
        document = json.loads(manifest.read_text(encoding='utf-8'))
        assert document['case'] == name, manifest
        document['inputFiles'] = inventory(CASES / name)
        order = ['version', 'case', 'inputFiles', 'facts', 'variants']
        document = {k: document[k] for k in order if k in document}
        manifest.write_text(render(document), encoding='utf-8')


if __name__ == '__main__':
    main(sys.argv[1:])
