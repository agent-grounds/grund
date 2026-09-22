"""§AR-core-module-layout.1.5 — every owner and destination in the approved
core split exists, has its configured fissile line measurement, fits its soft
budget, and retains the pre-split Rust test inventory at its component owner.

The pre-commit gate checks formatting before the Python suite, so these are the
post-format measurements required by the ownership point. The Rust workspace
suite remains responsible for executing the named cases and their assertions.
"""

import json
import os
import re
import subprocess
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
FISSILE = os.environ.get("FISSILE_BIN", "fissile")

# Six original owners and seven split destinations. Keeping the configured
# rule and soft limit beside every path makes a budget or classification change
# an explicit contract edit rather than an accidental way to turn this green.
OWNERSHIP = (
    ("crates/grund-core/src/writers/fmt_rewrite.rs", "core-source", 350),
    ("crates/grund-core/src/writers/fmt_tree.rs", "core-source", 350),
    ("crates/grund-core/src/checker/references.rs", "core-source", 350),
    ("crates/grund-core/src/checker/reference_scope.rs", "core-source", 350),
    ("crates/grund-core/src/api/tests_embedding.rs", "tests", 450),
    ("crates/grund-core/src/api/tests_init_guidance.rs", "tests", 450),
    ("crates/grund-core/src/writers/tests_integrations.rs", "tests", 450),
    ("crates/grund-core/src/writers/tests_init_agents.rs", "tests", 450),
    ("crates/grund-core/src/writers/tests_agent_entrypoints.rs", "tests", 450),
    ("crates/grund-core/src/checker/tests_grounding_style.rs", "tests", 450),
    ("crates/grund-core/src/config/tests_validation.rs", "tests", 450),
    ("crates/grund-core/src/scanner/tests_qualified_citations.rs", "tests", 450),
    ("crates/grund-core/src/workspace/tests_scope.rs", "tests", 450),
)

# The exact 66-test baseline, assigned to the owners approved for each subject.
# A new test is allowed; losing or leaving one outside its approved owner is not.
EXPECTED_TESTS_BY_PATH = {
    "crates/grund-core/src/api/tests_embedding.rs": {
        "list_summary_reports_single_file_kind_home",
        "public_check_api_returns_relative_slash_normalized_paths",
        "public_embedding_api_checks_and_shows_without_cli_dispatch",
        "validate_config_at_a_workspace_root_fails_on_a_broken_member",
    },
    "crates/grund-core/src/api/tests_init_guidance.rs": {
        "e2e_readme_scaffold_uses_effective_fs_home",
        "init_next_guidance_uses_effective_custom_fs_file",
        "init_next_guidance_uses_effective_legacy_fs_home",
    },
    "crates/grund-core/src/writers/tests_integrations.rs": {
        "claude_symlink_to_agents_md_is_detected",
        "codium_installs_into_its_own_extensions_root",
        "conversation_target_is_recorded_independently",
        "integrations_block_appends_then_is_idempotent",
        "integrations_block_consumes_complete_indented_marker_lines",
        "integrations_block_lookup_is_dialect_scoped",
        "integrations_block_rejects_multiple_blocks",
        "integrations_block_rejects_newer_version",
        "integrations_block_rejects_orphan_begin_marker",
        "integrations_block_updates_in_place",
        "integrations_block_upgrades_older_version_in_place",
        "integrations_block_markers_match_the_host_language",
        "link_instruction_names_the_effective_target",
        "link_support_gates_unverified_targets_to_path",
        "resolver_peek_prints_the_declaration_without_opening_an_editor",
        "symlinked_claude_entrypoint_is_reported_only_under_the_link_opinion",
        "wezterm_fresh_install_is_a_working_config",
        "wezterm_wiring_note_reads_only_outside_the_block",
    },
    "crates/grund-core/src/writers/tests_init_agents.rs": {
        "agent_setup_instructions_match_the_distributable_skill",
        "agents_guidance_uses_configured_section_separator",
        "agents_update_appends_managed_block_when_missing",
        "agents_update_does_not_append_current_block_twice",
        "agents_update_handles_crlf_line_endings",
        "agents_update_keeps_current_block_in_middle_position",
        "agents_update_migrates_legacy_block_to_delimited_form",
        "agents_update_preserves_non_heading_content_after_delimited_block",
        "agents_update_refuses_malformed_delimiters",
        "agents_update_rewrites_current_block_from_rendered_template",
        "check_reports_malformed_agents_block",
        "embedded_templates_are_lf_canonical",
        "rendered_block_citation_example_is_escaped",
    },
    "crates/grund-core/src/writers/tests_agent_entrypoints.rs": {
        "an_unclaimed_generic_file_is_not_its_agent_s_entrypoint",
        "check_ignores_companion_agent_entrypoints_without_canonical_agents_md",
        "check_ignores_unmanaged_generic_rules_without_zed_workspace",
        "check_validates_managed_companion_without_canonical_agents_md",
        "check_validates_managed_zed_rules_without_canonical_agents_md",
        "check_validates_zed_workspace_rules_when_canonical_exists",
        "discovers_known_companion_agent_entrypoints",
        "init_discovers_missing_aliases_for_existing_agent_workspaces",
        "init_requests_one_entrypoint_per_agent",
    },
    "crates/grund-core/src/checker/tests_grounding_style.rs": {
        "inline_citation_only_rejects_prose_once_per_site",
        "inline_declaration_blocks_are_not_citation_style_sites",
        "inline_note_column_cap_counts_characters_not_bytes",
        "inline_note_column_cap_is_exact_in_non_ascii_prose",
        "inline_note_hard_caps_can_report_multiple_errors",
        "inline_note_line_cap_names_citations_in_source_order_deduplicated",
        "inline_note_soft_cap_is_warning_only_when_enabled",
        "inline_style_respects_disabled_python_docstring_scanning",
        "inline_style_strips_block_comment_continuation_prefix",
        "inline_style_strips_configured_comment_prefixes_for_note_detection",
        "python_docstring_citations_keep_source_columns",
        "require_grounding_flags_uncited_source_file",
        "require_grounding_off_by_default",
    },
    "crates/grund-core/src/config/tests_validation.rs": {
        "config_parses_project_description",
        "config_rejects_multiline_project_description",
        "inline_note_config_rejects_soft_cap_above_hard_cap",
    },
    "crates/grund-core/src/scanner/tests_qualified_citations.rs": {
        "marked_qualified_citation_is_recognised_unmarked_one_is_text",
        "non_strict_bare_token_with_slash_prefix_is_not_a_citation",
    },
    "crates/grund-core/src/workspace/tests_scope.rs": {
        "workspace_root_scope_requires_canonical_root_for_explicit_path",
    },
}

