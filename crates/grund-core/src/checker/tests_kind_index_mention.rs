//! Test module: what "appears" means when the index mentions a covered ID but
//! holds no entry for it (§FS-check.3.18.5.1). Every case here is a boundary of
//! that look rather than of the rule it words: which occurrences are a mention,
//! which are not, and the two neighbouring findings it must leave alone —
//! §FS-check.3.18.7's missing index file and §FS-check.3.17's repairable bare
//! entry.

use crate::testing::{
    check_run, codes, findings, kind_index_repo, kind_index_repo_loose, only, test_root, write,
};

/// The clause §FS-check.3.18.5.1 puts on the finding where the index mentions
/// the ID. The cases below assert the clause rather than the whole line, so each
/// one says which of the two forms it expects without restating the path and the
/// ramp clause `scripts/check_release_ramps.py` reads (§FS-distribution.4.2).
const MENTION_CLAUSE: &str =
    "but not as an entry: an entry is a `§`-marked Markdown link to the declaration";

/// The same clause off strict mode, where a bare token is a citation
/// (§FS-config.3.1) and a hand-written link around one is already a correct entry
/// (§FS-check.3.17.5) — so the finding may not name the marked form as *the* one.
const LOOSE_MENTION_CLAUSE: &str =
    "but not as an entry: an entry is a Markdown link to the declaration whose text is the ID";

/// §FS-check.3.18.5.1: the file-name half of the look. A declaration whose file
/// is not named after its ID is mentioned by that file name, which is the half
/// an ID-token search cannot reach — the row in `grund#297` is this shape.
#[test]
fn an_index_naming_the_declaration_file_mentions_the_id() {
    let root = test_root("an_index_naming_the_declaration_file_mentions_the_id");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n\n\
         [[kinds]]\nkind = \"AR\"\nfolder = \"docs/architecture\"\n\n\
         [scan]\ninclude = [\"docs\"]\n",
    );
    write(
        &root.join("docs/architecture/overview.md"),
        "# AR-001-thing: how the thing is built\n\nThe thing is built out of parts.\n",
    );
    write(
        &root.join("docs/architecture/README.md"),
        "# Architecture\n\nThe design lives in `overview.md`.\n",
    );

    let run = check_run(&root, false);
    let finding = only(&run, "missing-index-entry");
    assert!(
        finding.message.contains(MENTION_CLAUSE),
        "§FS-check.3.18.5.1: the file name is a mention, so the finding says the \
         ID appears rather than that it is not listed: {}",
        finding.message
    );
}

/// §FS-check.3.18.5.1: the file name is matched as a *whole* name. A longer stem
/// and a longer extension are two different files, and a page naming either of
/// them has said nothing about this one.
#[test]
fn a_longer_file_name_is_not_a_mention_of_this_one() {
    let root = test_root("a_longer_file_name_is_not_a_mention_of_this_one");
    write(
        &root.join("grund.toml"),
        "grund_config_version = 1\n\n\
         [[kinds]]\nkind = \"AR\"\nfolder = \"docs/architecture\"\n\n\
         [scan]\ninclude = [\"docs\"]\n",
    );
    write(
        &root.join("docs/architecture/overview.md"),
        "# AR-001-thing: how the thing is built\n\nThe thing is built out of parts.\n",
    );
    write(
        &root.join("docs/architecture/README.md"),
        "# Architecture\n\nSee `my-overview.md`, and `overview.markdown` for the old draft.\n",
    );

    let run = check_run(&root, false);
    let finding = only(&run, "missing-index-entry");
    assert!(
        finding.message.contains("is not listed in") && !finding.message.contains(MENTION_CLAUSE),
        "§FS-check.3.18.5.1: neither `my-overview.md` nor `overview.markdown` is \
         `overview.md`, so the page mentions nothing: {}",
        finding.message
    );
}

/// §FS-check.3.18.5.1: an occurrence is an ID-shaped token on its own
/// boundaries, so a longer ID that contains this one is not a mention of it.
#[test]
fn a_longer_id_containing_this_one_is_not_a_mention() {
    let root = kind_index_repo("a_longer_id_containing_this_one_is_not_a_mention");
    write(
        &root.join("docs/specs/README.md"),
        "# Specs\n\nA longer neighbour, FS-001-login-extra, is named here; this one is not.\n",
    );

    let run = check_run(&root, false);
    let finding = only(&run, "missing-index-entry");
    assert!(
        finding.message.contains("is not listed in") && !finding.message.contains(MENTION_CLAUSE),
        "§FS-check.3.18.5.1: the token on that line is a different ID: {}",
        finding.message
    );
}

