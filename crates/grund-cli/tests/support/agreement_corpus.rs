//! All original cases and applicable variants, with no root-config filter
//! (§REQ-one-answer.proof.matrix). Receipts arise from completed entrypoints;
//! applicability and independent witnesses are validated by acceptance.

use super::agreement_host::{Host, repository};
use super::agreement_oracles::{golden, rows, text};
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

/// A receipt list a lane captured, and the place it takes among the cases asked:
/// `None` for a lane's own startup, else the case's index in discovery order.
type Captured = (Option<usize>, Vec<String>, Vec<evidence::Call>);

/// Every case the acceptance asks is visited; preparation cannot return fewer
/// cases than the independently discovered catalog it was taken from
/// (§REQ-one-answer.proof.matrix.family). The cases run on lanes, each with its
/// own same-source hosts and editor process, which pays for the time a tranche
/// adds (§REQ-one-answer.proof.delivery.payment); the receipts are adopted in
/// discovery order, as one lane would have recorded them.
pub fn execute(manifests: &std::collections::BTreeMap<String, Value>) -> Vec<String> {
    let mut host = Host::new();
    evidence::receipt(
        "host-provenance",
        &repository(),
        json!({}),
        host.provenance.clone(),
    );
    let discovered = host.request(json!({"op":"discover"}));
    let names: BTreeSet<_> = discovered
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert_eq!(
        names,
        super::agreement_families::discovered(),
        "prepared corpus differs from authored discovery"
    );
    let asked: Vec<String> = discovered
        .as_array()
        .expect("discovered cases")
        .iter()
        .map(|name| name.as_str().unwrap().to_string())
        .filter(|name| manifests.contains_key(name))
        .collect();
    // The first host built the native libraries, so a later lane's host finds them fresh.
    let lanes = std::thread::available_parallelism()
        .map_or(1, std::num::NonZero::get)
        .clamp(1, 4)
        .min(asked.len().max(1));
    let queue = std::sync::Mutex::new(asked.iter().enumerate());
    let mut first = Some(host);
    let mut captured: Vec<Captured> = std::thread::scope(|scope| {
        let lanes: Vec<_> = (0..lanes)
            .map(|_| {
                let (host, queue) = (first.take(), &queue);
                scope.spawn(move || lane(host, queue, manifests))
            })
            .collect();
        lanes
            .into_iter()
            .flat_map(|lane| {
                lane.join()
                    .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
            })
            .collect()
    });
    captured.sort_by_key(|(index, ..)| *index);
    let mut errors = Vec::new();
    for (_, found, calls) in captured {
        evidence::adopt(calls);
        errors.extend(found);
    }
    errors
}

/// One lane: its hosts and editor, taking the next case asked until none is left.
fn lane<'a>(
    host: Option<Host>,
    queue: &std::sync::Mutex<impl Iterator<Item = (usize, &'a String)>>,
    manifests: &std::collections::BTreeMap<String, Value>,
) -> Vec<Captured> {
    let mut started = None;
    let calls = evidence::capture(|| {
        let host = host.unwrap_or_else(|| {
            let host = Host::new();
            evidence::receipt(
                "host-provenance",
                &repository(),
                json!({}),
                host.provenance.clone(),
            );
            host
        });
        started = Some((host, super::agreement_editor_worker::Worker::new()));
    });
    let (mut host, mut worker) = started.expect("lane started");
    let mut captured = vec![(None, Vec::new(), calls)];
    loop {
        let next = queue.lock().unwrap().next();
        let Some((index, name)) = next else {
            return captured;
        };
        let mut errors = Vec::new();
        let calls = evidence::capture(|| run(&mut host, &mut worker, name, manifests, &mut errors));
        captured.push((Some(index), errors, calls));
    }
}

