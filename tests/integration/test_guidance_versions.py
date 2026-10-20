"""§REQ-discoverable-offline.1: shipped help, the setup guide and what `grund init`
writes name a version-specific spelling only with its version, or one both
versions accept, and a written refusal holds only where that version's reader
refuses the spelling (§REQ-discoverable-offline.3).

Every spelling is taken from Markdown code spans and TOML fences, from bracketed
spellings in help outside its usage synopsis, and from init's `grund.toml` parsed
as TOML. The oracle is the binary: `grund config explain` exits 0 for a table or
key its version admits and 1 for one it does not (§FS-config-explain.1.1,
§FS-config-explain.5). An unlabelled spelling must exist in both versions, a
labelled one in its own. The controls run the same judgement on doctored copies
of a guide file, so a check that passes cannot pass by reading nothing.
"""

import functools
import os
import re
import subprocess
import tempfile
import tomllib
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
EXE = "grund.exe" if os.name == "nt" else "grund"
SKILL_COPIES = (
    REPO_ROOT / "skills" / "grund-init",
    REPO_ROOT / "crates" / "grund-core" / "assets" / "skills" / "grund-init",
)
# Each template and the flags that lay everything it writes; `none` has no
# starter files, so `--scaffold` is refused there.
TEMPLATES = (("software-development", ("--scaffold",)), ("none", ()))
RED = "red until #832 config explain lands; implement removes this"

# A table as a config writes it, then optionally a key or an inline member:
# `[schema.leads.words]`, `[[kinds]] fetch`, `[[kinds]].fetch`, `[schema.kinds.FS]`.
SEGMENT = r"(?:[A-Za-z0-9_-]+|<[A-Za-z]+>|\*)"
TABLE = rf"\[\[?[a-z_][a-z0-9_]*(?:\.{SEGMENT})*\]\]?"
KEY = r"(?:<[A-Za-z]+>|[a-z_][a-z0-9_-]*)(?:\.[a-z_][a-z0-9_]*)?"
# The whole of a code span: a table, its key, and a value written after it.
SPAN = re.compile(rf"^(?P<spelling>{TABLE}(?:[ .]{KEY})?)(?:\s*=.*)?$")
# In help's bare prose a spaced key counts only where nothing but punctuation
# follows it: `[reference] require_grounding;`, not `[[kinds]] row that`.
BARE = re.compile(rf"(?P<spelling>{TABLE}(?:\.{KEY}| {KEY}(?=\s*(?:=|;|,|\)|$)))?)")
CODE_SPAN = re.compile(r"`([^`\n]+)`")
# "version-1 file" names its version as "version 1 file" does.
LABEL = re.compile(r"\b(?:version[ -]([12])|v([12]))\b", re.IGNORECASE)
REFUSAL = re.compile(r"With `grund_config_version = ([12])` there is no `([^`]+)`")
HEADING = re.compile(r"^(#{1,6})\s+(.*)$")
FENCE = re.compile(r"^\s*(```|~~~)\s*([A-Za-z0-9_-]*)")
TOML_HEADER = re.compile(r"^\s*(\[\[?[^\]]+\]\]?)\s*(?:#.*)?$")
TOML_KEY = re.compile(r"^\s*([A-Za-z0-9_<>-]+(?:\.[A-Za-z0-9_<>-]+)*)\s*=")
TOML_VERSION = re.compile(r"^\s*grund_config_version\s*=\s*([12])\s*$", re.MULTILINE)
SENTENCE_END = re.compile(r"(?<=[.!?])\s+(?=[A-Z`(\[])")
LIST_ITEM = re.compile(r"^\s*(?:[-*+]|\d+\.)\s")


@functools.lru_cache(maxsize=None)
def _grund():
    """The binary under test: the gate's build, or this checkout's built once."""
    built = os.environ.get("GRUND_BUILT")
    if built:
        return built
    target = Path(os.environ.get("CARGO_TARGET_DIR") or REPO_ROOT / "target")
    subprocess.run([os.environ.get("CARGO", "cargo"), "build", "-p", "grund", "--locked"],
                   cwd=REPO_ROOT, check=True)
    return str(target / "debug" / EXE)


def _versions(text):
    """The versions a stretch of text names: "version 1", "v2", …"""
    return frozenset(int(one or two) for one, two in LABEL.findall(text))


