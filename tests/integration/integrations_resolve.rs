//! §FS-integrations.4 — the headless half of `scripts/try-integrations.sh`,
//! run as a gate: install the resolver into a sandbox HOME from the binary
//! under test and resolve every citation form the clients hand it against this
//! repository — plain, sectioned, bare, punctuation-swept, workspace-qualified,
//! unknown, and `path:line` locations (§FS-integrations.3.1). The script is a
//! manual testbed for the clickable clients; its resolver checks need no
//! terminal, so nothing excuses them from CI. Unix only: the script is `bash`
//! and the resolver it installs is the Unix shell integration.
#![cfg(unix)]

#[path = "binaries.rs"]
mod binaries;

use std::fs;
use std::process::Command;

/// §FS-show.3.1: installed resolvers trust this terminal pair for declarations
/// and sections because authored body prose may contain location-shaped JSON.
#[test]
fn titled_show_json_keeps_path_and_line_as_the_terminal_pair() {
    let repo = binaries::repo_root();
    let sandbox = repo.join("target/integration-work/show-json-location-tail");
    let _ = fs::remove_dir_all(&sandbox);
    fs::create_dir_all(sandbox.join("docs")).expect("create show JSON fixture");
    fs::write(
        sandbox.join("grund.toml"),
        concat!(
            "grund_config_version = 1\n",
            "[id]\nformat = \"{kind}-{slug}\"\n",
            "[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\n",
            "index = false\ntitle = \"Product contracts\"\n",
        ),
    )
    .expect("write show JSON config");
    fs::write(
        sandbox.join("docs/FS-authored.md"),
        "# FS-authored: Authored\n\nLead.\n\n## 1. Detail\n\nDetail body.\n",
    )
    .expect("write show JSON declaration");

    for (query, expected) in [
        (
            "FS-authored",
            "{\"id\":\"FS-authored\",\"section\":null,\"body\":\"Lead.\\n\",\"kind_title\":\"Product contracts\",\"path\":\"docs/FS-authored.md\",\"line\":1}\n",
        ),
        (
            "FS-authored.1",
            "{\"id\":\"FS-authored\",\"section\":\"1\",\"body\":\"## 1. Detail\\n\\nDetail body.\\n\",\"kind_title\":\"Product contracts\",\"path\":\"docs/FS-authored.md\",\"line\":5}\n",
        ),
    ] {
        let output = Command::new(binaries::grund())
            .args([query, "--format=json"])
            .current_dir(&sandbox)
            .output()
            .expect("run titled show JSON query");
        assert!(
            output.status.success(),
            "{query} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
        assert!(output.stderr.is_empty());
    }

    fs::remove_dir_all(&sandbox).expect("remove show JSON fixture");
}

#[cfg(unix)]
#[test]
fn the_resolver_resolves_every_citation_form_headlessly() {
    let repo = binaries::repo_root();
    let sandbox = repo.join("target/integration-work/integrations-sandbox");
    let _ = fs::remove_dir_all(&sandbox);
    let output = Command::new("bash")
        .arg(repo.join("scripts/try-integrations.sh"))
        .arg("resolve")
        .arg("--binary")
        .arg(binaries::grund())
        .arg("--sandbox")
        .arg(&sandbox)
        .current_dir(&repo)
        .output()
        .expect("run scripts/try-integrations.sh");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "try-integrations.sh resolve exited with {}\nstdout:\n{stdout}\nstderr:\n{stderr}",
        output.status
    );
    let failed = stdout
        .lines()
        .filter(|line| line.trim_start().starts_with("FAIL"))
        .collect::<Vec<_>>();
    assert!(
        failed.is_empty(),
        "resolver checks failed:\n{}\n\nfull output:\n{stdout}",
        failed.join("\n")
    );
    let passed = stdout
        .lines()
        .filter(|line| line.trim_start().starts_with("PASS"))
        .count();
    assert!(
        passed >= 8,
        "expected the resolver checks to run; saw {passed} PASS line(s):\n{stdout}"
    );
}