/// Every variant of one case, then its materializations released.
fn run(
    host: &mut Host,
    editor_worker: &mut super::agreement_editor_worker::Worker,
    name: &str,
    manifests: &std::collections::BTreeMap<String, Value>,
    errors: &mut Vec<String>,
) {
    for variant in VARIANTS {
        let mut context = host.request(json!({"op":"prepare","case":name,"variant":variant}));
        let facts = &manifests[name]["variants"][*variant];
        super::agreement_truth::bind(&mut context, facts);
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
                    manifest(&context, errors);
                }
                return;
            }
            // Manifest mutation has its own isolated materialization; generic
            // questions continue on a freshly prepared copy below.
            if *variant == "whole-tree" {
                manifest(&context, errors);
            }
        });
        if !context["absent"].is_null() {
            continue;
        }
        // Recreate after a manifest that may write; no generic read sees its result.
        let mut context = if *variant == "whole-tree" {
            host.request(json!({"op":"prepare","case":name,"variant":variant}))
        } else {
            context
        };
        super::agreement_truth::bind(&mut context, facts);
        super::agreement_materialization::verify(&context);
        evidence::context(
            json!({"case":name,"variant":variant,"base":context["base"],
            "saved":context["saved"],"cwd":context["cwd"],"path":context["path"]}),
            || {
                exercise(host, editor_worker, &context, errors);
            },
        );
    }
    editor_worker.release();
    host.request(json!({"op":"release","case":name}));
}

fn exercise(
    host: &mut Host,
    worker: &mut super::agreement_editor_worker::Worker,
    context: &Value,
    errors: &mut Vec<String>,
) {
    let start = errors.len();
    let loaded = editor::snapshot(worker, context, errors);
    if let Some(loaded) = &loaded {
        editor::saved_control(worker, context, loaded, errors);
    }
    let cwd = text(context, "cwd");
    let coordinates = super::agreement_expectations::coordinates(&context["truth"])
        .expect("preflight requires an independent complete coordinate inventory");
    let mut tree_answers = std::collections::BTreeMap::new();
    for mut question in tree_questions(context) {
        if question.path != text(context, "path") {
            let mut scope_context = context.clone();
            scope_context["explicit"] = json!(question.explicit);
            question.scopes =
                host.request(json!({"op":"scope","context":scope_context,"path":question.path}));
        }
        let scoped = question.context(context, "cli");
        let output = invoke(Path::new(&cwd), &question.cli_args(), None);
        super::agreement_truth::cli(&scoped, &question, &output, errors);
        let answer = compare::hosts(host, context, &question, &output, errors);
        tree_answers.insert(question.surface, answer["python"].clone());
        if question.surface == "check"
            && let Some(loaded) = &loaded
        {
            editor::check(
                &question.context(context, "python"),
                loaded,
                &answer["python"],
                errors,
            );
        }
    }
    let mut scoped = context.clone();
    scoped["reportBase"] = context["scopes"]["python"]["reportBase"].clone();
    super::agreement_facts::declarations(&scoped, &tree_answers, loaded.as_ref(), errors);
    for coordinate in coordinates {
        for question in coordinate_questions(context, &coordinate) {
            let output = invoke(Path::new(&cwd), &question.cli_args(), None);
            let actual = rows(&output);
            super::agreement_truth::cli(
                &question.context(context, "cli"),
                &question,
                &output,
                errors,
            );
            let answer = compare::hosts(host, context, &question, &output, errors);
            if question.surface == "show" {
                super::agreement_facts::sections(
                    &scoped,
                    &coordinate,
                    &answer["python"],
                    &tree_answers["show-batch"],
                    &tree_answers["list"],
                    loaded.as_ref(),
                    errors,
                );
            } else {
                super::agreement_facts::citations(
                    &scoped,
                    &answer["python"],
                    &tree_answers["cover"],
                    errors,
                );
            }
            if question.surface == "show" && loaded.is_some() {
                let bodies: Vec<_> = actual
                    .iter()
                    .filter(|row| row.get("body").is_some())
                    .cloned()
                    .collect();
                editor::show(
                    worker,
                    &question.context(context, "cli"),
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
