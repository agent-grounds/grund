"""§FS-distribution.4.2.2 — the release gate reads the release each message names
out of the tree's own message text and refuses a version that contradicts one.
The synthetic trees below pin the two directions and the closed clause
vocabulary; `ThisRepositoryTests` runs the gate against this repository, because
a gate wired to text nobody writes any more would pass everything in silence."""

import importlib.util
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SCRIPT = REPO_ROOT / "scripts" / "check_release_ramps.py"

_spec = importlib.util.spec_from_file_location("check_release_ramps", SCRIPT)
ramps = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(ramps)


def claims(text, path="crates/grund-core/src/x.rs"):
    return list(ramps.scan_text(path, text))


def refused(text, release):
    return ramps.report(claims(text), release)


LIB_RS = "crates/grund-core/src/lib.rs"


def apart_from_the_integrations_retirement(found):
    """§FS-distribution.3.1.3: the tree's claims less the `0.21.0` lines `lib.rs`
    writes for the integrations retirement, which `IntegrationsTransitionRampTests`
    judges on their own, so a window pinned for another ramp does not move with them."""
    return [claim for claim in found if not (claim.path == LIB_RS and claim.release == "0.21.0")]


class ClauseReadingTests(unittest.TestCase):
    def test_a_pending_clause_names_its_release(self):
        (claim,) = claims('"… becomes an error in grund 0.14.0"')
        self.assertEqual((claim.release, claim.direction), ("0.14.0", ramps.PENDING))

    def test_a_landed_clause_names_its_release(self):
        (claim,) = claims('"`prefix` was removed in grund 0.13.0 — rename it"')
        self.assertEqual((claim.release, claim.direction), ("0.13.0", ramps.LANDED))

    def test_a_scalar_exit_promise_names_its_release(self):
        (claim,) = claims(
            "warning: `grund refs` invalid IDs currently exit 2; "
            "they will exit 1 (failed query) in grund 0.15.0"
        )
        self.assertEqual((claim.release, claim.direction), ("0.15.0", ramps.PENDING))
        self.assertTrue(ramps.report([claim], "0.15.0"))

    def test_a_pending_removal_names_its_release(self):
        (claim,) = claims('note = "is removed in 0.15.0; use the `grund` CLI package"')
        self.assertEqual((claim.release, claim.direction), ("0.15.0", ramps.PENDING))

    def test_a_removals_two_tenses_read_as_opposite_directions(self):
        """The same removal, promised and made: one clause each, so a deprecation
        that names its release is refused above it and the message that replaces
        it is refused below it. §FS-distribution.4.2.3 is why the pending half is
        spelled `is removed in` rather than "will be removed in" — the two tenses
        of one removal read as the same claim."""
        found = claims("is removed in 0.15.0\nwas removed in 0.13.0")
        self.assertEqual(
            [(c.direction, c.release) for c in found],
            [(ramps.PENDING, "0.15.0"), (ramps.LANDED, "0.13.0")],
        )

    def test_grund_is_optional_and_bold_is_tolerated(self):
        for text in ("stopped loading in 0.13.0", "stopped loading in grund **0.13.0**"):
            (claim,) = claims(text)
            self.assertEqual(claim.release, "0.13.0", text)

    def test_the_clause_vocabulary_is_closed(self):
        """§FS-distribution.4.2.3 — the vocabulary is closed on purpose: a ramp
        written outside the wording the warnings already use names no release the
        guard can read, so "will be removed in" is not a clause."""
        self.assertEqual(claims("this will be removed in 0.15.0 one day"), [])
        self.assertEqual(claims("the wording changed in 0.15.0"), [])

    def test_a_pending_wording_change_names_its_release(self):
        """§FS-distribution.4.2.2 — the inverted form of the assertion that used
        to pin this clause as unimplemented. A message promising that its own
        wording changes at a named release makes a pending claim about that
        release, in both spellings the other clauses accept."""
        self.assertIn(("wording changes in", ramps.PENDING), ramps.CLAUSES)
        for text in ("wording changes in 0.15.0", "wording changes in grund 0.15.0"):
            (claim,) = claims(text)
            self.assertEqual((claim.release, claim.direction), ("0.15.0", ramps.PENDING), text)

    def test_a_release_is_never_read_across_a_line_break(self):
        self.assertEqual(claims("… becomes an error in\ngrund 0.14.0."), [])

    def test_every_line_of_a_golden_is_read(self):
        found = claims("was removed in 0.13.0\nx\nbecomes an error in 0.14.0")
        self.assertEqual([(c.line, c.release) for c in found], [(1, "0.13.0"), (3, "0.14.0")])


    def test_an_attribution_pair_yields_both_of_its_claims(self):
        """§FS-distribution.4.2.3.1 — a verdict that moved names both of its
        releases on one line, so the line makes two claims and the bare
        `an error in <release>` is the landed one. These are the bytes two
        shipped rules print: the `local-section-citation` attribution
        (§FS-check.3.24.1) and the index rule's."""
        for text, prior, landed in (
            (
                "docs/FS-a.md:3: error: local section citation \u00a72; write \u00a7FS-a.2 "
                "— unchecked in grund 0.13.1, an error in 0.14.0; run `grund fmt --write`",
                "0.13.1",
                "0.14.0",
            ),
            (
                "docs/fs/README.md:3: error: index entry \u00a7FS-001-login is not a link; "
                "unchecked in grund 0.11.0, an error in 0.12.0 — run `grund fmt --write`",
                "0.11.0",
                "0.12.0",
            ),
        ):
            with self.subTest(landed=landed):
                found = claims(text)
                self.assertEqual(
                    [(c.clause, c.release, c.direction) for c in found],
                    [
                        ("unchecked in", prior, ramps.LANDED),
                        ("an error in", landed, ramps.LANDED),
                    ],
                )

    def test_a_verb_in_front_of_the_phrase_keeps_the_tense_it_spells(self):
        """§FS-distribution.4.2.3.1 — the bare clause is a substring of the two
        named-error tenses, so it is read only where it opens a clause. A verb
        in front of it must still yield exactly one claim in the direction that
        verb spells, and a tense the vocabulary does not carry must not be read
        as landed on the strength of the phrase it contains."""
        for text, direction in (
            ("… becomes an error in grund 0.14.0", ramps.PENDING),
            ("… became an error in grund 0.14.0", ramps.LANDED),
        ):
            with self.subTest(text=text):
                (claim,) = claims(text)
                self.assertEqual((claim.release, claim.direction), ("0.14.0", direction))
        for text in (
            "the absorbed-scan warning would become an error in 0.15.0. Land it first",
            "the unmarked-heading warning will become an error in grund 0.16.0",
        ):
            with self.subTest(text=text):
                landed = [c for c in claims(text) if c.direction == ramps.LANDED]
                self.assertEqual(landed, [], "a promise not yet made was read as landed")


