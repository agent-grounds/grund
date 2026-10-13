"""Fixture discovery and report bases (§FS-config.1, §FS-config.3.6.1,
§FS-workspace.5.1, §AR-config.1.5, §REQ-one-answer.proof.matrix).
"""

import importlib.util
from pathlib import Path
import unittest

SOURCE = Path(__file__).resolve().parents[1] / 'bindings/agreement_scope.py'
SPEC = importlib.util.spec_from_file_location('agreement_scope', SOURCE)
SCOPE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(SCOPE)


class AgreementScopeTests(unittest.TestCase):
    def setUp(self):
        self.temporary = SCOPE.isolated_fixtures()
        self.addCleanup(self.temporary.cleanup)
        self.home = Path(self.temporary.name)
        self.repo = self.home / 'repo'
        self.repo.mkdir()

    def test_nested_config_report_base_and_invocation_base(self):
        (self.repo / 'grund.toml').write_text('[workspace]\nmembers = ["docs"]\n')
        member = self.repo / 'docs'
        (member / 'src').mkdir(parents=True)
        (member / 'grund.toml').write_text('[scan]\ninclude = ["src"]\n')
        for surface in ('cli', 'python', 'node', 'editor'):
            answer = SCOPE.scope(self.repo, member / 'src', member / 'src', False, surface)
            self.assertEqual(str(member), answer['projectRoot'])
            self.assertEqual(str(member), answer['reportBase'])
            self.assertEqual(str(member / 'src'), answer['invocationBase'])
        (member / 'grund.toml').write_text('[output]\nrelative_paths = false\n')
        answer = SCOPE.scope(self.repo, member / 'src', member / 'src', False, 'cli')
        self.assertEqual(str(member / 'src'), answer['reportBase'])

    def test_member_without_config_owns_its_invocation(self):
        (self.repo / 'grund.toml').write_text('[workspace]\nmembers = ["apps/*"]\n')
        member = self.repo / 'apps/api'
        (member / 'src').mkdir(parents=True)
        answer = SCOPE.scope(self.repo, self.repo, member / 'src', True, 'node')
        self.assertEqual(str(member), answer['projectRoot'])
        self.assertIsNone(answer['configFile'])

    def test_no_config_and_ancestor_leak(self):
        child = self.repo / 'external'
        child.mkdir()
        for surface in ('cli', 'python', 'node', 'editor'):
            answer = SCOPE.scope(self.repo, child, child, False, surface)
            self.assertEqual(str(child), answer['projectRoot'])
            self.assertIsNone(answer['configFile'])
        (self.home / 'grund.toml').write_text('project_name = "outside"\n')
        with self.assertRaisesRegex(AssertionError, 'outside the copy'):
            SCOPE.scope(self.repo, child, child, False, 'cli')

    def test_deprecated_location_and_malformed_input(self):
        (self.repo / '.agents').mkdir()
        config = self.repo / '.agents/grund.toml'
        config.write_text('[output]\nrelative_paths = false\n')
        answer = SCOPE.scope(self.repo, self.repo, self.repo, True, 'editor')
        self.assertEqual(str(config), answer['configFile'])
        self.assertEqual(str(self.repo), answer['projectRoot'])
        config.write_text('this is not TOML')
        answer = SCOPE.scope(self.repo, self.repo, self.repo, True, 'editor')
        self.assertIsNotNone(answer['parseError'])
        self.assertEqual(str(config), answer['configFile'])


if __name__ == '__main__':
    unittest.main()