/// §FS-check.3.18.5.1: a declaration heading is neither an entry (§FS-fmt.6.4)
/// nor a mention — a line that *declares* an ID is not a page mentioning it.
#[test]
fn a_declaration_heading_in_the_index_is_not_a_mention() {
    let root = kind_index_repo("a_declaration_heading_in_the_index_is_not_a_mention");
    write(
        &root.join("docs/specs/README.md"),
        "# Specs\n\n- [§FS-001-login](FS-001-login.md#fs-001-login-a-user-logs-in)\n\n\
         ## FS-002-logout: A user logs out\n\nBody.\n",
    );

    let run = check_run(&root, false);
    let finding = only(&run, "missing-index-entry");
    assert!(
        finding.message.contains("FS-002-logout"),
        "the declared-in-place ID is the one owed an entry: {}",
        finding.message
    );
    assert!(
        finding.message.contains("is not listed in") && !finding.message.contains(MENTION_CLAUSE),
        "§FS-check.3.18.5.1: its own heading is the only occurrence, and a \
         heading is not a mention: {}",
        finding.message
    );
}

/// §FS-check.3.18.7 with §FS-check.3.18.5.1: where the index file could not be
/// read there is no text to look at, so the parenthesis stands alone and the two
/// clauses never appear on one line.
#[test]
fn a_missing_index_file_never_carries_the_mention_clause() {
    let root = kind_index_repo("a_missing_index_file_never_carries_the_mention_clause");

    let run = check_run(&root, false);
    let finding = only(&run, "missing-index-entry");
    assert!(
        finding.message.contains("(the index file does not exist)")
            && finding.message.contains("is not listed in"),
        "§FS-check.3.18.7 keeps its whole sentence: {}",
        finding.message
    );
    assert!(
        !finding.message.contains(MENTION_CLAUSE),
        "§FS-check.3.18.5.1: no index text, so nothing can appear in it: {}",
        finding.message
    );
}

/// §FS-check.3.17 / §FS-check.3.17.6: the 3.17/3.18 split is unmoved. A bare
/// citation `fmt --write` would wrap is still `unlinked-index-entry` at its own
/// line, and never earns the mention clause of a rule it does not reach.
#[test]
fn a_repairable_bare_entry_never_earns_the_mention_clause() {
    let root = kind_index_repo("a_repairable_bare_entry_never_earns_the_mention_clause");
    write(
        &root.join("docs/specs/README.md"),
        "# Specs\n\n- §FS-001-login\n",
    );

    let run = check_run(&root, false);
    assert!(
        !codes(&run).contains(&"missing-index-entry".to_string()),
        "one cause, one finding: {:?}",
        findings(&run)
    );
    assert!(
        only(&run, "unlinked-index-entry")
            .message
            .contains("run `grund fmt --write`"),
        "§FS-check.3.17.3: the command that repairs it, unchanged"
    );
    assert!(
        run.report
            .errors
            .iter()
            .chain(run.report.warnings.iter())
            .all(|diagnostic| !diagnostic.message.contains(MENTION_CLAUSE)),
        "§FS-check.3.18.5.1 belongs to the rule this case does not reach: {:?}",
        findings(&run)
    );
}

/// §FS-check.3.18.5: the form the finding names follows `[reference] strict`. Off
/// strict mode an unmarked link is an entry, so a clause demanding the marker
/// would be false about the page it is printed on — which is the defect this
/// wording exists to remove, pointed one mode over.
#[test]
fn off_strict_the_clause_names_the_form_that_mode_requires() {
    let root = kind_index_repo_loose("off_strict_the_clause_names_the_form_that_mode_requires");
    write(
        &root.join("docs/specs/README.md"),
        "# Specs\n\n- FS-001-login\n",
    );

    let run = check_run(&root, false);
    let finding = only(&run, "missing-index-entry");
    assert!(
        finding.message.contains(LOOSE_MENTION_CLAUSE)
            && finding.message.contains("with or without the `§` marker"),
        "§FS-check.3.18.5: off strict the named form is a link whose text is the ID, \
         with the marker optional: {}",
        finding.message
    );
    assert!(
        !finding.message.contains(MENTION_CLAUSE),
        "§FS-check.3.17.5: an unmarked link is a correct entry here, so the marked \
         form is not the form this run requires: {}",
        finding.message
    );
}
