//! All original cases and applicable variants, with no root-config filter
//! (§REQ-one-answer.proof.matrix). Receipts arise from completed entrypoints;
//! applicability and independent witnesses are validated by acceptance.

use super::agreement_host::{Host, repository};
use super::agreement_oracles::{golden, rows, text, witnesses};
use super::agreement_questions::{coordinate_questions, tree_questions};
use super::{
    agreement_compare as compare, agreement_editor as editor, agreement_evidence as evidence,
};
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

pub const VARIANTS: &[&str] = &["whole-tree", "narrowed", "path-argument", "editor-overlay"];
pub const SURFACES: &[&str] = &[
    "check",
    "show",
    "show-batch",
    "refs",
    "list",
    "list-size",
    "cover",
    "cover-lines",
    "fmt",
    "editor",
    "python",
    "node",
];

pub fn invoke(cwd: &Path, args: &[String], input: Option<&str>) -> Output {
    invoke_program(cwd, env!("CARGO_BIN_EXE_grund"), args, input)
}

fn invoke_program(cwd: &Path, program: &str, args: &[String], input: Option<&str>) -> Output {
    let start = std::time::Instant::now();
    let mut command = Command::new(program);
    command
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().expect("actual CLI entrypoint");
    if let Some(input) = input {
        let result = child.stdin.take().unwrap().write_all(input.as_bytes());
        if let Err(error) = result {
            assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe);
        }
    } else {
        drop(child.stdin.take());
    }
    let result = child.wait_with_output().expect("completed CLI entrypoint");
    let borrowed: Vec<_> = args.iter().map(String::as_str).collect();
    if program == env!("CARGO_BIN_EXE_grund") {
        evidence::cli(cwd, &borrowed, &result);
        evidence::stdin(input);
    } else {
        evidence::receipt(
            "external",
            cwd,
            json!({"program":program,"args":args,"stdin":input}),
            json!({"exit":result.status.code(),"stdout":result.stdout,"stderr":result.stderr}),
        );
    }
    evidence::seconds(start.elapsed().as_secs_f64());
    result
}

fn manifest(context: &Value, errors: &mut Vec<String>) {
    let request = &context["manifestRequest"];
    if context["manifests"].get("requires-unix-shell").is_some() && !cfg!(unix) {
        evidence::receipt(
            "manifest-inapplicable",
            Path::new(&text(context, "base")),
            context.clone(),
            json!({"reason":"POSIX fetcher on a non-Unix platform","point":"§FS-fetch.2"}),
        );
        return;
    }
    let (program, args) = super::agreement_manifest::command(context);
    let output = invoke_program(
        Path::new(request["cwd"].as_str().unwrap()),
        program.as_deref().unwrap_or(env!("CARGO_BIN_EXE_grund")),
        &args,
        request["stdin"].as_str(),
    );
    match golden(context, &output) {
        Ok(expected) => evidence::witness(expected),
        Err(error) => errors.push(error),
    }
    // Result trees are independently reviewed expected bytes, not extra inputs.
    let expected = Path::new(context["original"].as_str().unwrap()).join("expected.repo");
    if expected.is_dir() {
        let compared = compare_tree(&expected, Path::new(&text(context, "base")));
        if let Err(error) = compared {
            errors.push(error);
        }
    }
}

pub fn compare_tree(expected: &Path, actual: &Path) -> Result<(), String> {
    fn inventory(path: &Path, root: &Path, out: &mut std::collections::BTreeMap<String, Vec<u8>>) {
        for entry in std::fs::read_dir(path).expect("result tree") {
            let path = entry.unwrap().path();
            if path.is_symlink() {
                continue;
            }
            if path.is_dir() {
                inventory(&path, root, out);
            } else {
                out.insert(
                    path.strip_prefix(root)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/"),
                    std::fs::read(&path).unwrap(),
                );
            }
        }
    }
    let mut expected_files = std::collections::BTreeMap::new();
    let mut actual_files = std::collections::BTreeMap::new();
    inventory(expected, expected, &mut expected_files);
    inventory(actual, actual, &mut actual_files);
    if expected_files == actual_files {
        Ok(())
    } else {
        Err(format!("result-tree golden {} differs", expected.display()))
    }
}