class VerdictTests(unittest.TestCase):
    LANDED = '"`prefix` was removed in grund 0.13.0"'
    PENDING = '"an index entry becomes an error in grund 0.13.0"'
    REMOVAL = '"is removed in 0.13.0; use the `grund` CLI package"'
    WORDING = '"… the {scope} subtree; this wording changes in grund 0.14.0"'

    def test_a_landed_change_may_not_ship_below_the_release_it_names(self):
        report = refused(self.LANDED, "0.12.4")
        self.assertTrue(report)
        self.assertIn("cannot be released as 0.12.4", report[0])
        self.assertTrue(any("was removed in 0.13.0" in line for line in report))

    def test_a_landed_change_ships_at_the_release_it_names(self):
        self.assertEqual(refused(self.LANDED, "0.13.0"), [])

    def test_a_landed_change_ships_above_the_release_it_names(self):
        self.assertEqual(refused(self.LANDED, "0.14.0"), [])

    def test_a_pending_promise_may_not_ship_at_the_release_it_names(self):
        report = refused(self.PENDING, "0.13.0")
        self.assertTrue(any("becomes an error in 0.13.0" in line for line in report))

    def test_a_pending_promise_ships_below_the_release_it_names(self):
        self.assertEqual(refused(self.PENDING, "0.12.4"), [])

    def test_a_named_removal_may_not_ship_at_the_release_it_names(self):
        report = refused(self.REMOVAL, "0.13.0")
        self.assertTrue(any("is removed in 0.13.0" in line for line in report))

    def test_a_named_removal_ships_below_the_release_it_names(self):
        self.assertEqual(refused(self.REMOVAL, "0.12.4"), [])

    def test_a_promised_wording_change_ships_below_the_release_it_names(self):
        """§FS-distribution.4.2.2 — the clause's own three cases, on a fixture
        that names nothing else, so no other ramp in the tree can supply or
        mask the verdict. Below its deadline the promise is still ahead of the
        tree, so the release is allowed."""
        self.assertEqual(refused(self.WORDING, "0.13.2"), [])

    def test_a_promised_wording_change_may_not_ship_at_the_release_it_names(self):
        report = refused(self.WORDING, "0.14.0")
        self.assertTrue(report)
        self.assertIn("cannot be released as 0.14.0", report[0])
        self.assertTrue(any("wording changes in 0.14.0" in line for line in report))

    def test_a_promised_wording_change_may_not_ship_above_the_release_it_names(self):
        report = refused(self.WORDING, "0.15.0")
        self.assertTrue(report)
        self.assertIn("cannot be released as 0.15.0", report[0])
        self.assertTrue(any("wording changes in 0.14.0" in line for line in report))

    def test_a_tree_that_landed_and_still_promises_one_release_can_cut_nothing(self):
        """§FS-distribution.4.2.5 — the window of releases left can be empty, and
        an empty window is itself the answer: a tree that landed one ramp and
        still promises another at the same release may be published as nothing."""
        report = refused(f"{self.LANDED}\n{self.PENDING}", "0.12.4")
        self.assertTrue(any("no release at all" in line for line in report))

    def test_the_window_is_the_highest_landed_and_the_lowest_promised(self):
        """§FS-distribution.4.2.5 — the window the refusal names is bounded by the
        highest release a landed message claims and the lowest one a pending
        message promises."""
        found = claims('"was removed in 0.11.0"\n"was removed in 0.13.0"\n"becomes an error in 0.14.0"')
        self.assertEqual(ramps.release_window(found), ("0.13.0", "0.14.0"))

    def test_a_tree_naming_no_release_refuses_nothing(self):
        self.assertEqual(refused('"ordinary message"', "0.12.4"), [])

    def test_a_version_that_is_not_a_release_is_a_usage_error(self):
        self.assertEqual(ramps.main(["0.12.4-dev"]), 2)


    PAIR = (
        '"docs/FS-a.md:3: error: local section citation \u00a72; write \u00a7FS-a.2 '
        '— unchecked in grund 0.13.1, an error in 0.14.0"'
    )
    INDEX_PAIR = (
        '"docs/fs/README.md:3: error: index entry \u00a7FS-001-login is not a link; '
        'unchecked in grund 0.11.0, an error in 0.12.0 — run `grund fmt --write`"'
    )

    def test_an_attribution_pair_may_not_ship_below_the_release_it_landed_in(self):
        """§FS-distribution.4.2.3.1 — the floor is the later half of the pair.
        A tree carrying this line says the verdict moved in 0.14.0, so cutting
        0.13.5 would ship a binary whose own diagnostics deny its version."""
        report = refused(self.PAIR, "0.13.5")
        self.assertTrue(report, "the `an error in 0.14.0` half set no floor")
        self.assertIn("cannot be released as 0.13.5", report[0])
        self.assertTrue(any("an error in 0.14.0" in line for line in report))

    def test_the_index_rules_attribution_pair_sets_the_floor_too(self):
        """§FS-distribution.4.2.3.1 — not one rule's wording: the index rule
        prints the same shape, and has since 0.12.0."""
        report = refused(self.INDEX_PAIR, "0.11.1")
        self.assertTrue(report, "the `an error in 0.12.0` half set no floor")
        self.assertTrue(any("an error in 0.12.0" in line for line in report))

    def test_the_same_claim_in_the_landed_tense_is_refused_as_well(self):
        """The control: written `became an error in 0.14.0`, the identical claim
        is refused by a clause the vocabulary already carried. That is what
        makes the two tests above a gap in the vocabulary rather than a fixture
        that never reached the guard."""
        report = refused('"… became an error in 0.14.0"', "0.13.5")
        self.assertTrue(report)
        self.assertIn("cannot be released as 0.13.5", report[0])

    def test_an_attribution_pair_ships_at_the_release_it_landed_in(self):
        """§FS-distribution.4.2.2 — the later half is a floor, not a ban: at it
        and above it the version and the message agree."""
        self.assertEqual(refused(self.PAIR, "0.14.0"), [])
        self.assertEqual(refused(self.PAIR, "0.15.0"), [])


