"""§AR-bindings.2.1, §FS-distribution.3.1.2 — a frontend is another crate, so each
`match` it makes on a promised enum ends in a wildcard arm, and that arm takes
away the compiler's prompt when grund-core adds a variant. Whatever the arm
renders — what core gave that does not depend on the variant, or, with nothing
left, `unnamed_variant::<E>()` — the `match` that holds it names every variant
`E` has today in its own patterns, so a variant added in this workspace is red
here until the frontend renders it. Comments and literals are blanked before
anything is read and a guard is no pattern, so neither names a variant; a
variant a frontend only constructs, looks up, or asks after in a `let … else`
or `matches!` is in no `match` and holds no arm."""

import re
import unittest

from test_public_surface_inventory import (
    FROZEN, REPO_ROOT, _core_sources, _sample, inventory_dispositions,
)


FRONTENDS = ("grund-cli", "grund-lsp", "grund-py", "grund-node")
ARM = re.compile(r"\bunnamed_variant::<([A-Za-z_][A-Za-z0-9_]*)>\(")
# A comment, a raw or plain string literal, or a char literal; a lifetime is none.
UNREAD = re.compile(
    r"//[^\n]*|/\*.*?\*/|(?<![A-Za-z0-9_])b?r(#*)\".*?\"\1|\"(?:\\.|[^\"\\])*\""
    r"|'(?:\\u\{[0-9A-Fa-f]+\}|\\.|[^\\'\n])'",
    re.DOTALL,
)
VARIANT = re.compile(r"\b([A-Z][A-Za-z0-9_]*)::([A-Z][A-Za-z0-9_]*)")
OPEN, CLOSE = "([{", ")]}"


def blank_unread(text):
    """`text` with each comment and literal blanked to spaces, its newlines kept."""
    return UNREAD.sub(lambda m: re.sub(r"[^\n]", " ", m.group(0)), text)


def _close(text, at):
    """The index just past the bracket that closes the one at `at`."""
    depth = 0
    for i in range(at, len(text)):
        depth += (text[i] in OPEN) - (text[i] in CLOSE)
        if depth == 0:
            return i + 1
    return len(text)


def _skip_expression(body, at):
    """The index past the arm expression that starts at `at`, and its comma."""
    while at < len(body) and body[at].isspace():
        at += 1
    if body.startswith("{", at):
        at = _close(body, at)
        while at < len(body) and body[at].isspace():
            at += 1
        return at + 1 if body.startswith(",", at) else at
    depth = 0
    while at < len(body) and not (body[at] == "," and depth == 0):
        depth += (body[at] in OPEN) - (body[at] in CLOSE)
        at += 1
    return at + 1


def arm_patterns(body):
    """The pattern of each arm of a `match` body, its `if` guard cut off."""
    patterns, start, depth, at = [], 0, 0, 0
    while at < len(body):
        if depth == 0 and body.startswith("=>", at):
            patterns.append(re.split(r"\bif\b", body[start:at], maxsplit=1)[0])
            at = start = _skip_expression(body, at + 2)
            continue
        depth += (body[at] in OPEN) - (body[at] in CLOSE)
        at += 1
    return patterns


def matches_in(text):
    """`(line, patterns)` for each `match` expression of blanked Rust source."""
    for keyword in re.finditer(r"\bmatch\b", text):
        depth, at = 0, keyword.end()
        while at < len(text) and not (text[at] == "{" and depth == 0):
            depth += (text[at] in "([") - (text[at] in ")]")
            at += 1
        body = text[at + 1:_close(text, at) - 1]
        yield text.count("\n", 0, keyword.start()) + 1, arm_patterns(body)


def named_variants(patterns):
    """`{enum: {variant, …}}` for each `Enum::Variant` path the patterns name."""
    named = {}
    for pattern in patterns:
        for enum, variant in VARIANT.findall(pattern):
            named.setdefault(enum, set()).add(variant)
    return named


def frontend_sources():
    """`(path, blanked text)` for each Rust source of a frontend."""
    for frontend in FRONTENDS:
        for path in sorted((REPO_ROOT / "crates" / frontend / "src").rglob("*.rs")):
            yield path, blank_unread(path.read_text(encoding="utf-8"))


def core_enums():
    """grund-core's `pub enum`s, each with the variants read at its definition."""
    enums = {}
    for _, text in _core_sources():
        for name, body in re.findall(
            r"^pub enum (\w+)\b[^{]*\{\n(.*?)^\}", text, re.MULTILINE | re.DOTALL
        ):
            enums[name] = set(re.findall(r"^    ([A-Z][A-Za-z0-9_]*)\b", body, re.MULTILINE))
    return enums


class FrontendWildcardArmTests(unittest.TestCase):
    """§FS-distribution.3.1.2: a wildcard arm is for a variant this release does
    not name, never for one it does."""

    @classmethod
    def setUpClass(cls):
        frozen = {n for n, d in inventory_dispositions().items() if d == FROZEN}
        cls.promised = {n: v for n, v in core_enums().items() if n in frozen}
        cls.sources = list(frontend_sources())
        cls.held = [
            (path, line, enum, named)
            for path, text in cls.sources
            for line, patterns in matches_in(text)
            for enum, named in named_variants(patterns).items()
            if enum in cls.promised
        ]

    def test_each_unnamed_variant_arm_is_in_a_match_this_guard_holds(self):
        refused = {name for _, text in self.sources for name in ARM.findall(text)}
        self.assertTrue(refused, "no `unnamed_variant::<…>()` call in any frontend")
        held = {enum for _, _, enum, _ in self.held}
        self.assertEqual("", _sample(refused - held), "unnamed_variant arms in no held match")

    def test_each_match_on_a_promised_enum_names_every_variant_of_today(self):
        for path, line, enum, named in self.held:
            where = f"{path.relative_to(REPO_ROOT).as_posix()}:{line}"
            with self.subTest(match=where, enum=enum):
                unnamed = self.promised[enum] - named
                self.assertEqual("", _sample(unnamed), f"{enum} variants only its wildcard arm renders")

    def test_a_comment_a_literal_a_guard_or_a_construction_names_no_variant(self):
        source = """
fn f(kind: Kind, note: Option<&Note>) -> String {
    // Kind::Commented is not rendered yet.
    let _ = ("Kind::Quoted", Kind::Built, '"');
    match kind {
        Kind::Here if kind == Kind::Guarded => "{".into(),
        Kind::There { at, .. } => { format!("{at}") }
        _ => match note { Some(Note::Inner) => "a", _ => "}" }.into(),
    }
}"""
        found = [named_variants(patterns) for _, patterns in matches_in(blank_unread(source))]
        self.assertEqual([{"Kind": {"Here", "There"}}, {"Note": {"Inner"}}], found)


if __name__ == "__main__":
    unittest.main()
