#!/usr/bin/env python3
"""Hold each OKF bundle named on the command line to OKF v0.2's conformance section (§FS-examples.2).

The OKF example claims its bundles stay OKF bundles, and this is the claim's run:
the shared example runner invokes it through `command.external` (§FS-examples.5.4),
so the claim is pinned by a golden rather than checked once by hand. It holds a
bundle to OKF 11 and nothing more: every concept opens with a frontmatter block
carrying a non-empty `type`, every `index.md` follows OKF 8, and every `log.md`
follows OKF 9. It uses the standard library alone, so it reads a frontmatter block
as the top-level `key:` lines it opens with, never as YAML in full.
"""
import re
import sys
from pathlib import Path

KEY = re.compile(r'([A-Za-z_][A-Za-z0-9_-]*):(?:\s+(.*))?$')
ENTRY = re.compile(r'[*-] \[[^\]]+\]\([^)]+\)( - .+)?$')
DATE = re.compile(r'\d{4}-\d{2}-\d{2}$')


def plural(count, noun):
    """`1 log`, `7 concepts`."""
    return f'{count} {noun}' + ('' if count == 1 else 's')


def frontmatter(lines):
    """The top-level keys of a leading `---` block and the body's first index, or None (OKF 4).

    A line indented, a sequence item, blank or a comment continues the key above it.
    """
    if not lines or lines[0] != '---':
        return None, 0
    keys = {}
    for index, line in enumerate(lines[1:], start=1):
        if line == '---':
            return keys, index + 1
        if not line.strip() or line[0] in ' \t-#':
            continue
        match = KEY.match(line)
        if not match:
            raise ValueError(index + 1, f'frontmatter line is not `key: value`: {line}')
        keys[match.group(1)] = (match.group(2) or '').strip().strip('\'"')
    raise ValueError(1, 'frontmatter block opens with `---` and never closes')


def concept(lines):
    """A concept's problems: a frontmatter block with a non-empty `type` (OKF 4.1, 11.1, 11.2)."""
    keys, _ = frontmatter(lines)
    if keys is None:
        return [(1, 'concept does not open with a frontmatter block')]
    if not keys.get('type'):
        return [(1, 'frontmatter has no non-empty `type`')]
    return []


def index(lines, root):
    """An `index.md`'s problems: no frontmatter but the root's `okf_version`, entries under headings (OKF 8)."""
    keys, body = frontmatter(lines)
    problems = []
    if keys is not None and (not root or set(keys) - {'okf_version'}):
        allowed = 'only `okf_version` at the bundle root' if root else 'none below the bundle root'
        problems.append((1, f'index frontmatter carries {", ".join(sorted(keys))}; OKF 8 allows {allowed}'))
    heading = False
    for number, line in enumerate(lines[body:], start=body + 1):
        if line.startswith('#'):
            heading = True
        elif ENTRY.match(line):
            if not heading:
                problems.append((number, 'index entry before any section heading'))
        elif line.strip():
            problems.append((number, f'index line is neither a heading nor `* [Title](url) - description`: {line}'))
    return problems


def log(lines):
    """A `log.md`'s problems: `## YYYY-MM-DD` date headings, newest first (OKF 9)."""
    problems = []
    previous = None
    for number, line in enumerate(lines, start=1):
        if not line.startswith('## '):
            continue
        date = line[3:].strip()
        if not DATE.match(date):
            problems.append((number, f'log date heading is not YYYY-MM-DD: {date}'))
        elif previous is not None and date > previous:
            problems.append((number, f'log date {date} follows {previous}; entries are newest first'))
        else:
            previous = date
    return problems


def check(bundle):
    """Every problem in one bundle as `(path, line, message)`, and its concept, index and log counts."""
    problems = []
    counts = {'concept': 0, 'index file': 0, 'log': 0}
    for path in sorted(bundle.rglob('*.md')):
        relative = path.relative_to(bundle).as_posix()
        try:
            lines = path.read_text(encoding='utf-8').splitlines()
            if path.name == 'index.md':
                counts['index file'] += 1
                found = index(lines, root=path.parent == bundle)
            elif path.name == 'log.md':
                counts['log'] += 1
                found = log(lines)
            else:
                counts['concept'] += 1
                found = concept(lines)
        except UnicodeDecodeError:
            found = [(1, 'file is not UTF-8')]
        except ValueError as error:
            found = [error.args]
        problems.extend((relative, number, message) for number, message in found)
    return problems, counts


def main(argv):
    if not argv:
        print('usage: okf_conformance.py BUNDLE...', file=sys.stderr)
        return 2
    failed = False
    for name in argv:
        name = name.rstrip('/')
        bundle = Path(name)
        if not bundle.is_dir():
            print(f'{name}: not a directory', file=sys.stderr)
            return 2
        problems, counts = check(bundle)
        for relative, number, message in problems:
            print(f'{name}/{relative}:{number}: {message}')
        if problems:
            failed = True
            print(f'{name}: not conformant to OKF v0.2 ({plural(len(problems), "problem")})')
        else:
            summary = ', '.join(plural(count, noun) for noun, count in counts.items())
            print(f'{name}: conformant to OKF v0.2 ({summary})')
    return 1 if failed else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