RUST_FUNCTION = re.compile(r"(?m)^fn\s+([a-z][a-z0-9_]*)\s*\(")


def _measurements(paths):
    command = [FISSILE, "measure", *paths, "--format", "json", "--no-color"]
    completed = subprocess.run(
        command,
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    if completed.returncode != 0:
        raise AssertionError(
            f"{' '.join(command)} exited {completed.returncode}:\n"
            f"{completed.stdout}{completed.stderr}"
        )
    try:
        return json.loads(completed.stdout)
    except json.JSONDecodeError as error:
        raise AssertionError(f"fissile returned malformed JSON: {error}") from error


class FileSizeOwnershipTests(unittest.TestCase):
    def test_approved_split_owners_fit_and_keep_the_test_inventory(self):
        problems = []
        existing = []
        for path, rule, _soft in OWNERSHIP:
            if (REPO_ROOT / path).is_file():
                existing.append(path)
            else:
                problems.append(f"missing: {path} (expected {rule} measurement)")

        rows = _measurements(existing)
        by_key = {
            (row.get("path"), row.get("rule_id")): row
            for row in rows
            if row.get("unit") == "lines"
        }
        for path, rule, expected_soft in OWNERSHIP:
            if path not in existing:
                continue
            row = by_key.get((path, rule))
            if row is None:
                problems.append(f"unmeasured: {path} (expected {rule} lines)")
                continue
            if row.get("soft") != expected_soft:
                problems.append(
                    f"budget changed: {path} has {rule} soft {row.get('soft')}, "
                    f"expected {expected_soft}"
                )
            actual = row.get("actual")
            if not isinstance(actual, int):
                problems.append(f"malformed measurement: {path} actual={actual!r}")
            elif actual > expected_soft:
                problems.append(f"over budget: {path} is {actual}/{expected_soft} lines")

        found_tests_by_path = {}
        for path in EXPECTED_TESTS_BY_PATH:
            source = REPO_ROOT / path
            if not source.is_file():
                continue
            found_names = set(RUST_FUNCTION.findall(source.read_text(encoding="utf-8")))
            found_tests_by_path[path] = found_names

        expected_inventory = set().union(*EXPECTED_TESTS_BY_PATH.values())
        found_inventory = set().union(*found_tests_by_path.values())
        for name in sorted(expected_inventory - found_inventory):
            problems.append(f"missing baseline test from approved owners: {name}")

        for path, expected_names in EXPECTED_TESTS_BY_PATH.items():
            if path not in found_tests_by_path:
                continue
            found_names = found_tests_by_path[path]
            for name in sorted(expected_names - found_names):
                problems.append(f"missing test: {path}::{name}")

        expected_count = sum(map(len, EXPECTED_TESTS_BY_PATH.values()))
        self.assertEqual(66, expected_count, "the pinned baseline inventory changed")
        self.assertEqual(66, len(expected_inventory), "a test name is assigned twice")
        self.assertEqual([], problems, "\n" + "\n".join(problems))


if __name__ == "__main__":
    unittest.main()
