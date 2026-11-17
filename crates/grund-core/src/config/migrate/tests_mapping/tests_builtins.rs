//! Test module: §FS-config-v2.schema.sources.migration as agent-grounds/grund#1384 amends it.
//! Migration names a built-in row where every suffix the row claims is read by version 1, and
//! each reads as the row reads it. Hosts are compared as a set (§FS-config-migrate.5.1), and
//! the `mapped:` lines name the built-ins (§FS-config-migrate.mapping). Every case must also pass
//! the records gate. These cases are kept out of `every_mapping_case_holds_its_outcome` until the
//! fix, so that test stays green. `a_lone_sh_stays_on_its_definition` already holds today.

use super::{Case, Outcome::*, case, missed, translate_case};

/// Built-in rows a version-1 file reads exactly as the row does, and the definitions that stay.
const NAMES_BUILTINS: &[Case] = &[
    case(
        "[scan] extensions",
        then_fs!(
            "[scan]\nextensions = [\"md\", \"java\"]\ncomment_prefixes = [\"//\", \"/*\", \"*\"]\n"
        ),
        &[
            Mapped("languages = [\"v1_markdown\", \"java\"]\n"),
            Mapped("md = \"v1_markdown\"\n"),
            Lacks("java = "),
            Lacks("[schema.sources.definitions.v1_c_family]"),
            Maps(
                "[schema.sources] languages and extensions, built-in java, definition v1_markdown",
            ),
            Maps(
                "comment_prefixes of definition v1_markdown, and the comment hosts of built-in java",
            ),
        ],
    ),
    case(
        "[scan] comment_prefixes",
        then_fs!(
            "[scan]\nextensions = [\"md\", \"java\"]\ncomment_prefixes = [\"//\", \"*\", \"/*\"]\n"
        ),
        &[
            Mapped("languages = [\"v1_markdown\", \"java\"]\n"),
            Lacks("java = "),
            Lacks("[schema.sources.definitions.v1_c_family]"),
        ],
    ),
    case(
        "[scan] extensions",
        then_fs!("[scan]\nextensions = [\"md\"]\n"),
        &[
            Mapped("languages = [\"markdown\"]\n"),
            Lacks("[schema.sources.extensions]"),
            Lacks("[schema.sources.definitions."),
            Maps("[schema.sources] languages, built-in markdown"),
        ],
    ),
    case(
        "[scan] extensions",
        then_fs!(
            "[scan]\nextensions = [\"md\", \"sh\", \"bash\", \"zsh\"]\ncomment_prefixes = [\"#\"]\n"
        ),
        &[
            Mapped("languages = [\"v1_markdown\", \"shell\"]\n"),
            Lacks("sh = "),
            Lacks("v1_shell"),
        ],
    ),
    case(
        "[scan] extensions",
        then_fs!(
            "[scan]\nextensions = [\"md\", \"java\", \"h\"]\ncomment_prefixes = [\"//\", \"/*\", \"*\"]\n"
        ),
        &[
            Mapped("languages = [\"v1_markdown\", \"java\", \"v1_c_family\"]\n"),
            Mapped("h = \"v1_c_family\"\n"),
            Lacks("java = "),
        ],
    ),
    case(
        "[scan] extensions",
        then_fs!(
            "[scan]\nextensions = [\"md\", \"java\", \"kt\", \"gradle\", \"h\"]\n\
             comment_prefixes = [\"//\", \"/*\", \"*\"]\n"
        ),
        &[
            Mapped(
                "languages = [\"v1_markdown\", \"java\", \"kotlin\", \"v1_plain\", \"v1_c_family\"]\n",
            ),
            Mapped("gradle = \"v1_plain\"\n"),
            Mapped("h = \"v1_c_family\"\n"),
            Lacks("java = "),
            Lacks("kt = "),
            Maps(
                "[schema.sources] languages and extensions, built-ins java and kotlin, \
                 definitions v1_markdown, v1_plain and v1_c_family",
            ),
        ],
    ),
];

/// `shell` also claims `bash` and `zsh`, which this file does not read, so `.sh` keeps its
/// definition. This holds before the fix and must hold after it.
const KEEPS_A_LONE_SH: &[Case] = &[case(
    "[scan] extensions",
    then_fs!("[scan]\nextensions = [\"md\", \"sh\"]\ncomment_prefixes = [\"#\"]\n"),
    &[
        Mapped("languages = [\"v1_markdown\", \"v1_shell\"]\n"),
        Mapped("sh = \"v1_shell\"\n"),
    ],
)];

/// Each case of `cases` translated, with every outcome it misses; `offset` keeps the roots apart.
fn failures(cases: &[Case], offset: usize) -> Vec<String> {
    let mut failures = Vec::new();
    for (index, case) in cases.iter().enumerate() {
        let report = translate_case(offset + index, case);
        for outcome in case.expect {
            if let Some(why) = missed(outcome, &report) {
                failures.push(format!("{} ({:?}): {why}", case.row, case.toml));
            }
        }
    }
    failures
}

/// §FS-config-v2.schema.sources.migration, §FS-config-migrate.mapping: a suffix read as its
/// built-in row reads it is written as that row, in `languages` and in the `mapped:` lines. The
/// records gate passes on each. Today every suffix gets a `v1_*` definition.
#[test]
#[ignore = "red until #1384 names a built-in where it reads as version 1; implement removes this"]
fn migration_names_a_builtin_where_it_reads_as_version_1() {
    let failures = failures(NAMES_BUILTINS, 900);
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// §FS-config-v2.schema.sources.migration: a row is named only when it claims nothing version 1
/// does not read.
#[test]
fn a_lone_sh_stays_on_its_definition() {
    let failures = failures(KEEPS_A_LONE_SH, 950);
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}
