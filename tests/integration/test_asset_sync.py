"""§FS-init.5.3 — what the binary embeds is what the tree shows: the `grund-init`
skill under `skills/`, its core and every reference file, is byte-identical to
its copy under `crates/grund-core/assets/`, file for file in both directions, so
neither can drift from the other unnoticed. The scaffold templates have one copy
only, the embedded one (§FS-init.2.3.7), so there is no second tree to keep in
step."""

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ASSETS = REPO_ROOT / "crates" / "grund-core" / "assets"


class AssetSyncTests(unittest.TestCase):
    def test_grund_init_skill_matches_embedded_asset(self):
        repo_skill = REPO_ROOT / "skills" / "grund-init"
        embedded_skill = ASSETS / "skills" / "grund-init"

        def files(root):
            return {
                path.relative_to(root).as_posix(): path.read_bytes()
                for path in sorted(root.rglob("*"))
                if path.is_file()
            }

        repo_files = files(repo_skill)
        embedded_files = files(embedded_skill)
        self.assertIn("SKILL.md", repo_files)
        self.assertEqual(
            sorted(repo_files),
            sorted(embedded_files),
            "skills/grund-init/ and the embedded asset must hold the same files",
        )
        for name, data in repo_files.items():
            self.assertEqual(
                data,
                embedded_files[name],
                f"skills/grund-init/{name} must stay byte-identical to the embedded asset",
            )

    def test_skill_citation_directions_match_canonical_page(self):
        canonical = REPO_ROOT / "docs" / "user-facing" / "citation-directions.md"
        # §FS-init.5.5: the page sits in the skill's `citations` reference section.
        skill = (REPO_ROOT / "skills" / "grund-init" / "references" / "citations.md").read_bytes()
        begin = b"<!-- BEGIN citation-directions -->\n"
        end = b"<!-- END citation-directions -->"
        self.assertEqual(skill.count(begin), 1)
        self.assertEqual(skill.count(end), 1)
        start = skill.index(begin) + len(begin)
        finish = skill.index(end, start)
        self.assertEqual(
            skill[start:finish],
            canonical.read_bytes(),
            "the marked skill copy must stay byte-identical to the canonical citation-directions page",
        )

    def test_the_templates_have_one_copy_the_embedded_one(self):
        # §FS-init.2.3.7: the source tree's one copy is the asset `grund init` embeds.
        self.assertFalse((REPO_ROOT / "templates").exists(),
                         "templates/ must not exist: crates/grund-core/assets/templates/ is the one copy")
        embedded_templates = ASSETS / "templates"
        self.assertTrue((embedded_templates / "AGENTS.md").is_file())
        self.assertTrue((embedded_templates / "software-development" / "grund.toml").is_file())
        self.assertTrue((embedded_templates / "none" / "grund.toml").is_file())


if __name__ == "__main__":
    unittest.main()