class Spelling:
    """One spelling where guidance writes it, with the versions its local context
    names (empty for none), or the version a written refusal says refuses it."""

    def __init__(self, source, line, spelling, labels, refused=None):
        self.source, self.line, self.spelling = source, line, spelling
        self.labels, self.refused = labels, refused

    def __repr__(self):
        return f"{self.source}:{self.line}: {self.spelling}"


def _sentences(block):
    """A paragraph, list item or table row, cut into its sentences."""
    return SENTENCE_END.split(" ".join(line.strip() for line in block))


def _heading_label(headings):
    """The versions the nearest enclosing heading that names one names."""
    for _, text in reversed(headings):
        if _versions(text):
            return _versions(text)
    return frozenset()


def _toml_spellings(lines):
    """Each table and key a TOML fence writes, with the line it is on."""
    header = None
    for offset, line in enumerate(lines):
        if match := TOML_HEADER.match(line):
            header = match.group(1)
            yield offset, header
        elif match := TOML_KEY.match(line):
            key = match.group(1)
            yield offset, f"{header} {key}" if header else key


def markdown_spellings(text, source, written=frozenset()):
    """Every spelling a Markdown file writes, each with its local context's label:
    its sentence, or its whole list item or table row; the sentence introducing
    its fence; its nearest heading that names a version; else `written`, the
    version of the `grund.toml` a file was rendered for."""
    found = []
    headings = []
    lines = text.splitlines()
    block, block_start, whole = [], 0, False
    intro = frozenset()

    def flush():
        nonlocal block, intro
        if not block:
            return
        sentences = [" ".join(block)] if whole else _sentences(block)
        for sentence in sentences:
            refusal = REFUSAL.search(sentence)
            if refusal:
                found.append(Spelling(source, block_start, refusal.group(2),
                                      frozenset(), int(refusal.group(1))))
            for span in CODE_SPAN.findall(sentence):
                match = SPAN.match(span.strip())
                if not match or (refusal and span == refusal.group(2)):
                    continue
                labels = _versions(sentence) or _heading_label(headings) or written
                found.append(Spelling(source, block_start, match.group("spelling"), labels))
        intro = _versions(sentences[-1])
        block = []

    index = 0
    while index < len(lines):
        line = lines[index]
        fence = FENCE.match(line)
        if fence:
            flush()
            closing = next((end for end in range(index + 1, len(lines))
                            if lines[end].strip().startswith(fence.group(1))), len(lines))
            body = lines[index + 1:closing]
            if fence.group(2).lower() == "toml":
                own = TOML_VERSION.search("\n".join(body))
                labels = (frozenset({int(own.group(1))}) if own
                          else intro or _heading_label(headings) or written)
                for offset, spelling in _toml_spellings(body):
                    found.append(Spelling(source, index + 2 + offset, spelling, labels))
            intro = frozenset()
            index = closing + 1
            continue
        heading = HEADING.match(line)
        if heading:
            flush()
            depth = len(heading.group(1))
            headings = [h for h in headings if h[0] < depth] + [(depth, heading.group(2))]
            intro = frozenset()
        elif not line.strip():
            flush()
        elif LIST_ITEM.match(line) or line.lstrip().startswith("|"):
            flush()
            block, block_start, whole = [line], index + 1, True
            if line.lstrip().startswith("|"):
                flush()
        else:
            if not block:
                block_start, whole = index + 1, False
            block.append(line)
        index += 1
    flush()
    return found


def help_spellings(text, source):
    """Every spelling a help page writes outside its usage synopsis, each labelled
    by the line it is on: a help line is one sentence or one list item."""
    found = []
    in_usage = False
    for number, line in enumerate(text.splitlines(), 1):
        if line.startswith("Usage:"):
            in_usage = True
        elif not line.strip():
            in_usage = False
        if in_usage:
            continue
        spans = [SPAN.match(span.strip()) for span in CODE_SPAN.findall(line)]
        spellings = [match.group("spelling") for match in spans if match]
        prose = CODE_SPAN.sub(" ", line)
        spellings += [match.group("spelling") for match in BARE.finditer(prose)]
        for spelling in spellings:
            found.append(Spelling(source, number, spelling, _versions(line)))
    return found


def _init_config(root):
    """The `grund.toml` init wrote, parsed, and the version it is read as."""
    config = tomllib.loads((root / "grund.toml").read_text(encoding="utf-8"))
    return config, frozenset({config.get("grund_config_version", 1)})


