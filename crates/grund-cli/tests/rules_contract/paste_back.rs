//! The issue's reproducer as a contract (§FS-rules.3.5.4): every form a
//! refused rule offers is accepted when it is pasted back, a refusal that
//! offers none is its reason alone with `known kinds:` only where a kind is
//! missing, and a rule declaration's finding carries the same text. It runs in
//! a repository whose one kind is `FS`, and in one configuring `FS`, `REQ` and
//! `GOAL` with named sections off and on.

use super::support::{case_repo, run, scratch_from, text, write};
use std::fs;
use std::path::{Path, PathBuf};

/// What a refused sentence is answered with.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Answer {
    /// One or two forms, each accepted when pasted back.
    Form,
    /// The reason alone.
    NoForm,
    /// The reason, then `known kinds:`.
    NoFormKinds,
}

use Answer::{Form, NoForm, NoFormKinds};

/// The repository whose one kind is `FS`, named sections on.
fn fs_only() -> PathBuf {
    case_repo("list-selector-refused-numbered-section")
}

/// Kinds `FS`, `REQ` and `GOAL`, named sections off.
fn named_off() -> PathBuf {
    case_repo("check-rule-named-off-rebuilt-subject")
}

/// The same tree with `[id] named_sections = true`.
fn named_on() -> PathBuf {
    let root = scratch_from(&named_off(), "paste-back-named-on");
    let config = fs::read_to_string(root.join("grund.toml")).expect("fixture config");
    fs::write(
        root.join("grund.toml"),
        config.replace("named_sections = false", "named_sections = true"),
    )
    .expect("enable named sections");
    root
}

/// At least one sentence per row of §FS-rules.3.5.4.5, with the answer it gets.
fn fs_only_sentences() -> Vec<(&'static str, Answer)> {
    vec![
        ("Each FS may not cite any FS.", Form),
        ("Each FS should cite no FS.", Form),
        ("each FS must cite at least one FS.", Form),
        ("Each FS must cite at least 1 FS", Form),
        ("Each FS should be cited by exactly 1 FS.", Form),
        (
            "FS-login.requirements must cite each FS at least 1 times.",
            Form,
        ),
        ("FS-login must cite each FS exactly 2.", Form),
        ("Each FS must cite exactly 02 FS.", Form),
        ("Each FS must cite at least 0 FS.", NoForm),
        ("Each FS must cite a FS.", Form),
        (
            "Each FS must cite at least one GOAL and must not cite any AR.",
            NoFormKinds,
        ),
        ("Each FS should not cite at least one FS.", Form),
        ("FS-login must have exactly one requirements section.", Form),
        ("FS-login should have at most 2 requirements chapter.", Form),
        (
            "FS-login must have exactly one  requirements chapter.",
            Form,
        ),
        (
            "FS-login must have exactly one Goal and hypothesis chapter.",
            NoForm,
        ),
        (
            "FS-login.requirements must have exactly one should chapter.",
            Form,
        ),
        (
            "The requirements chapter of each FS should have at least one should chapter.",
            Form,
        ),
        ("FS-*.requirements must cite at least one FS.", Form),
        ("FS.* must cite at least one FS.", Form),
        ("FS-login.2 should cite at most 2 FS.", Form),
        (
            "Each chapter of each FS must be cited by at least one FS.",
            Form,
        ),
        ("Each */FS must cite at least one FS.", Form),
        ("FSbogus must cite at least one FS.", NoFormKinds),
        ("Each POLICY must cite at least one FS.", NoFormKinds),
        (
            "FS-*.requirements must cite at least one GOAL.",
            NoFormKinds,
        ),
        (
            "Each file in vendor/ must cite at least one FS.",
            NoFormKinds,
        ),
        ("Each FS must cite at least one POLICY.", NoFormKinds),
        ("Each FS must cite at least one fs.", NoFormKinds),
        ("Each FS cites at least one FS.", NoForm),
        ("Each FS must reference at least one FS.", NoForm),
        ("Each FS must cite some FS.", NoForm),
        ("Each FS must cite at least FS.", NoForm),
        ("Each FS must cite at least two FS.", NoForm),
    ]
}