/// Every live corpus directory is visited; preparation cannot return fewer cases
/// than the independently discovered acceptance catalog.
pub fn execute() -> Vec<String> {
    let mut host = Host::new();
    evidence::receipt(
        "host-provenance",
        &repository(),
        json!({}),
        host.provenance.clone(),
    );
    let discovered = host.request(json!({"op":"discover"}));
    let mut errors = Vec::new();
    for name in discovered.as_array().expect("discovered cases") {
        for variant in VARIANTS {
            let context = host.request(json!({"op":"prepare","case":name,"variant":variant}));
            let attribution = json!({"case":name,"variant":variant,"base":context["base"],
                "saved":context["saved"],"cwd":context["cwd"],"path":context["path"]});
            evidence::context(attribution, || {
                let verified = super::agreement_materialization::verify(&context);
                evidence::receipt(
                    "materialization",
                    Path::new(&text(&context, "base")),
                    json!({}),
                    context.clone(),
                );
                evidence::witness(verified);
                if !context["absent"].is_null() {
                    for surface in SURFACES {
                        evidence::receipt(
                            "inapplicable",
                            Path::new(&text(&context, "base")),
                            json!({"surface":surface,"context":context}),
                            json!({"reason":context["absent"],"point":"§REQ-one-answer.proof.matrix"}),
                        );
                    }
                    if *variant == "whole-tree"
                        && context["links"].as_array().unwrap().is_empty()
                        && context.get("symlinkError").is_none()
                    {
                        manifest(&context, &mut errors);
                    }
                    return;
                }
                // Manifest mutation has its own isolated materialization; generic
                // questions continue on a freshly prepared copy below.
                if *variant == "whole-tree" {
                    manifest(&context, &mut errors);
                }
            });
            if !context["absent"].is_null() {
                continue;
            }
            // Recreate after a manifest that may write; no generic read sees its result.
            let context = if *variant == "whole-tree" {
                host.request(json!({"op":"prepare","case":name,"variant":variant}))
            } else {
                context
            };
            super::agreement_materialization::verify(&context);
            evidence::context(
                json!({"case":name,"variant":variant,"base":context["base"],
                "saved":context["saved"],"cwd":context["cwd"],"path":context["path"]}),
                || {
                    exercise(&mut host, &context, &mut errors);
                },
            );
        }
        host.request(json!({"op":"release","case":name}));
    }
    errors
}

fn exercise(host: &mut Host, context: &Value, errors: &mut Vec<String>) {
    let start = errors.len();
    let loaded = editor::snapshot(context, errors);
    if let Some(loaded) = &loaded {
        editor::saved_control(context, loaded, errors);
    }
    let cwd = text(context, "cwd");
    let mut coordinates = BTreeSet::new();
    let mut tree_answers = std::collections::BTreeMap::new();
    for question in tree_questions(context) {
        let output = invoke(Path::new(&cwd), &question.cli_args(), None);
        let actual = rows(&output);
        match witnesses(context, &actual, question.point) {
            Ok(expected) => evidence::witness(expected),
            Err(error) => errors.push(error),
        }
        let answer = compare::hosts(host, context, &question, &output, errors);
        tree_answers.insert(question.surface, answer["python"].clone());
        if question.surface == "check"
            && let Some(loaded) = &loaded
        {
            editor::check(context, loaded, &answer["python"], errors);
        }
        if question.surface == "list-size" {
            for row in answer["python"]["result"]["entries"]
                .as_array()
                .into_iter()
                .flatten()
            {
                let mut coordinate = row["id"].as_str().expect("listed ID").to_string();
                if let Some(section) = row["section"].as_str() {
                    coordinate.push_str(row["section_separator"].as_str().unwrap_or("."));
                    coordinate.push_str(section);
                }
                if let Some(project) = row["project"].as_str() {
                    if !coordinate.starts_with(&format!("{project}/")) {
                        coordinate = format!("{project}/{coordinate}");
                    }
                }
                coordinates.insert(coordinate);
            }
        }
    }
    super::agreement_facts::declarations(context, &tree_answers, errors);
    if coordinates.is_empty() {
        coordinates.insert("FS-agreement-absent".into());
    }
    for coordinate in coordinates {
        for question in coordinate_questions(context, &coordinate) {
            let output = invoke(Path::new(&cwd), &question.cli_args(), None);
            let actual = rows(&output);
            match witnesses(context, &actual, question.point) {
                Ok(expected) => evidence::witness(expected),
                Err(error) => errors.push(error),
            }
            let answer = compare::hosts(host, context, &question, &output, errors);
            if question.surface == "show" {
                super::agreement_facts::sections(
                    context,
                    &coordinate,
                    &answer["python"],
                    &tree_answers["show-batch"],
                    &tree_answers["list"],
                    loaded.as_ref(),
                    errors,
                );
            } else {
                super::agreement_facts::citations(
                    context,
                    &answer["python"],
                    &tree_answers["cover"],
                    errors,
                );
            }
            if question.surface == "show"
                && let Some(loaded) = &loaded
            {
                let bodies: Vec<_> = actual
                    .iter()
                    .filter(|row| row.get("body").is_some())
                    .cloned()
                    .collect();
                editor::show(
                    context,
                    loaded,
                    &coordinate,
                    &bodies,
                    !output.status.success(),
                    errors,
                );
            }
        }
    }
    super::agreement_mutations::execute(host, context, errors);
    for error in &mut errors[start..] {
        *error = format!(
            "{} / {}: {error}",
            text(context, "case"),
            text(context, "variant")
        );
    }
}
