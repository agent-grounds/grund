"""§FS-distribution.4.6.4.2 — contributor-visible correction audit guidance.

Run the actual repository walk (§AR-goal-measurement.3), capturing the default
unittest rendering rather than substituting the helper's unabridged strings.
"""

import io
import re
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from tests.integration import test_backwards_compatibility_routes as audit


REJECTION = """# DF-reject-route: Reject the correction route

## status: Accepted

## context: Compatibility

The route in §REQ-backwards-compatibility.5 is not used; this decision uses the
pre-1.0 allowance in §REQ-backwards-compatibility.4.
"""
NUMBERED_REJECTION = REJECTION.replace("## context: Compatibility", "## 1. Context")
PROOF_REASONS = (
    "the route must cite a numbered section of a different hard requirement",
    "the route must prove that the prior verdict violated the citation",
    "the route must prove that the prohibition already applied",
    "the route must explain why \u00a72 deprecation does not fit",
    "the route must explain why \u00a73 mechanical migration does not fit",
    "the route must deny a broader compatibility licence",
    "the decision must have a matching release record",
)


class CorrectionRouteGuidanceTests(unittest.TestCase):
    def _walk(self, text, expected_audits=1):
        """The pre-commit meter over one scratch decision, with its real proofs.

        Keep TestCase.maxDiff at its normal value: guidance that only exists in
        a shortened assertion value is not guidance the contributor can read.
        """
        with tempfile.TemporaryDirectory(prefix="grund-846-walk-") as scratch:
            root = Path(scratch)
            decisions = root / "docs" / "decisions" / "functional"
            decisions.mkdir(parents=True)
            (decisions / "DF-reject-route.md").write_text(text, encoding="utf-8")
            stream = io.StringIO()
            case = audit.DecisionRouteTests(
                "test_every_correction_decision_proves_all_five_gates"
            )
            with (
                patch.object(audit, "DECISIONS", root / "docs" / "decisions"),
                patch.object(audit, "REPO_ROOT", root),
                patch.object(
                    audit, "_correction_route_errors", wraps=audit._correction_route_errors
                ) as proof,
            ):
                result = unittest.TextTestRunner(stream=stream).run(case)
            output = stream.getvalue()
            self.assertEqual(expected_audits, proof.call_count, output)
        self.assertFalse(result.errors, output)
        return result, output

    def _refusal(self, text):
        result, output = self._walk(text)
        self.assertEqual(1, len(result.failures), output)
        self.assertFalse(result.wasSuccessful(), output)
        # Only the failure's rendered message counts, not names in its traceback.
        message = output[output.index("AssertionError:"):]
        return message

    def _assert_guidance(self, message):
        normalized = re.sub(r"\s+", " ", message).lower()
        checks = {
            "the live section-5 citation": audit.CORRECTION_ROUTE in message,
            "a live citation": "live citation" in normalized,
            "citation as the audit trigger": bool(re.search(
                r"citation.{0,180}(?:audit|select|trigger)|"
                r"(?:audit|select|trigger).{0,180}citation", normalized
            )),
            "the escaped mention": "<§>REQ-backwards-compatibility.5" in message,
            "the parent citation": bool(re.search(
                r"§REQ-backwards-compatibility(?![\w.-])", message
            )),
            "section 5 in prose": bool(re.search(
                r"section\s+5.{0,100}prose|prose.{0,100}section\s+5", normalized
            )),
            "discussion without invoking the route": bool(re.search(
                r"(?:discuss|reject|mention).{0,160}without.{0,40}(?:invok|cit)",
                normalized,
            )),
        }
        missing = [label for label, present in checks.items() if not present]
        self.assertEqual([], missing, "Missing guidance: " + ", ".join(missing)
                         + "\nActual contributor-visible failure:\n" + message)

    def test_named_section_rejection_explains_trigger_and_both_mentions(self):
        message = self._refusal(REJECTION)
        self.assertIn("the correction route must be invoked in one numbered section", message)
        self._assert_guidance(message)

    def test_numbered_section_rejection_keeps_guidance_with_proof_failures(self):
        message = self._refusal(NUMBERED_REJECTION)
        errors = audit._correction_route_errors(
            NUMBERED_REJECTION, [], audit._requirement_catalog()
        )
        for reason in PROOF_REASONS:
            with self.subTest(reason=reason):
                self.assertIn(reason, errors)
                self.assertIn(reason, message)
        self._assert_guidance(message)

    def test_escaped_mention_is_outside_the_audit(self):
        escaped = REJECTION.replace(audit.CORRECTION_ROUTE,
                                    "<§>REQ-backwards-compatibility.5")
        result, output = self._walk(escaped, expected_audits=0)
        self.assertTrue(result.wasSuccessful(), output)

    def test_parent_citation_with_section_in_prose_is_outside_the_audit(self):
        prose = REJECTION.replace(audit.CORRECTION_ROUTE,
                                  "§REQ-backwards-compatibility (section 5)")
        result, output = self._walk(prose, expected_audits=0)
        self.assertTrue(result.wasSuccessful(), output)

    def test_unproved_genuine_correction_still_fails_the_walk(self):
        genuine = NUMBERED_REJECTION.replace(
            "The route in §REQ-backwards-compatibility.5 is not used; this decision uses the\n"
            "pre-1.0 allowance in §REQ-backwards-compatibility.4.",
            "§REQ-backwards-compatibility.5 is invoked.",
        )
        self.assertNotEqual(NUMBERED_REJECTION, genuine)
        self._refusal(genuine)
        errors = audit._correction_route_errors(genuine, [], audit._requirement_catalog())
        for reason in PROOF_REASONS:
            with self.subTest(reason=reason):
                self.assertIn(reason, errors)

    def test_valid_correction_still_passes_the_walk(self):
        valid = audit.COVER_DECISION.read_text(encoding="utf-8")
        result, output = self._walk(valid)
        self.assertTrue(result.wasSuccessful(), output)


if __name__ == "__main__":
    unittest.main()