/// Sentences over `FS`, `REQ` and `GOAL` with named sections on.
fn named_on_sentences() -> Vec<(&'static str, Answer)> {
    vec![
        ("FS-login.requirements must cite at least 1 REQ.", Form),
        ("Each REQ should be cited by exactly 1 FS or GOAL.", Form),
        ("FS-login.requirements.1 must cite at least one GOAL.", Form),
        ("FS-login.* must cite at least one REQ.", Form),
        (
            "Each chapter of each REQ must cite at least one GOAL.",
            Form,
        ),
        ("REQ-password.2 must cite at least one GOAL.", Form),
        ("Each REQ must have exactly 02 requirements chapters.", Form),
        ("Each GOAL must cite a REQ.", Form),
        (
            "The requirements chapter of each FS must cite each REQ exactly 1 times.",
            Form,
        ),
        (
            "GOAL-access must have exactly one Goal and hypothesis chapter.",
            NoForm,
        ),
        (
            "Each FS must cite at least one GOAL or POLICY.",
            NoFormKinds,
        ),
        (
            "Each FS must cite at least one GOAL and must not cite any AR.",
            NoFormKinds,
        ),
        ("Each POLICY must cite at least one GOAL.", NoFormKinds),
    ]
}

/// Sentences over `FS`, `REQ` and `GOAL` with named sections off (§FS-rules.3.5.2).
fn named_off_sentences() -> Vec<(&'static str, Answer)> {
    vec![
        ("FS-*.requirements must cite at least one GOAL.", Form),
        (
            "FS-login.Requirements should be cited by at most 2 GOAL.",
            Form,
        ),
        ("FS-login.requirements must cite at least 1 GOAL.", Form),
        ("FS-login.2 must cite at least one GOAL.", Form),
        ("Each REQ must cite at least 1 GOAL.", Form),
        (
            "FS-login.requirements must cite at least one POLICY.",
            NoFormKinds,
        ),
        (
            "POLICY.requirements must cite at least 1 GOAL.",
            NoFormKinds,
        ),
        (
            "FS-login.requirements must reference at least one REQ.",
            NoForm,
        ),
    ]
}

/// `check --rule` with `sentence` in `root`: the stderr of a refusal before the
/// scan, or `None` where the sentence was accepted (§FS-rules.4).
fn refusal(root: &Path, sentence: &str) -> Option<String> {
    let output = run(root, &["check", ".", "--rule", sentence]);
    (output.status.code() == Some(2)).then(|| text(&output.stderr))
}

/// The forms the first line of `stderr` offers, and whether they are labelled
/// `after enabling it`. The §FS-rules.3.5.1 explanation is not part of a form.
fn offered_forms(stderr: &str) -> (Vec<String>, bool) {
    let line = stderr.lines().next().unwrap_or_default();
    let line = line
        .strip_suffix(" NAME forbids whitespace anywhere.")
        .unwrap_or(line);
    if let Some((_, form)) = line.split_once("; accepted form after enabling it: ") {
        return (vec![form.to_string()], true);
    }
    if let Some((_, form)) = line.split_once("; accepted form: ") {
        return (vec![form.to_string()], false);
    }
    if let Some((_, pair)) = line.split_once("; accepted forms: ") {
        let quoted = pair
            .strip_prefix('"')
            .and_then(|pair| pair.strip_suffix('"'));
        let forms = quoted.and_then(|pair| {
            pair.split_once("\" or \"")
                .or_else(|| pair.split_once("\" and \""))
        });
        if let Some((first, second)) = forms {
            return (vec![first.to_string(), second.to_string()], false);
        }
    }
    (Vec::new(), false)
}

/// Why the refusal of `sentence` in `root` is not the `answer` it should be,
/// pasting every offered form back where its label says.
fn answered_wrongly(
    root: &Path,
    enabled: Option<&Path>,
    sentence: &str,
    answer: Answer,
) -> Option<String> {
    let Some(stderr) = refusal(root, sentence) else {
        return Some(format!("{sentence:?} was accepted"));
    };
    let (forms, after_enabling) = offered_forms(&stderr);
    let lines: Vec<&str> = stderr.lines().collect();
    let shape = match answer {
        Form if forms.is_empty() => Some("offers no form"),
        Form if lines.len() != 1 => Some("prints more than its error line"),
        NoForm | NoFormKinds if !forms.is_empty() || stderr.contains("accepted form") => {
            Some("offers a form")
        }
        NoForm if lines.len() != 1 => Some("prints more than its reason"),
        NoFormKinds if lines.len() != 2 || !lines[1].starts_with("known kinds: ") => {
            Some("is not its reason and then `known kinds:`")
        }
        _ => None,
    };
    if let Some(shape) = shape {
        return Some(format!("{sentence:?} {shape}: {stderr:?}"));
    }
    for form in forms {
        let target = match (after_enabling, enabled) {
            (true, Some(on)) => on,
            (true, None) => return Some(format!("{sentence:?} needs no enabling: {stderr:?}")),
            (false, _) => root,
        };
        if let Some(again) = refusal(target, &form) {
            return Some(format!(
                "{sentence:?} offers {form:?}, which is refused: {again:?}"
            ));
        }
        if after_enabling && refusal(root, &form).is_none() {
            return Some(format!(
                "{sentence:?} offers {form:?} after enabling, but it needs none"
            ));
        }
    }
    None
}

