//! Test module: the records gate compares each suffix's multi-line string and quote reading
//! (§FS-config-migrate.5.1, §FS-languages.multi-line-string.suffix). The reading a file of the
//! suffix gets, not the suffix that names it: `.h` read under version 1 and `.h` mapped to the
//! built-in `c` differ, `.cc` under version 1 and `.cc` mapped to the built-in `cpp` do not, and
//! a Haskell prime is part of it although no row of the forms table names `hs`. What migration
//! itself writes compares equal, so no version-1 migration starts refusing.
//! Fixture builders retain their temporary-root owner (§FS-repository-maintenance.4.1).

use super::opted_out::Block;
use super::records_gate::compare;
use super::{GateResult, Located, Prepared, translate};
use crate::config::discovery::pending_config;
use crate::config::project::Project;
use crate::config::run::{Invocation, Selection};
use crate::config::v1::default_project;
use crate::testing::{TemporaryRoot, test_root, write};

const FS: &str = "[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\n";
const NAMED: &str = "[id]\nnamed_sections = true\n";

/// Built-in `c`'s and `cpp`'s comment hosts, in the epoch's order, so a version-1 list of them
/// compares equal to the built-in's on the prefix column.
const C_HOSTS: &str = "comment_prefixes = [\"//\", \"/*\", \"*\"]\n";

/// A v1 tree whose `[scan]` table holds `include = ["docs"]` and `scan`, translated, with each
/// `(from, to)` of `edits` replaced once in the candidate's text before it is loaded: the v1
/// project, the loaded candidate and the root. The unedited candidate passes both gates, so a
/// difference the gate finds below is the edits'.
fn edited(name: &str, scan: &str, edits: &[(&str, &str)]) -> (Project, Project, TemporaryRoot) {
    let root = test_root(&format!("migrate-string-forms-{name}"));
    write(&root.join("docs/FS-a.md"), "# FS-a: A\n\nText.\n");
    let text = format!("grund_config_version = 1\n{FS}[scan]\ninclude = [\"docs\"]\n{scan}{NAMED}");
    write(&root.join("grund.toml"), &text);
    let located = Located {
        file: root.join("grund.toml"),
        root: root.clone(),
        config: "grund.toml".into(),
    };
    let Prepared::Translated { report, v1, .. } =
        translate(&located, false, false, &|| true, &|_| Ok(Vec::new()))
            .expect("the v1 file reads")
    else {
        panic!("read as version 2");
    };
    assert!(report.refusals.is_empty(), "refused: {:?}", report.refusals);
    assert_eq!(report.gates, Some(GateResult::Equal), "{name}");
    let mut candidate = report.candidate.expect("a candidate");
    for (from, to) in edits {
        assert!(
            candidate.contains(from),
            "{name}: the candidate has no `{from}`:\n{candidate}"
        );
        candidate = candidate.replacen(from, to, 1);
    }
    let loaded = v2(&root, &candidate);
    (*v1, loaded, root)
}

/// The version-2 file `text` loaded through the v2 reader at `root`, as the gate loads a
/// candidate.
fn v2(root: &TemporaryRoot, text: &str) -> Project {
    let invocation = Invocation::new(root, false, Selection::Project);
    pending_config(text, root, &invocation)
        .unwrap_or_else(|err| panic!("the version-2 file loads: {err:#}\n{text}"))
        .project()
        .clone()
}

/// That the gate's difference is on the `sources` line, on both sides.
fn differs_on_sources(lines: &[String]) {
    assert!(
        lines.iter().any(|line| line.starts_with("- sources:"))
            && lines.iter().any(|line| line.starts_with("+ sources:")),
        "expected a `sources` difference, got: {lines:?}"
    );
}

/// §FS-config-migrate.5.1, §FS-languages.multi-line-string.suffix: version 1 reads a `.h` file's
/// C++ raw strings, because the forms table names `h` in the C++ row, and the built-in `c` reads
/// none. A candidate that maps `h = "c"` is the same on every other column, and the gate names the
/// difference on the `sources` line rather than calling the records equal.
#[test]
fn records_gate_catches_h_mapped_to_c_losing_cpp_raw_strings() {
    let (v1, v2, root) = edited(
        "h-to-c",
        &format!("extensions = [\"md\", \"c\", \"h\"]\n{C_HOSTS}"),
        &[
            (
                "languages = [\"v1_markdown\", \"v1_c_family\"]",
                "languages = [\"v1_markdown\", \"v1_c_family\", \"c\"]",
            ),
            ("h = \"v1_c_family\"", "h = \"c\""),
        ],
    );
    differs_on_sources(&compare(&v1, &v2, &root, Block::Project));
}