class ThisRepositoryTests(unittest.TestCase):
    """The gate is only worth its step if it still sees this tree's own ramps."""

    @classmethod
    def setUpClass(cls):
        cls.claims = ramps.scan_tree(REPO_ROOT)

    def test_the_scan_reaches_both_a_rust_source_and_an_e2e_golden(self):
        homes = {claim.path.split("/")[0] for claim in self.claims}
        self.assertIn("crates", homes)
        self.assertIn("tests", homes)

    def test_the_fulfilled_main_entry_removal_is_no_longer_pending(self):
        """§FS-distribution.3.1.1: 0.14.0 shipped the removal notice and 0.15.0
        removes the symbol, so this tree no longer carries that pending claim.
        Other 0.15.0 ramps remain ordinary input to the generic release gate."""
        pending = [
            claim
            for claim in self.claims
            if claim.path == "crates/grund-core/src/compat/cli.rs"
            and claim.clause == "is removed in"
            and claim.release == "0.15.0"
            and claim.direction == ramps.PENDING
        ]
        self.assertEqual([], pending, "main_entry() is still pending removal in 0.15.0")

    def test_no_golden_still_promises_a_wording_change_in_0_16_0(self):
        """§FS-distribution.4.2.2 — the wording ramps this tree carried all named
        0.16.0, and their goldens hold the final wording (§FS-errors.3.3,
        §FS-errors.3.6.1, §FS-errors.3.7.1). A golden that still promised a
        wording change would ship a promise the release has already reached.
        The one-grammar window names 0.19.0 instead (§FS-errors.3.8)."""
        golden = [
            claim
            for claim in self.claims
            if claim.clause == "wording changes in"
            and claim.release == "0.16.0"
            and claim.path.startswith("tests/")
        ]
        self.assertEqual([], golden)

    def test_no_line_still_promises_a_change_in_0_20_0(self):
        """§FS-distribution.4.2.2 — the two ramps 0.19.0 announced for 0.20.0 have
        landed: `grund agent-setup-instructions` is gone (§FS-init.5.7) and a
        citation-only image is no index entry (§FS-citations.checks.image-index-entry).
        A source line or golden that still promised a change at 0.20.0 would make the
        release guard refuse 0.20.0."""
        pending = [
            claim.quoted()
            for claim in self.claims
            if claim.direction == ramps.PENDING and claim.release == "0.20.0"
        ]
        self.assertEqual([], pending)

    def test_the_landed_stub_target_flip_holds_the_floor_at_0_19_0(self):
        """§FS-distribution.4.2.2 — excluded and ignored stub targets became
        errors in 0.19.0 (§FS-declarations.stubs.target.3), raising the message
        floor independently of the shared floor. Historical landed clauses
        still refuse lower releases: the bare-`grund` removal (§FS-cli.4) and
        unreached-declaration flip (§FS-rules.checks.unreached-declaration)
        refuse 0.15.x, the unlisted-`[workspace]` flip (§FS-check.3.29.14)
        refuses 0.14.x, and the `prefix` removal refuses 0.12.x. The integrations
        retirement's lines raise the tree's floor further, and are judged apart."""
        floor, _ = ramps.release_window(apart_from_the_integrations_retirement(self.claims))
        self.assertEqual(floor, "0.19.0")
        report = ramps.report(self.claims, "0.15.0")
        self.assertTrue(
            any(
                "cli-no-args/expected.stderr" in line
                and "was removed in 0.16.0" in line
                for line in report
            ),
            "the bare-`grund` removal must refuse a 0.15.x release",
        )
        self.assertTrue(
            any("became an error in 0.16.0" in line for line in report),
            "the unreached-declaration flip must refuse a 0.15.x release",
        )
        report = ramps.report(self.claims, "0.14.3")
        self.assertTrue(
            any("became an error in 0.15.0" in line for line in report),
            "the unlisted-[workspace] flip must be what refuses a 0.14.x release",
        )
        self.assertTrue(
            any("became an error in 0.16.0" in line for line in report),
            "the unreached-declaration flip must refuse a 0.14.x release too",
        )
        report = ramps.report(self.claims, "0.12.4")
        self.assertTrue(
            any("config/v1/kind_table.rs" in line for line in report),
            "the `prefix` removal is still read as a landed clause of this tree",
        )

    def test_the_stub_target_message_alone_refuses_0_18_1_in_source_and_goldens(self):
        """§FS-distribution.4.2.2 — the active diagnostic and its exclude/ignore
        goldens name the landed 0.19.0 change (§FS-declarations.stubs.target.3).
        Read those actual bytes and attribute the 0.18.1 refusal to each home,
        without supplying the independent shared floor."""
        report = ramps.report(self.claims, "0.18.1")
        self.assertEqual(report[0], "error: this tree cannot be released as 0.18.1")
        for path in (
            "crates/grund-core/src/checker/conform.rs",
            "tests/e2e/cases/check-excluded-stub-target-exclude-entry/expected.stdout",
            "tests/e2e/cases/check-excluded-stub-target-ignore-file/expected.stdout",
        ):
            with self.subTest(path=path):
                self.assertTrue(any(
                    claim.path == path and claim.clause == "became an error in"
                    and claim.release == "0.19.0" and claim.direction == ramps.LANDED
                    for claim in self.claims
                ), f"{path} must carry the landed stub-target clause")
                self.assertTrue(any(
                    path + ":" in line and "`became an error in 0.19.0`" in line
                    for line in report
                ), f"{path} must explicitly refuse a 0.18.1 release")

    def test_the_attribution_pairs_this_tree_ships_are_read(self):
        """§FS-distribution.4.2.3.1 — two shipped rules print the pair, and two
        checked goldens pin those bytes, so the guard must see the landed half
        in both homes it scans. A clause wired to text nobody writes passes in
        silence."""
        bare = [claim for claim in self.claims if claim.clause == "an error in"]
        self.assertTrue(bare, "this tree carries no bare `an error in` claim to read")
        self.assertEqual({claim.direction for claim in bare}, {ramps.LANDED})
        homes = {claim.path.split("/")[0] for claim in bare}
        self.assertIn("crates", homes)
        self.assertIn("tests", homes)



