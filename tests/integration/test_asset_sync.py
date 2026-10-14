"""§FS-init.5.3 — what the binary embeds is what the tree shows: the `grund-init`
skill under `skills/` is byte-identical to its copy under
`crates/grund-core/assets/`, in both directions, so neither can drift from the
other unnoticed. The scaffold templates have one copy only, the embedded one
(§FS-init.2.3.7), so there is no second tree to keep in step."""

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ASSETS = REPO_ROOT / "crates" / "grund-core" / "assets"


class AssetSyncTests(unittest.TestCase):
    def test_grund_init_skill_matches_embedded_asset(self):
        repo_skill = REPO_ROOT / "skills" / "grund-init" / "SKILL.md"
        embedded_skill = ASSETS / "skills" / "grund-init" / "SKILL.md"
        self.assertEqual(
            repo_skill.read_bytes(),
            embedded_skill.read_bytes(),
            "skills/grund-init/SKILL.md must stay byte-identical to the embedded asset",
        )

    def test_skill_citation_directions_match_canonical_page(self):
        canonical = REPO_ROOT / "docs" / "user-facing" / "citation-directions.md"
        skill = (REPO_ROOT / "skills" / "grund-init" / "SKILL.md").read_bytes()
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