/// §FS-config-migrate.5.1, §FS-languages.multi-line-string.suffix: the column holds the reading,
/// not the suffix. `.cc` under version 1 and `.cc` mapped to the built-in `cpp` both read the one
/// C++ row, so they compare equal. This passes before the fix and must keep passing after it.
#[test]
fn records_gate_compares_the_reading_not_the_suffix_for_cc_mapped_to_cpp() {
    let (v1, v2, root) = edited(
        "cc-to-cpp",
        &format!("extensions = [\"md\", \"cpp\", \"cc\"]\n{C_HOSTS}"),
        &[
            (
                "languages = [\"v1_markdown\", \"v1_c_family\"]",
                "languages = [\"v1_markdown\", \"v1_c_family\", \"cpp\"]",
            ),
            ("cc = \"v1_c_family\"", "cc = \"cpp\""),
        ],
    );
    assert_eq!(
        compare(&v1, &v2, &root, Block::Project),
        Vec::<String>::new()
    );
}

/// A version-2 project admitting `markdown`, `haskell` and the definition `myhs`, which has
/// Haskell's host and doc profile, with `.txt` mapped to `txt`.
fn haskell_txt(txt: &str) -> String {
    format!(
        "grund_config_version = 2\n[schema.sources]\n\
         languages = [\"markdown\", \"haskell\", \"myhs\"]\ndeclarations = \"any-comment\"\n\
         [schema.sources.extensions]\ntxt = \"{txt}\"\n\
         [schema.sources.definitions.myhs]\nclass = \"source\"\ncomment_prefixes = [\"--\"]\n\
         doc_comments = \"haddock\"\n\
         [schema.kinds.FS]\nfolders = [\"docs\"]\nindex = false\n"
    )
}

/// §FS-config-migrate.5.1, §FS-languages.multi-line-string.suffix: the column carries the quote
/// reading as well as the row. `.txt` mapped to the built-in `haskell` reads Haskell primes, and
/// `.txt` read through a `source` definition with Haskell's host and profile reads its own suffix,
/// which gives no prime. Neither reads a row, and reader class, prefixes, docstrings, measurement
/// rule and eligibility are equal, so a column that held only the row would compare them equal.
/// No pair of readings of `.hs` itself isolates the prime: after the fix every reading of `.hs`
/// but a foreign built-in's has it, and a foreign built-in brings its own measurement rule.
#[test]
fn records_gate_catches_a_lost_haskell_prime_reading() {
    let root = test_root("migrate-string-forms-haskell-prime");
    write(&root.join("docs/FS-a.md"), "# FS-a: A\n\nText.\n");
    let primes = v2(&root, &haskell_txt("haskell"));
    let none = v2(&root, &haskell_txt("myhs"));
    differs_on_sources(&compare(&primes, &none, &root, Block::Project));
}

/// Every suffix of version 1's default extension list that the forms table names a row for, the
/// two Haskell suffixes, which carry the prime, and `md`.
const FORMS_SUFFIXES: &[&str] = &[
    "md", "rs", "go", "java", "kt", "ts", "tsx", "js", "py", "cpp", "swift", "scala", "php", "cs",
    "hs", "lhs", "h", "cc", "cxx", "hpp", "hh", "hxx", "kts", "jsx", "mjs", "cjs", "dart", "sh",
    "bash", "zsh",
];

/// §FS-config-migrate.5.1, §FS-config-v2.mapping: what migration writes for every suffix that
/// reads a row or a quote reading compares equal, because the candidate's definitions read the
/// suffix the version-1 file reads. This passes before the fix and must keep passing after it: a
/// gate that compared the reading while definitions still read none would refuse here.
#[test]
fn records_gate_passes_the_translation_of_every_forms_suffix() {
    let defaults = default_project().schema.sources.extensions;
    for suffix in FORMS_SUFFIXES {
        assert!(
            defaults.iter().any(|default| default == suffix),
            "{suffix} is in version 1's default list"
        );
    }
    let list = FORMS_SUFFIXES
        .iter()
        .map(|suffix| format!("\"{suffix}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let (v1, v2, root) = edited(
        "every-forms-suffix",
        &format!("extensions = [{list}]\n"),
        &[],
    );
    assert_eq!(
        compare(&v1, &v2, &root, Block::Project),
        Vec::<String>::new()
    );
}