def init_spellings(root, source):
    """Every table and key a `grund.toml` init wrote, labelled by its own version."""
    config, labels = _init_config(root)
    found = []

    def walk(table, path):
        for key, value in table.items():
            if isinstance(value, dict):
                header = f"[{'.'.join(path + [key])}]"
                found.append(Spelling(source, 0, header, labels))
                walk(value, path + [key])
            elif isinstance(value, list) and value and all(isinstance(v, dict) for v in value):
                header = f"[[{'.'.join(path + [key])}]]"
                found.append(Spelling(source, 0, header, labels))
                for row in value:
                    for row_key in row:
                        found.append(Spelling(source, 0, f"{header} {row_key}", labels))
            else:
                at = f"[{'.'.join(path)}] {key}" if path else key
                found.append(Spelling(source, 0, at, labels))

    walk(config, [])
    return found


class Oracle:
    """`grund config explain` in a version-1 and a version-2 project outside any
    checkout: whether that version admits a spelling."""

    def __init__(self):
        self._dirs = tempfile.TemporaryDirectory(prefix="grund-guidance-versions-")
        self.roots = {}
        for version in (1, 2):
            root = Path(self._dirs.name) / f"v{version}"
            root.mkdir()
            (root / "grund.toml").write_text(f"grund_config_version = {version}\n", encoding="utf-8")
            self.roots[version] = root

    def close(self):
        self._dirs.cleanup()

    @functools.lru_cache(maxsize=None)
    def admits(self, version, spelling):
        run = subprocess.run([_grund(), "config", "explain", spelling], cwd=self.roots[version],
                             capture_output=True, text=True)
        if run.returncode not in (0, 1):
            raise AssertionError(f"`grund config explain '{spelling}'` in version {version} "
                                 f"exited {run.returncode}: {run.stderr.strip()}")
        return run.returncode == 0


def judge(spellings, oracle):
    """What is wrong with each spelling, as `source:line: spelling: why`."""
    problems = []
    for found in spellings:
        if found.refused is not None:
            if oracle.admits(found.refused, found.spelling):
                problems.append(f"{found!r}: version {found.refused} admits it; the refusal is false")
        elif not found.labels:
            missing = [v for v in (1, 2) if not oracle.admits(v, found.spelling)]
            if missing:
                where = " or ".join(f"version {v}" for v in missing)
                problems.append(f"{found!r}: unlabelled, and {where} does not admit it")
        elif not any(oracle.admits(v, found.spelling) for v in sorted(found.labels)):
            named = " or ".join(f"version {v}" for v in sorted(found.labels))
            problems.append(f"{found!r}: labelled {named}, which does not admit it")
    return problems


def help_pages():
    """The top-level page and each command's, as `(source, text)`."""
    top = subprocess.run([_grund(), "--help"], capture_output=True, text=True, check=True).stdout
    commands = re.findall(r"^  ([a-z][a-z-]+)\s{2,}", top, re.MULTILINE)
    pages = [("grund --help", top)]
    for command in commands:
        page = subprocess.run([_grund(), command, "--help"], capture_output=True, text=True)
        pages.append((f"grund {command} --help", page.stdout + page.stderr))
    return pages


def skill_files():
    """Every file of the setup skill, in both of its copies."""
    return [path for copy in SKILL_COPIES for path in sorted(copy.rglob("*.md"))]


def guidance_spellings(work):
    """Every spelling of shipped guidance: help, both skill copies, and init's output."""
    found = []
    for source, text in help_pages():
        found += help_spellings(text, source)
    for path in skill_files():
        found += markdown_spellings(path.read_text(encoding="utf-8"), path.relative_to(REPO_ROOT))
    for template, flags in TEMPLATES:
        root = Path(work) / template
        root.mkdir()
        subprocess.run([_grund(), "init", "--no-vcs", *flags, "--template", template, str(root)],
                       capture_output=True, text=True, check=True)
        found += init_spellings(root, f"grund init --template {template}: grund.toml")
        _, written = _init_config(root)
        for path in sorted(root.rglob("*.md")):
            text = path.read_text(encoding="utf-8")
            found += markdown_spellings(text, f"grund init --template {template}: "
                                              f"{path.relative_to(root)}", written)
    return found


