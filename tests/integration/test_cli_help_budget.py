"""§FS-cli.2.2 — the top-level help page fits the one screen its own
specification sets at ≤ 24 lines, measured on the golden that records what the
binary prints (`tests/e2e/cases/cli-help/expected.stdout`). The width bound is
the same one-screen requirement read across the other axis: a 150-column row
does not fit a screen either (§GOAL-friendliness-first.1). The page is rendered
by `print_help` in the CLI frontend crate, so the golden and the code move
together (§AR-bindings.3).

Measured on the golden rather than by running the binary: the golden is the
contract the e2e harness already compares byte-for-byte, so a page that grows
cannot satisfy one meter and miss the other.
"""

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]

# §FS-cli.2.2: "The whole page fits one screen … which this spec sets at ≤ 24
# lines".
MAX_LINES = 24

# One screen is a width as well as a height; 100 columns is the bound this
# repository holds its own text to.
MAX_COLUMNS = 100

HELP_GOLDEN = REPO_ROOT / "tests/e2e/cases/cli-help/expected.stdout"


class TopLevelHelpFitsOneScreen(unittest.TestCase):
    def setUp(self):
        self.lines = HELP_GOLDEN.read_text(encoding="utf-8").splitlines()

    def test_the_page_is_at_most_twenty_four_lines(self):
        self.assertLessEqual(
            len(self.lines),
            MAX_LINES,
            f"{HELP_GOLDEN.relative_to(REPO_ROOT)} is {len(self.lines)} lines; "
            f"§FS-cli.2.2 sets the budget at {MAX_LINES}",
        )

    def test_no_row_runs_past_the_screen(self):
        wide = [
            f"{number}: {len(line)} columns"
            for number, line in enumerate(self.lines, start=1)
            if len(line) > MAX_COLUMNS
        ]
        self.assertEqual(
            [],
            wide,
            f"{HELP_GOLDEN.relative_to(REPO_ROOT)} has rows past "
            f"{MAX_COLUMNS} columns: {'; '.join(wide)}",
        )


if __name__ == "__main__":
    unittest.main()