class IntegrationsTransitionRampTests(unittest.TestCase):
    """§FS-distribution.3.1.2 and §FS-distribution.3.1.3 — the lines the
    integrations retirement writes, read by the guard and judged on their own:
    the whole tree's verdict belongs to every other ramp it carries."""

    LIB_RS = LIB_RS
    ALIASES = ("Findings", "Config", "KindConfig", "CitationRules")
    RETIRED = 30 + 9  # the root names and the methods noticed in 0.19.0, removed in 0.21.0

    @classmethod
    def setUpClass(cls):
        cls.claims = ramps.scan_tree(REPO_ROOT)
        cls.lib_lines = (REPO_ROOT / cls.LIB_RS).read_text(encoding="utf-8").splitlines()

    def _claims(self, clause, release, direction, path_prefix):
        return [
            claim
            for claim in self.claims
            if claim.clause == clause and claim.release == release
            and claim.direction == direction and claim.path.startswith(path_prefix)
        ]

    def _refuses_0_20_and_allows_0_21(self, found):
        for cut in ("0.20.0", "0.20.1"):
            self.assertTrue(ramps.report(found, cut), f"the lines must refuse a {cut} cut")
        self.assertEqual([], ramps.report(found, "0.21.0"), "the lines allow 0.21.0")

    def test_each_retired_name_and_method_leaves_one_line_read_as_landed_in_0_21_0(self):
        removed = self._claims("was removed in", "0.21.0", ramps.LANDED, self.LIB_RS)
        self.assertEqual(self.RETIRED, len(removed), "one `was removed in 0.21.0` line per name and method")
        named = {line.split("`")[1] for line in (self.lib_lines[c.line - 1] for c in removed) if "`" in line}
        self.assertEqual(self.RETIRED, len(named), "each line names a different retired name or method")
        self._refuses_0_20_and_allows_0_21(removed)

    def test_no_integrations_notice_is_still_pending(self):
        pending = self._claims("is removed in", "0.21.0", ramps.PENDING, "crates/grund-core/src/")
        self.assertEqual([], [claim.quoted() for claim in pending])

    def test_each_removed_alias_leaves_one_line_read_as_landed_in_0_19_0(self):
        removed = self._claims("was removed in", "0.19.0", ramps.LANDED, self.LIB_RS)
        named = sorted(
            alias
            for alias in self.ALIASES
            for claim in removed
            if f"`{alias}`" in self.lib_lines[claim.line - 1]
        )
        self.assertEqual(sorted(self.ALIASES), named, "one `was removed in 0.19.0` line per alias")
        self.assertTrue(ramps.report(removed, "0.18.1"), "the removal lines refuse a 0.18.x patch")
        self.assertEqual([], ramps.report(removed, "0.19.0"), "the removal lines allow 0.19.0")

    def test_the_crate_level_literal_notice_is_read_as_landed_in_0_21_0(self):
        def crate_level(found):
            return [c for c in found if self.lib_lines[c.line - 1].lstrip().startswith("//!")]

        pending = crate_level(self._claims("becomes an error in", "0.21.0", ramps.PENDING, self.LIB_RS))
        self.assertEqual([], [claim.quoted() for claim in pending], "the crate notice still pending")
        landed = crate_level(self._claims("became an error in", "0.21.0", ramps.LANDED, self.LIB_RS))
        self.assertEqual(1, len(landed), "one crate-level `became an error in 0.21.0` line")
        self._refuses_0_20_and_allows_0_21(landed)


if __name__ == "__main__":
    unittest.main()