/// §FS-rules.3.5.4.3, the issue's property: every form a refused rule offers,
/// in each of the three repositories, is accepted when it is pasted back, and
/// one labelled `after enabling it` is accepted once named sections are on.
#[test]
fn every_offered_form_is_accepted_when_pasted_back() {
    let (off, on) = (named_off(), named_on());
    let mut wrong = Vec::new();
    let mut total = 0;
    for (root, enabled, sentences) in [
        (fs_only(), None, fs_only_sentences()),
        (on.clone(), None, named_on_sentences()),
        (off.clone(), Some(on.as_path()), named_off_sentences()),
    ] {
        for (sentence, answer) in sentences.into_iter().filter(|(_, a)| *a == Form) {
            total += 1;
            wrong.extend(answered_wrongly(&root, enabled, sentence, answer));
        }
    }
    assert!(
        wrong.is_empty(),
        "{} of {total} refusals did not offer a form that pastes back:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}

/// §FS-rules.3.5.4.4: a refusal with nothing to supply offers no form; it is
/// its reason alone, followed by `known kinds:` only where a kind is missing.
#[test]
fn a_refusal_with_nothing_to_supply_is_its_reason_alone() {
    let (off, on) = (named_off(), named_on());
    let mut wrong = Vec::new();
    let mut total = 0;
    for (root, sentences) in [
        (fs_only(), fs_only_sentences()),
        (on, named_on_sentences()),
        (off, named_off_sentences()),
    ] {
        for (sentence, answer) in sentences.into_iter().filter(|(_, a)| *a != Form) {
            total += 1;
            wrong.extend(answered_wrongly(&root, None, sentence, answer));
        }
    }
    assert!(
        wrong.is_empty(),
        "{} of {total} refusals with nothing to supply were not their reason alone:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}

/// `root` copied with a `rules = true` kind `RULE` declaring every sentence
/// of `sentences`, the `n`th as `RULE-p<n>`.
fn with_rule_declarations(root: &Path, name: &str, sentences: &[&str]) -> PathBuf {
    let copy = scratch_from(root, name);
    let config = fs::read_to_string(copy.join("grund.toml")).expect("fixture config");
    let config = format!(
        "{config}\n[[kinds]]\nkind = \"RULE\"\nfolder = \"docs/rules\"\nindex = false\nrules = true\n"
    );
    fs::write(copy.join("grund.toml"), config).expect("add the rule kind");
    for (n, sentence) in sentences.iter().enumerate() {
        write(
            &copy,
            &format!("docs/rules/RULE-p{n}.md"),
            &format!("# RULE-p{n}: {sentence}\n\nWhy the rule exists.\n"),
        );
    }
    copy
}

/// §FS-rules.3.5.4, §FS-rules.7.1: a configured rule declaration's finding
/// carries the text `check --rule` prints on its `error:` line, and never the
/// `known kinds:` line after it, because one front end renders both.
#[test]
fn a_rule_declarations_finding_carries_the_check_rule_text() {
    let mut wrong = Vec::new();
    for (root, name, sentences) in [
        (fs_only(), "paste-back-declarations-fs", fs_only_sentences()),
        (
            named_on(),
            "paste-back-declarations-on",
            named_on_sentences(),
        ),
        (
            named_off(),
            "paste-back-declarations-off",
            named_off_sentences(),
        ),
    ] {
        // A heading's title is trimmed and its runs of spaces are its own business.
        let sentences: Vec<&str> = sentences
            .iter()
            .map(|(sentence, _)| *sentence)
            .filter(|sentence| !sentence.contains("  "))
            .collect();
        let copy = with_rule_declarations(&root, name, &sentences);
        let output = run(&copy, &["check", ".", "--only", "invalid-rule"]);
        let stdout = text(&output.stdout);
        for (n, sentence) in sentences.iter().enumerate() {
            let prefix =
                format!("docs/rules/RULE-p{n}.md:1: error: RULE-p{n} is not a valid rule: ");
            let finding = stdout
                .lines()
                .find_map(|line| line.strip_prefix(prefix.as_str()));
            let line = refusal(&copy, sentence).unwrap_or_default();
            let line = line.lines().next().unwrap_or_default();
            let expected = line.strip_prefix("error: ");
            if finding.is_none() || finding != expected {
                wrong.push(format!(
                    "{sentence:?}: finding {finding:?}\n  check --rule {expected:?}"
                ));
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
