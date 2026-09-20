"""§AR-rules.1 and §AR-rules.6 — parser, facts and engine are one-way.

The scanner cannot reach rules; sentence code cannot reach facts, evaluation,
deduplication or diagnostics; engine code cannot reach sentences, Markdown,
scanner/resolver records or filesystem layout; and every production file in
the component has one of the owners the architecture names. The component is
test-only today, and it cannot become a production module until all four
boundary drivers are enabled.
"""

import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CORE = REPO_ROOT / "crates" / "grund-core" / "src"
RULES = CORE / "rules"
LIB = CORE / "lib.rs"
OWNERS = {"sentence", "facts", "markdown", "engine"}
DRIVERS = (
    "sentence_front_end_returns_complete_parsed_rule_without_facts_or_diagnostics",
    "logic_engine_evaluates_hand_built_rule_and_facts_without_parser_or_scanner",
    "markdown_adapter_and_second_producer_drive_the_same_engine_result",
    "incomplete_fact_snapshot_suppresses_absence_and_count_conclusions",
)

# These patterns judge Rust paths and boundary types after comments and string
# literals are removed. They are intentionally narrower than a word search: a
# module comment may explain a forbidden dependency without importing it.
COMMENTS = re.compile(r"/\*.*?\*/|//[^\n]*", re.S)
STRINGS = re.compile(r'(?s)r#*".*?"#*|"(?:\\.|[^"\\])*"')
SCANNER_TO_RULES = re.compile(r"\b(?:crate|super(?::super)*)::rules\b")
SENTENCE_FORBIDDEN = (
    re.compile(r"\b(?:crate|super(?::super)*)::(?:rules::)?(?:facts|markdown|engine)\b"),
    re.compile(r"\bcrate::(?:scanner|checker)::"),
    re.compile(r"\b(?:RuleFacts|Diagnostic|Finding|Report)\b"),
)
ENGINE_FORBIDDEN = (
    re.compile(r"\b(?:crate|super(?::super)*)::(?:rules::)?(?:sentence|markdown)\b"),
    re.compile(r"\bcrate::(?:grammar|scanner|resolver|queries|writers)::"),
    re.compile(r"\bstd::(?:fs|path)::"),
    re.compile(r"\b(?:Path|PathBuf|Findings|Finding|Report|Declaration|Citation|SectionInfo)\b"),
    re.compile(r"\b(?:parse_rule|RuleParser|RuleSentence|sentence|title)\b"),
)
ADAPTER_FORBIDDEN = (
    re.compile(r"\b(?:crate|super(?::super)*)::(?:rules::)?(?:sentence|engine)\b"),
    re.compile(r"\b(?:parse_rule|evaluate_rule|deduplicate_rules)\b"),
)


def _code(path):
    text = path.read_text(encoding="utf-8")
    return STRINGS.sub('""', COMMENTS.sub("", text))


def _implementation_files(root):
    if not root.exists():
        return []
    return sorted(
        path
        for path in root.glob("**/*.rs")
        if path != root / "mod.rs" and not path.name.startswith("tests_")
    )


def _owner(path):
    relative = path.relative_to(RULES)
    first = relative.parts[0]
    if first.endswith(".rs"):
        first = first.removesuffix(".rs")
    return first if first in OWNERS else None


def _owned_files(owner):
    return [path for path in _implementation_files(RULES) if _owner(path) == owner]


def _production_rules_module():
    lines = LIB.read_text(encoding="utf-8").splitlines()
    for index, line in enumerate(lines):
        if line.strip() != "mod rules;":
            continue
        attributes = []
        cursor = index - 1
        while cursor >= 0 and lines[cursor].strip().startswith("#["):
            attributes.append(lines[cursor].strip())
            cursor -= 1
        if "#[cfg(test)]" not in attributes:
            return True
    return False


def _violations(paths, patterns):
    found = []
    for path in paths:
        code = _code(path)
        for pattern in patterns:
            match = pattern.search(code)
            if match:
                line = code.count("\n", 0, match.start()) + 1
                relative = path.relative_to(CORE).as_posix()
                found.append(f"{relative}:{line}: {match.group(0)}")
    return found


class RulesArchitectureTests(unittest.TestCase):
    def test_every_rule_implementation_file_has_one_architectural_owner(self):
        unowned = [
            path.relative_to(CORE).as_posix()
            for path in _implementation_files(RULES)
            if _owner(path) is None
        ]
        self.assertEqual([], unowned, "rules implementation files with no §AR-rules.1 owner")

    def test_scanner_cannot_import_rules(self):
        scanner = CORE / "scanner"
        files = [scanner / "mod.rs", *_implementation_files(scanner)]
        violations = _violations(files, (SCANNER_TO_RULES,))
        self.assertEqual([], violations, "scanner imports the rules component")

    def test_sentence_front_end_cannot_import_facts_engine_or_diagnostics(self):
        violations = _violations(_owned_files("sentence"), SENTENCE_FORBIDDEN)
        self.assertEqual([], violations, "sentence front end crosses §AR-rules.1")

    def test_logic_engine_cannot_import_sentence_markdown_scanner_or_filesystem(self):
        violations = _violations(_owned_files("engine"), ENGINE_FORBIDDEN)
        self.assertEqual([], violations, "logic engine crosses §AR-rules.1")

    def test_markdown_adapter_does_not_parse_or_evaluate_rules(self):
        violations = _violations(_owned_files("markdown"), ADAPTER_FORBIDDEN)
        self.assertEqual([], violations, "Markdown fact adapter crosses §AR-rules.1")

    def test_production_rules_component_has_no_pending_contract_drivers(self):
        pending = RULES / "tests_boundaries.rs"
        text = pending.read_text(encoding="utf-8")
        if not _production_rules_module():
            missing = [
                name
                for name in DRIVERS
                if not re.search(
                    rf"#\[ignore = \"implementation pending:[^\"]+\"\]\s*fn {name}\b",
                    text,
                )
            ]
            self.assertEqual([], missing, "pending architecture drivers lost their marker")
            return

        test_text = "\n".join(
            path.read_text(encoding="utf-8")
            for path in sorted(RULES.glob("**/tests_*.rs"))
        )
        missing = [name for name in DRIVERS if f"fn {name}" not in test_text]
        self.assertEqual([], missing, "production rules component lacks boundary drivers")
        self.assertNotIn(
            "implementation pending:",
            test_text,
            "production rules component still has ignored architecture sentinels",
        )


if __name__ == "__main__":
    unittest.main()
