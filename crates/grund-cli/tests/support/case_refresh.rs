// The refresh half of the case runner, in a file of its own along the same seam
// as `case_report.rs`: what `UPDATE_EXPECTED=1` writes, and what the pass says
// about it afterwards. `case_report.rs` is the comparison half — there a
// mismatch is data, collected per case and rendered once at the end of the pass;
// here a write is data, collected and rendered by the same reporter. Included
// into the same module, so both halves share `relative_files`, `write_expected`
// and `CaseOutcome`.

/// One golden file a refresh pass moved (§AR-workspace.9.5): the surface it
/// belongs to, the file as the accounting names it — relative to the case, so a
/// tree golden reads as its path inside `expected.repo` — and which way it moved.
pub struct GoldenChange {
    surface: &'static str,
    file: String,
    removed: bool,
}

/// Refresh every golden surface of one case, in the same fixed
/// exit → stdout → stderr → repo order the compare path walks
/// (§AR-workspace.9.3), and return what moved rather than a bare
/// [`CaseOutcome::Ran`]. The fourth write is the one that was missing: the
/// refresh branch returned before the final-tree surface was reached at all, so
/// a stale `expected.repo` golden survived a green refresh and failed the very
/// next compare pass (§AR-workspace.9.1.5).
fn refresh_case(
    manifest_dir: &Path,
    case: &Path,
    name: &str,
    exit: i32,
    stdout: &str,
    stderr: &str,
) -> CaseOutcome {
    let mut changes = Vec::new();
    for (surface, file, content) in [
        ("exit", "expected.exit", format!("{exit}\n")),
        ("stdout", "expected.stdout", stdout.to_string()),
        ("stderr", "expected.stderr", stderr.to_string()),
    ] {
        if write_expected(&case.join(file), &content) {
            changes.push(GoldenChange {
                surface,
                file: file.to_string(),
                removed: false,
            });
        }
    }
    let tree = refresh_expected_repo(manifest_dir, case, name, &mut changes);
    CaseOutcome::Refreshed {
        case: name.to_string(),
        tree,
        changes,
    }
}

/// The tree surface's writer, the counterpart of [`assert_expected_repo`]: it
/// reuses that reader's own walk, so writer and reader cannot disagree about
/// what a tree golden holds. The golden's file set is made equal to the produced
/// tree's, because the file list is compared before the bytes are and a golden
/// file left behind would fail the next compare pass (§AR-workspace.9.1.5).
///
/// Returns whether the case carries a tree at all, which is what the accounting
/// counts. A case with no `expected.repo` pins nothing here and gains no golden.
fn refresh_expected_repo(
    manifest_dir: &Path,
    case: &Path,
    name: &str,
    changes: &mut Vec<GoldenChange>,
) -> bool {
    let expected = case.join("expected.repo");
    if !expected.exists() {
        return false;
    }
    let actual = manifest_dir.join("target/e2e-work").join(name).join("repo");
    // The same fixture-authoring precondition the compare path asserts: a case
    // that cannot produce a tree is not a case whose golden gets deleted.
    assert!(
        actual.exists(),
        "{name}: expected.repo requires command.args or command.cwd to use {{repo_copy}}"
    );
    let produced = relative_files(&actual);
    let golden = relative_files(&expected);
    // One sorted walk over the union, so the accounting's order is the tree's own
    // discovery order rather than every write followed by every removal
    // (§AR-workspace.9.5).
    for rel in golden.union(&produced) {
        let removed = !produced.contains(rel);
        let moved = if removed {
            remove_tree_golden(&expected, rel, name);
            true
        } else {
            write_tree_golden(&actual.join(rel), &expected.join(rel), name)
        };
        if moved {
            changes.push(GoldenChange {
                surface: "repo tree",
                file: rel.display().to_string(),
                removed,
            });
        }
    }
    true
}

/// Write one tree golden byte-for-byte, and only where the bytes differ from the
/// bytes already there (§AR-workspace.9.1.4) — so a refresh that changed nothing
/// leaves even the mtimes alone. Deliberately not routed through
/// [`write_expected`]: that writer folds `\r\n` and takes a `&str`, while a tree
/// file is read as raw bytes and need not be UTF-8, so a canonicalising write
/// would make the very next compare pass report `bytes differ` on a file it had
/// just written (§AR-workspace.9.1.5).
fn write_tree_golden(produced: &Path, golden: &Path, name: &str) -> bool {
    let bytes = fs::read(produced)
        .unwrap_or_else(|err| panic!("{name}: read {}: {err}", produced.display()));
    if fs::read(golden).is_ok_and(|current| current == bytes) {
        return false;
    }
    if let Some(parent) = golden.parent() {
        fs::create_dir_all(parent)
            .unwrap_or_else(|err| panic!("{name}: create {}: {err}", parent.display()));
    }
    fs::write(golden, &bytes)
        .unwrap_or_else(|err| panic!("{name}: write {}: {err}", golden.display()));
    carry_mode(produced, golden, name);
    true
}