def _guide_refusals():
    """The refusals the setup guide itself writes, in its first copy."""
    path = SKILL_COPIES[0] / "references" / "languages.md"
    found = markdown_spellings(path.read_text(encoding="utf-8"), path.relative_to(REPO_ROOT))
    return [spelling for spelling in found if spelling.refused is not None]


class GuidanceExtraction(unittest.TestCase):
    """The extraction alone, which needs no oracle: what the judgement is asked."""

    def test_a_code_span_takes_its_label_from_its_sentence_or_heading(self):
        text = ("## Version 1\n\nSet `[scan] include` here.\n\n"
                "## Both\n\nIn version 2, `[schema.sources]` holds it. Use `[workspace] members`.\n")
        found = {(s.spelling, s.labels) for s in markdown_spellings(text, "doc")}
        self.assertEqual({("[scan] include", frozenset({1})),
                          ("[schema.sources]", frozenset({2})),
                          ("[workspace] members", frozenset())}, found)

    def test_a_toml_fence_is_labelled_by_its_own_version_or_its_introduction(self):
        text = ("For a version-1 file:\n\n```toml\n[[kinds]]\nkind = \"FS\"\n```\n\n"
                "```toml\ngrund_config_version = 2\n[schema.kinds.FS]\nfolders = [\"docs\"]\n```\n")
        found = {(s.spelling, s.labels) for s in markdown_spellings(text, "doc")}
        self.assertIn(("[[kinds]] kind", frozenset({1})), found)
        self.assertIn(("[schema.kinds.FS] folders", frozenset({2})), found)

    def test_a_written_refusal_is_read_as_one(self):
        text = "With `grund_config_version = 2` there is no `[scan]`.\n"
        [found] = markdown_spellings(text, "doc")
        self.assertEqual(("[scan]", 2), (found.spelling, found.refused))

    def test_the_guides_own_refusal_is_read_as_one(self):
        self.assertTrue(_guide_refusals(), "the guide writes no refusal")

    def test_help_prose_yields_its_bracketed_spellings_outside_the_synopsis(self):
        text = ("Usage:  grund check [PATH] [--only CODE]\n\n"
                "  --require-grounding  ([reference] require_grounding; a [[kinds]] row that stays).\n"
                "set [fmt.cross_refs].enabled = false to opt out\n")
        found = [s.spelling for s in help_spellings(text, "help")]
        self.assertEqual(["[reference] require_grounding", "[[kinds]]",
                          "[fmt.cross_refs].enabled"], found)

    def test_both_skill_copies_are_read_and_yield_spellings(self):
        for copy in SKILL_COPIES:
            config = copy / "references" / "config.md"
            self.assertIn(config, skill_files())
            self.assertTrue(markdown_spellings(config.read_text(encoding="utf-8"), config))


@unittest.skip(RED)
class GuidanceVersions(unittest.TestCase):
    """§REQ-discoverable-offline.1, judged by the binary."""

    @classmethod
    def setUpClass(cls):
        cls.oracle = Oracle()

    @classmethod
    def tearDownClass(cls):
        cls.oracle.close()

    def test_every_spelling_exists_in_the_version_its_context_names(self):
        with tempfile.TemporaryDirectory(prefix="grund-guidance-init-") as work:
            problems = judge(guidance_spellings(work), self.oracle)
        self.assertEqual([], problems, f"{len(problems)} spellings")

    def _doctored(self, sentence):
        """The problems one sentence adds to a copy of a real guide file."""
        path = SKILL_COPIES[0] / "references" / "config.md"
        text = path.read_text(encoding="utf-8")
        before = judge(markdown_spellings(text, "copy"), self.oracle)
        after = judge(markdown_spellings(f"{text}\n\n{sentence}\n", "copy"), self.oracle)
        return [problem for problem in after if problem not in before]

    def test_control_the_guides_own_refusal_passes(self):
        self.assertEqual([], judge(_guide_refusals(), self.oracle))

    def test_control_a_false_refusal_fails(self):
        added = self._doctored("With `grund_config_version = 2` there is no `[schema.sources]`.")
        self.assertEqual(1, len(added), added)

    def test_control_an_unlabelled_v1_spelling_fails(self):
        self.assertEqual(1, len(self._doctored("Set `[scan] include` to the roots it reads.")))

    def test_control_a_misspelt_spelling_fails_even_when_labelled(self):
        self.assertEqual(1, len(self._doctored("Version 1 reads `[scna] include`.")))


if __name__ == "__main__":
    unittest.main()
