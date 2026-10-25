//! grund#1010's tree, through the shipped `grund` frontend: a repository `shop`
//! with one nested `[workspace]` in `packages/hub`, under a directory whose own
//! `grund.toml` names it in a `members` list that never closes its bracket.
//! §FS-workspace.6.1.7.5: that value cannot be obtained, so whether the ancestor
//! lists `packages/hub` is undecidable in both directions, and the quiet climb of
//! §FS-check.3.29.2 says nothing and leaves the block unreported — a typo above a
//! repository is not that repository's problem, and must not fail its `check`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

/// What `check` prints in `project/` once nothing reports the block: the issue's
/// closed-bracket run, the one finding the tree earns on its own.
const QUIET: &str = "requirements.md:1: warning: declared but never cited: FS-shop\n";

/// The scratch directory above `project/`, which holds the ancestor config.
struct Outer(PathBuf);

impl Outer {
    /// The issue's tree, with `members` written as the ancestor's whole value.
    fn listing(members: &str) -> Self {
        Self::under(&format!(
            "grund_config_version = 2\nproject_name = \"outer\"\n\n\
             [workspace]\nmembers = {members}\n"
        ))
    }

    /// The issue's tree, under an ancestor config written as `ancestor`.
    fn under(ancestor: &str) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let base = std::env::temp_dir();
        // The climb reaches `/`, so a config above the scratch would answer too.
        let covering = fs::canonicalize(&base).ok().and_then(|walk| {
            walk.ancestors()
                .flat_map(|dir| [dir.join("grund.toml"), dir.join(".agents/grund.toml")])
                .find(|config| config.is_file())
        });
        assert!(
            covering.is_none(),
            "point TMPDIR at a directory no grund config sits above: {covering:?}"
        );
        let outer = Self(base.join(format!(
            "grund-1010-unreadable-ancestor-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )));
        write(&outer.0, "grund.toml", ancestor);
        write(
            &outer.project(),
            "grund.toml",
            "grund_config_version = 2\nproject_name = \"shop\"\n\n\
             [schema.kinds.FS]\nfiles = [\"requirements.md\"]\n",
        );
        write(&outer.project(), "requirements.md", "# FS-shop: Shop\n");
        write(
            &outer.project(),
            "packages/hub/grund.toml",
            "grund_config_version = 2\nproject_name = \"hub\"\n\n\
             [schema.kinds.FS]\nfiles = [\"requirements.md\"]\n\n\
             [workspace]\nmembers = []\n",
        );
        outer
    }

    fn project(&self) -> PathBuf {
        self.0.join("project")
    }

    fn check(&self) -> Output {
        self.check_in(&self.project())
    }

    fn check_in(&self, dir: &Path) -> Output {
        Command::new(env!("CARGO_BIN_EXE_grund"))
            .current_dir(dir)
            .arg("check")
            .output()
            .expect("run grund check")
    }
}

impl Drop for Outer {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn write(root: &Path, relative: &str, body: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, body).unwrap();
}

/// The run's exit code, stdout and stderr, for one assertion message.
fn outcome(output: &Output) -> (Option<i32>, String, String) {
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// §FS-workspace.6.1.7.5, §FS-check.3.29.2: the issue's reproducer. The run goes
/// ahead exactly as it does once the bracket is closed — exit 0, the one warning,
/// and no `unlisted-workspace-block` naming `packages/hub` as listed by nobody.
#[test]
fn an_ancestor_members_list_that_will_not_parse_leaves_the_block_unreported() {
    let unreadable = Outer::listing("[\"project/packages/hub\"");
    assert_eq!(
        outcome(&unreadable.check()),
        (Some(0), QUIET.to_string(), String::new()),
        "an ancestor whose members value cannot be read cannot say nobody lists packages/hub"
    );
}

/// The controls that prove the tree reaches both answers when the entry text is
/// read: the issue's closed bracket lists `packages/hub`, so the run is quiet, and
/// §FS-check.3.29.2's floor — an ancestor that names nothing here confirms the
/// absence, so the block is reported and the run fails as with no ancestor at all.
#[test]
fn a_readable_ancestor_answers_the_claim_both_ways() {
    let closed = Outer::listing("[\"project/packages/hub\"]");
    assert_eq!(
        outcome(&closed.check()),
        (Some(0), QUIET.to_string(), String::new()),
        "an ancestor that lists packages/hub leaves the block unreported"
    );
    let elsewhere = Outer::listing("[\"elsewhere\"]");
    let (code, stdout, _) = outcome(&elsewhere.check());
    assert_eq!(code, Some(1), "{stdout}");
    assert!(
        stdout.starts_with(
            "packages/hub/grund.toml:7: error: this [workspace] is listed by no enclosing workspace"
        ),
        "{stdout}"
    );
}

/// The two spellings of a list the config reader loads and a one-line read does
/// not: written across lines, and ending in the trailing comma TOML allows
/// (§FS-config-v2.reader.5).
const LOADABLE: [&str; 2] = ["[\n  \"{}\",\n]", "[\"{}\",]"];

/// `members` naming `entry` in each [`LOADABLE`] spelling.
fn loadable(entry: &str) -> impl Iterator<Item = String> + '_ {
    LOADABLE
        .into_iter()
        .map(move |shape| shape.replace("{}", entry))
}

/// §FS-workspace.6.1.7.5: a value the config reader loads is a list the climb reads
/// too, so the run's own workspace root written that way confirms that it lists
/// `project` and not `project/packages/hub`, and §FS-check.3.29.2 reports the block
/// as it does under the one-line spelling. Read one line at a time, the root
/// counted as unreadable and silenced every unlisted block below it.
#[test]
fn a_run_root_whose_members_list_the_config_loads_still_reports_the_block() {
    for members in loadable("project") {
        let outer = Outer::listing(&members);
        let (code, stdout, _) = outcome(&outer.check_in(&outer.0));
        assert_eq!(code, Some(1), "{members}: {stdout}");
        assert!(
            stdout.contains(
                "project/packages/hub/grund.toml:7: error: this [workspace] is listed by no \
                 enclosing workspace"
            ),
            "{members}: {stdout}"
        );
    }
}

/// The same spellings above the run's root answer the claim both ways, as the
/// one-line controls do: naming elsewhere confirms the absence and the block is
/// reported (§FS-check.3.29.2), and naming `packages/hub` lists it, so it is not.
#[test]
fn an_ancestor_members_list_the_config_loads_answers_the_claim_both_ways() {
    for members in loadable("elsewhere") {
        let (code, stdout, _) = outcome(&Outer::listing(&members).check());
        assert_eq!(code, Some(1), "{members}: {stdout}");
        assert!(
            stdout.starts_with(
                "packages/hub/grund.toml:7: error: this [workspace] is listed by no enclosing workspace"
            ),
            "{members}: {stdout}"
        );
    }
    for members in loadable("project/packages/hub") {
        assert_eq!(
            outcome(&Outer::listing(&members).check()),
            (Some(0), QUIET.to_string(), String::new()),
            "{members}: an ancestor that lists packages/hub leaves the block unreported"
        );
    }
}

/// §FS-workspace.6.1.7.5: another key's array that never closes holds no claim, so
/// it makes neither value unobtainable, and §FS-check.3.29.2's floor holds. An
/// ancestor that names elsewhere before it, and one that writes no `[workspace]`
/// at all, in either version, still confirm the absence: the block is reported,
/// as it is with that array closed.
#[test]
fn an_unclosed_array_of_another_key_leaves_the_claim_answered() {
    for ancestor in [
        "grund_config_version = 2\nproject_name = \"outer\"\n\n\
         [workspace]\nmembers = [\"elsewhere\"]\n\n[schema.kinds.FS]\nfiles = [\"a\"\n",
        "grund_config_version = 2\nproject_name = \"outer\"\n\n\
         [schema.sources]\nexclude = [\"a\"\n",
        "grund_config_version = 1\n\n[scan]\nexclude = [\"a\"\n",
    ] {
        let (code, stdout, stderr) = outcome(&Outer::under(ancestor).check());
        assert_eq!(
            (code, stderr.as_str()),
            (Some(1), ""),
            "{ancestor}: {stdout}"
        );
        assert!(
            stdout.starts_with(
                "packages/hub/grund.toml:7: error: this [workspace] is listed by no enclosing workspace"
            ),
            "{ancestor}: {stdout}"
        );
    }
}