/// Remove a golden file the run no longer produces, and the directories that
/// removal empties — upward until a non-empty one stops the walk, which is
/// exactly `remove_dir`'s own refusal. Git tracks no empty directory, so one
/// left behind is untracked cruft nothing will clean. The `expected.repo` root
/// survives even when it ends up empty: removing it would silently opt the case
/// out of the final-tree surface (§AR-workspace.9.1.5).
fn remove_tree_golden(root: &Path, rel: &Path, name: &str) {
    let path = root.join(rel);
    fs::remove_file(&path).unwrap_or_else(|err| panic!("{name}: remove {}: {err}", path.display()));
    let mut dir = path.parent().map(Path::to_path_buf);
    while let Some(current) = dir.filter(|current| current != root && current.starts_with(root)) {
        if fs::remove_dir(&current).is_err() {
            return;
        }
        dir = current.parent().map(Path::to_path_buf);
    }
}

/// §AR-workspace.9.1.5: the write carries the produced file's mode. It travels
/// with the bytes rather than being pinned on its own, because the compare path
/// reads bytes and never a mode — so this is what preserves the executable bit
/// of a golden the write *creates*, where `fs::write` over an existing file
/// would have kept whatever mode that file already had.
#[cfg(unix)]
fn carry_mode(produced: &Path, golden: &Path, name: &str) {
    let mode = fs::metadata(produced)
        .unwrap_or_else(|err| panic!("{name}: stat {}: {err}", produced.display()))
        .permissions();
    fs::set_permissions(golden, mode)
        .unwrap_or_else(|err| panic!("{name}: set the mode of {}: {err}", golden.display()));
}

/// Where the platform records a file's permissions as an ACL rather than a mode,
/// there is no mode to carry and the golden's own checkout decides.
#[cfg(not(unix))]
fn carry_mode(_produced: &Path, _golden: &Path, _name: &str) {}

/// Render what a refresh pass covered and everything it moved
/// (§AR-workspace.9.5), or `None` where no case was refreshed at all — the
/// determinism passes never consult `UPDATE_EXPECTED` and the synthetic verdict
/// corpus is a comparison run whatever its caller selected
/// (§FS-examples.5.1), so neither accounts for a refresh it did not do.
///
/// Having moved nothing it still says so in as many words: a refresh returns the
/// same verdict for a case whose goldens it rewrote and for one it declined to
/// touch, so silence is the one thing that carries no information.
fn refresh_summary(label: &str, outcomes: &[CaseOutcome]) -> Option<String> {
    let refreshed = outcomes
        .iter()
        .filter_map(|outcome| match outcome {
            CaseOutcome::Refreshed {
                case,
                tree,
                changes,
            } => Some((case, *tree, changes)),
            _ => None,
        })
        .collect::<Vec<_>>();
    if refreshed.is_empty() {
        return None;
    }
    let trees = refreshed.iter().filter(|(_, tree, _)| *tree).count();
    let mut written = 0;
    let mut removed = 0;
    let mut lines = Vec::new();
    for (case, _, changes) in &refreshed {
        for change in changes.iter() {
            let verb = if change.removed {
                removed += 1;
                "removed"
            } else {
                written += 1;
                "wrote"
            };
            lines.push(format!(
                "    {case}: {} — {verb} {}",
                change.surface, change.file
            ));
        }
    }
    let trees_covered = if trees == 0 {
        "and no case it ran carries a final tree".to_string()
    } else {
        format!("and the final tree in the {trees} case(s) that carry one")
    };
    let mut summary = format!(
        "{label}: refreshed {} case(s) — exit, stdout and stderr everywhere, {trees_covered}.\n  \
         {written} file(s) written, {removed} removed",
        refreshed.len()
    );
    if !lines.is_empty() {
        summary.push_str(&format!(":\n{}", lines.join("\n")));
    }
    Some(summary)
}

/// Write one pass-level report where a passing test's reader still gets it
/// (§AR-workspace.9.5). libtest's capture is installed on the `print!` and
/// `eprintln!` macro path and not on the handle, so it discards a passing test's
/// output unless `--nocapture` is passed — and a refresh pass is exactly such a
/// passing test. An accounting the harness swallows is §AR-workspace.9.5's own
/// defect in a new spelling, which is why this goes straight to the handle.
fn report_to_stderr(text: &str) {
    let mut stderr = std::io::stderr();
    let _ = writeln!(stderr, "{text}");
    let _ = stderr.flush();
}
