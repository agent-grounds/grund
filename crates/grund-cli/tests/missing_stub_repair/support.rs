//! Fixture and assertions for §FS-declarations.stubs.verdict.repair.

use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

pub(super) const CONFIG: &str = "grund_config_version = 2\nproject_name = \"demo\"\n\
    [schema.kinds.AR]\nfolders = [\"docs/architecture\", \"zdocs\"]\nindex = false\n\
    [schema.kinds.code]\nfolders = [\"src\", \"asrc\"]\ncitable = false\n";
pub(super) const DECLARATION: &str = "/// AR-cache: Cache\n///\n/// ## 1. Eviction\n///\n\
    /// Least recently used.\npub struct Cache;\n";
pub(super) const STUB: &str = "docs/architecture/AR-cache.md";
pub(super) const REAL: &str = "src/store/cache.rs";
pub(super) const SAME_FILE: &str = "docs/architecture/cache.md";
pub(super) const MISSING: &str = "stub link target missing: src/cache.rs";
pub(super) const DUPLICATE: &str =
    "duplicate declaration of AR-cache (also declared at src/store/cache.rs:1)";
pub(super) const QUERY_PREFIX: &str =
    "ambiguous ID: AR-cache (declared at docs/architecture/AR-cache.md:1, src/store/cache.rs:1)";
static NEXT: AtomicUsize = AtomicUsize::new(0);

pub(super) struct Repo(pub(super) PathBuf);

impl Repo {
    pub(super) fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "grund-820-repair-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).expect("create isolated fixture");
        let repo = Self(root.canonicalize().unwrap());
        repo.write("grund.toml", CONFIG);
        repo.write(STUB, "# AR-cache: [src/cache.rs](src/cache.rs)\n");
        repo.write(REAL, DECLARATION);
        repo.write("src/main.rs", "// Uses \u{a7}AR-cache\nfn main() {}\n");
        repo
    }

    pub(super) fn write(&self, path: &str, body: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }

    pub(super) fn reverse_sites(&self) {
        fs::remove_file(self.0.join(STUB)).unwrap();
        fs::remove_file(self.0.join(REAL)).unwrap();
        self.write(
            "zdocs/cache.md",
            "# AR-cache: [src/cache.rs](src/cache.rs)\n",
        );
        self.write("asrc/cache.rs", DECLARATION);
    }

    pub(super) fn same_file_home(&self, target: &str) {
        fs::remove_file(self.0.join(STUB)).unwrap();
        fs::remove_file(self.0.join(REAL)).unwrap();
        self.write_same_file_home(target);
    }

    pub(super) fn write_same_file_home(&self, target: &str) {
        self.write(
            SAME_FILE,
            &format!("# AR-cache: [cache]({target})\n\n# AR-cache: Cache\n\nCache body.\n"),
        );
    }

    pub(super) fn run(&self, args: &[&str], exit: i32) -> Output {
        let output = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .env("GIT_CONFIG_GLOBAL", self.0.join("no-global-config"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .output()
            .expect("run this checkout's CLI");
        assert_eq!(output.status.code(), Some(exit), "{args:?}: {output:?}");
        output
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

pub(super) fn utf8(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).unwrap()
}

pub(super) fn records(bytes: &[u8]) -> Vec<Value> {
    utf8(bytes)
        .lines()
        .map(|line| serde_json::from_str(line).expect("finding JSON"))
        .collect()
}

pub(super) fn sites(paths: &[&str]) -> Value {
    sites_at(&paths.iter().map(|path| (*path, 1)).collect::<Vec<_>>())
}

pub(super) fn sites_at(sites: &[(&str, usize)]) -> Value {
    json!(
        sites
            .iter()
            .map(|(path, line)| json!({"path": path, "line": line}))
            .collect::<Vec<_>>()
    )
}

pub(super) fn shape(finding: &Value) {
    let mut keys = finding
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>();
    keys.sort_unstable();
    assert_eq!(
        keys,
        [
            "authority",
            "code",
            "line",
            "message",
            "path",
            "severity",
            "sites"
        ]
    );
    assert_eq!(finding["severity"], "error");
    assert_eq!(finding["authority"], Value::Null);
}

pub(super) fn unique_repair(message: &str, real: &str) {
    assert!(
        message.contains(real),
        "missing real declaration {real}: {message}"
    );
    assert!(
        message.contains("AR-cache is declared at"),
        "missing relationship: {message}"
    );
    assert!(
        message.contains("point the stub"),
        "missing repair instruction: {message}"
    );
}

pub(super) fn located_repair(message: &str, prefix: &str, stub: &str, real: &str) {
    let suffix = message.strip_prefix(prefix).expect("complete old prefix");
    assert!(
        suffix.contains("broken stub"),
        "missing broken-stub explanation: {message}"
    );
    assert!(
        suffix.contains(&format!("{stub}:1")),
        "missing stub site: {message}"
    );
    assert!(
        suffix.contains("src/cache.rs"),
        "missing authored target: {message}"
    );
    unique_repair(suffix, &format!("{real}:1"));
}

pub(super) fn site_free_connection(message: &str, prefix: &str, homes: &[&str]) {
    let suffix = message.strip_prefix(prefix).expect("complete old prefix");
    for home in homes {
        assert!(
            !suffix.contains(home),
            "site-free finding names another home: {message}"
        );
    }
    assert!(
        suffix.contains("duplicate") && suffix.contains("declaration"),
        "missing companion duplicate declaration finding: {message}"
    );
    assert!(
        suffix.contains("AR-cache"),
        "missing ID connection: {message}"
    );
    assert!(
        suffix.contains("real") && suffix.contains("home"),
        "missing real-home connection: {message}"
    );
    assert!(
        !suffix.contains("above") && !suffix.contains("below"),
        "{message}"
    );
    assert_eq!(
        suffix.contains("point the stub"),
        homes.len() == 1,
        "unsafe or missing action: {message}"
    );
}
