//! Validate copied/held/applicability context independently of the producer
//! (§REQ-one-answer.proof.matrix, §FS-scan.text.buffer).

use super::agreement_corpus::compare_tree;
use super::agreement_oracles::text;
use serde_json::{Value, json};
use std::fs;
use std::path::Path;

pub fn verify(context: &Value) -> Value {
    let original = text(context, "original");
    let original = Path::new(&original).join("repo");
    let base = text(context, "base");
    let base = Path::new(&base);
    assert_eq!(original.is_dir(), context["inputExists"].as_bool().unwrap());
    let manifest_dir = Path::new(context["original"].as_str().unwrap());
    for (name, contents) in context["manifests"].as_object().unwrap() {
        assert_eq!(
            fs::read_to_string(manifest_dir.join(name)).unwrap(),
            contents.as_str().unwrap()
        );
    }
    if original.is_dir() && context.get("symlinkError").is_none() {
        compare_tree(&original, base).expect("copy is the physical original's bytes");
        if let Some(manifest) = context["manifests"]["symlinks"].as_str() {
            let declared: Vec<_> = manifest
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .collect();
            assert_eq!(declared.len(), context["links"].as_array().unwrap().len());
            for line in declared {
                let (link, target) = line.split_once("->").expect("symlink arrow");
                assert_eq!(
                    fs::read_link(base.join(link.trim())).unwrap(),
                    Path::new(target.trim())
                );
            }
        }
    }
    if !context["overlays"].as_object().unwrap().is_empty() {
        let relative = text(context, "overlayRelative");
        let saved = Path::new(context["saved"].as_str().unwrap()).join(&relative);
        let held = text(context, "heldBytes");
        assert_eq!(fs::read(&saved).unwrap(), held.as_bytes());
        if context["bufferOnly"] == true {
            assert!(!base.join(&relative).exists());
        } else {
            assert_ne!(fs::read(base.join(&relative)).unwrap(), held.as_bytes());
        }
        assert_eq!(
            context["overlays"][base.join(relative).to_string_lossy().as_ref()],
            json!(held)
        );
    }
    let reason = context["absent"].as_str();
    fn readable_files(root: &Path, directory: &Path, result: &mut Vec<String>) {
        for entry in fs::read_dir(directory).expect("physical applicability walk") {
            let path = entry.unwrap().path();
            if path.is_symlink() || path.file_name().unwrap().to_string_lossy().starts_with('.') {
                continue;
            }
            if path.is_dir() {
                readable_files(root, &path, result);
            } else if path.file_name().unwrap() != "grund.toml"
                && path.file_name().unwrap() != "AGENTS.md"
                && fs::read_to_string(&path).is_ok()
            {
                result.push(
                    path.strip_prefix(root)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            }
        }
    }
    let mut independently_readable = Vec::new();
    readable_files(base, base, &mut independently_readable);
    independently_readable.sort();
    let mut reported_files: Vec<_> = context["textFiles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|file| file.as_str().unwrap().to_string())
        .collect();
    reported_files.sort();
    assert_eq!(
        independently_readable, reported_files,
        "applicability cannot invent an empty file set"
    );
    fn all_paths(root: &Path, directory: &Path, files: &mut Vec<String>, dirs: &mut Vec<String>) {
        for entry in fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_symlink() {
                continue;
            }
            let relative = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            if path.is_dir() {
                dirs.push(relative);
                all_paths(root, &path, files, dirs);
            } else {
                files.push(relative);
            }
        }
    }
    let mut physical_files = Vec::new();
    let mut physical_dirs = Vec::new();
    all_paths(base, base, &mut physical_files, &mut physical_dirs);
    physical_files.sort();
    physical_dirs.sort();
    assert_eq!(json!(physical_files), context["files"]);
    assert_eq!(json!(physical_dirs), context["directories"]);
    if let Some(reason) = reason {
        match reason {
            "no input repository: command grammar/process case" => assert!(!original.exists()),
            "Windows refused symlink creation (winerror 1314)" => {
                assert!(cfg!(windows));
                assert_eq!(context["symlinkError"]["winerror"], 1314);
                assert!(manifest_dir.join("symlinks").is_file());
            }
            "no proper directory below the input root" => {
                assert_eq!(context["variant"], "narrowed");
                assert!(physical_dirs.is_empty());
            }
            "no proper file or directory path below the input root" => {
                assert_eq!(context["variant"], "path-argument");
                assert!(physical_files.is_empty() && physical_dirs.is_empty());
            }
            other => panic!("unrecognized applicability: {other}"),
        }
    } else {
        assert!(Path::new(context["path"].as_str().unwrap()).exists());
        if context["variant"] == "narrowed" {
            let cwd = Path::new(context["cwd"].as_str().unwrap());
            assert!(cwd.is_dir() && cwd != Path::new(context["saved"].as_str().unwrap()));
            assert_eq!(context["explicit"], false);
        } else if context["variant"] == "path-argument" {
            assert_ne!(context["path"], context["saved"]);
            assert_eq!(context["explicit"], true);
        }
    }
    json!({"oracle":"original-manifest-and-physical-bytes","point":"§REQ-one-answer.proof.matrix",
        "original":original,"reason":context["absent"],"heldBytes":context["heldBytes"]})
}
