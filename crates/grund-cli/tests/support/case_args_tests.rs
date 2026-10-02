// The `command.args` contract of the case runner, pinned in a file of its own
// (§AR-core-module-layout.3). Included into the same module, so the tests reach
// `command_args` itself.

/// How a manifest's `command.args` becomes an argv, or is refused
/// (§FS-examples.5.3).
///
/// Every test drives `command_args` — the one reader both e2e cases and
/// examples go through (§FS-examples.5) — over a scratch case holding nothing
/// but the line, rather than the tokenizer behind it: the contract is the argv
/// a case author gets, or the message the runner refuses the line with, and the
/// tokenizer's own shape is the implementation's to choose. A refusal is a
/// panic, as `command.cwd` and the `symlinks` manifest refuse theirs, and is
/// caught here so its message can be read.
#[cfg(test)]
mod command_args_tests {
    use super::command_args;
    use std::fs;
    use std::panic::{self, AssertUnwindSafe};
    use std::path::PathBuf;

    fn manifest_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    /// A case directory of its own for each name: the tests in a binary run in
    /// parallel, and two binaries include this runner.
    fn scratch_case(name: &str) -> PathBuf {
        manifest_dir()
            .join("target/e2e-harness-tests/command-args")
            .join(format!("{}-{name}", std::process::id()))
    }

    /// The `{repo}` argument `command_args` substitutes for the scratch case.
    fn repo_arg(name: &str) -> String {
        let manifest_dir = manifest_dir();
        let repo = scratch_case(name).join("repo");
        repo.strip_prefix(&manifest_dir)
            .unwrap_or(&repo)
            .to_string_lossy()
            .into_owned()
    }

    /// The argv the runner builds from `line`, or the message it refuses
    /// `line` with.
    fn argv(name: &str, line: &str) -> Result<Vec<String>, String> {
        let case = scratch_case(name);
        fs::create_dir_all(&case).expect("create the scratch case");
        fs::write(case.join("command.args"), line).expect("write command.args");
        let manifest_dir = manifest_dir();
        let built = panic::catch_unwind(AssertUnwindSafe(|| {
            command_args(&manifest_dir, &case, name)
        }));
        let _ = fs::remove_dir_all(&case);
        built.map_err(|payload| {
            payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| payload.downcast_ref::<&str>().map(|s| (*s).to_string()))
                .unwrap_or_default()
        })
    }

    /// The refusal of `line`, which must name the case and `command.args`
    /// first and quote the line, here by `quoted`, a run of it with no quote
    /// character in it, so the way the line is quoted stays the runner's call.
    fn refusal(name: &str, line: &str, quoted: &str) -> String {
        let message = match argv(name, line) {
            Ok(args) => panic!("{line:?} must be refused, but ran as {args:?}"),
            Err(message) => message,
        };
        let prefix = format!("{name}: command.args");
        assert!(
            message.starts_with(&prefix),
            "a refusal names the case and command.args first:\n{message}"
        );
        assert!(
            message.contains(quoted),
            "a refusal quotes the line:\n{message}"
        );
        message
    }

    fn args(words: &[&str]) -> Vec<String> {
        words.iter().map(|word| (*word).to_string()).collect()
    }

    #[test]
    fn a_single_quoted_run_is_one_argument_with_its_quotes_removed() {
        assert_eq!(
            argv("single-quoted", "id FS 'Password reset' --explain\n"),
            Ok(args(&["id", "FS", "Password reset", "--explain"]))
        );
    }

    #[test]
    fn inside_one_quote_the_other_is_an_ordinary_character() {
        assert_eq!(
            argv("other-quote", "id FS 'say \"hi\"'\n"),
            Ok(args(&["id", "FS", "say \"hi\""]))
        );
        assert_eq!(
            argv("other-quote", "id FS \"it's\"\n"),
            Ok(args(&["id", "FS", "it's"]))
        );
    }

    #[test]
    fn adjacent_parts_join_and_an_empty_pair_is_an_empty_argument() {
        assert_eq!(
            argv("adjacent", "check --rule='a b'\"c d\"e x\n"),
            Ok(args(&["check", "--rule=a bc de", "x"]))
        );
        assert_eq!(
            argv("adjacent", "id FS '' \"\"\n"),
            Ok(args(&["id", "FS", "", ""]))
        );
    }

    #[test]
    fn a_backslash_inside_single_quotes_is_an_ordinary_character() {
        assert_eq!(
            argv("kept-backslash", "id FS 'C:\\dir' --explain\n"),
            Ok(args(&["id", "FS", "C:\\dir", "--explain"]))
        );
    }

    /// The pre-existing contract, which the fix must leave alone: plain words,
    /// a double-quoted run, the placeholders, and a blank line.
    #[test]
    fn plain_words_and_double_quotes_keep_their_meaning() {
        assert_eq!(
            argv("plain", "check {repo}\n"),
            Ok(vec!["check".to_string(), repo_arg("plain")])
        );
        assert_eq!(
            argv(
                "plain",
                "check --rule \"Each FS must have one chapter.\" --only-rule\n"
            ),
            Ok(args(&[
                "check",
                "--rule",
                "Each FS must have one chapter.",
                "--only-rule"
            ]))
        );
        assert_eq!(
            argv("plain", "id skill \"Review\"\n"),
            Ok(args(&["id", "skill", "Review"]))
        );
        assert_eq!(argv("plain", "  \n "), Ok(Vec::new()));
    }

    /// What the runner must neither expand nor refuse: §FS-examples.5.3 gives
    /// these characters no meaning, so they reach `grund` as written.
    #[test]
    fn every_other_character_reaches_grund_as_written() {
        assert_eq!(
            argv(
                "literal",
                "show FS-001-login#1 $HOME * ? ~ # | ; {other} `x` {repo}\n"
            ),
            Ok(vec![
                "show".to_string(),
                "FS-001-login#1".to_string(),
                "$HOME".to_string(),
                "*".to_string(),
                "?".to_string(),
                "~".to_string(),
                "#".to_string(),
                "|".to_string(),
                ";".to_string(),
                "{other}".to_string(),
                "`x`".to_string(),
                repo_arg("literal"),
            ])
        );
    }

    fn assert_refused_as_unclosed(name: &str, line: &str) {
        let message = refusal(name, line, "Password reset --explain");
        assert!(
            message.contains("unclosed"),
            "the refusal says the quote is unclosed:\n{message}"
        );
    }

    #[test]
    fn an_unclosed_single_quote_is_refused_naming_the_case_and_the_line() {
        assert_refused_as_unclosed("unclosed-single", "id FS 'Password reset --explain\n");
    }

    #[test]
    fn an_unclosed_double_quote_is_refused_naming_the_case_and_the_line() {
        assert_refused_as_unclosed("unclosed-double", "id FS \"Password reset --explain\n");
    }

    #[test]
    fn a_backslash_outside_single_quotes_is_refused_asking_for_quotes() {
        for line in [
            "id FS Password\\ reset --explain\n",
            "id FS \"Password\\ reset\" --explain\n",
        ] {
            let message = refusal("backslash", line, "--explain");
            assert!(
                message.contains("backslash") && message.contains("quote"),
                "the refusal names the backslash and asks for quotes:\n{message}"
            );
        }
    }
}
