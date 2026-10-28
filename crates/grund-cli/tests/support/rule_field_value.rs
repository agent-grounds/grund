//! Test-only repositories for the whose-clause tests: the #1047 fixture and the repository
//! the refusal tables of §FS-rule-refusals.sentences.whose are read in (§FS-rules.2.2).

#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The issue's config: `SWR.verification` is a scalar, and no `TC` is written yet.
pub const CONFIG: &str = "grund_config_version = 2
project_name = \"swr\"

[schema.kinds.SWR]
files = [\"req/SWR-brake-latency.md\", \"req/SWR-fault-log.md\"]
[schema.kinds.SWR.fields.verification]
one_of = [\"Test\", \"Analysis\", \"Inspection\", \"Demonstration\"]
presence = \"must\"
position = \"first\"

[schema.kinds.TC]
files = [\"tc/TC-placeholder.md\"]

[schema.kinds.AN]
files = [\"an/AN-brake-latency.md\"]
";

pub const BRAKE: &str = "# SWR-brake-latency: The brake command reaches the actuator within 10 ms

## verification: Test

Measured on the bench with the actuator under load.
";

pub const FAULT: &str = "# SWR-fault-log: Every actuator fault is written to the fault log

## verification: Inspection

Checked by reading the logging code.
";

pub const ANALYSIS: &str = "# AN-brake-latency: Worst-case latency analysis

The scheduling analysis bounds §SWR-brake-latency at 7 ms. It also covers §SWR-fault-log.
";

/// The repository the refusal tables of §FS-rule-refusals.sentences.whose are read in: `SWR`
/// adds the prose field `rationale`, `AN` declares the scalar `status`, `TC` declares none.
pub const TABLE_CONFIG: &str = "grund_config_version = 2
project_name = \"swr\"

[schema.kinds.SWR]
files = [\"req/SWR-brake-latency.md\", \"req/SWR-fault-log.md\"]
[schema.kinds.SWR.fields.verification]
one_of = [\"Test\", \"Analysis\", \"Inspection\", \"Demonstration\"]
presence = \"must\"
position = \"first\"
[schema.kinds.SWR.fields.rationale]
presence = \"may\"

[schema.kinds.TC]
files = [\"tc/TC-placeholder.md\"]

[schema.kinds.AN]
files = [\"an/AN-brake-latency.md\"]
[schema.kinds.AN.fields.status]
one_of = [\"Draft\", \"Must fix\", \"fix and retest\", \"fix must wait\"]
presence = \"may\"
";

pub const RULE: &str = "Each SWR whose verification is Test must be cited by at least one TC.";

pub const BRAKE_UNCITED: &str = "req/SWR-brake-latency.md:1: error: SWR-brake-latency is cited by TC \
                             0 times; --rule requires at least one; add or remove the citations \
                             of it until the count fits\n";

pub const KNOWN: &str = "known kinds: SWR, TC, AN\n";

pub struct Fixture(pub PathBuf);

pub struct Run {
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

impl Fixture {
    pub fn with(files: &[(&str, &str)]) -> Self {
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("grund-1047-{}-{serial}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        for (path, contents) in files {
            let path = dir.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, contents).unwrap();
        }
        Self(dir)
    }

    /// The issue's repository, with `config` in place of its `grund.toml`.
    pub fn issue(config: &str) -> Self {
        Self::with(&[
            ("grund.toml", config),
            ("req/SWR-brake-latency.md", BRAKE),
            ("req/SWR-fault-log.md", FAULT),
            ("an/AN-brake-latency.md", ANALYSIS),
        ])
    }

    pub fn table() -> Self {
        Self::with(&[
            ("grund.toml", TABLE_CONFIG),
            ("req/SWR-brake-latency.md", BRAKE),
            ("req/SWR-fault-log.md", FAULT),
            (
                "an/AN-brake-latency.md",
                "# AN-brake-latency: Worst-case latency analysis\n\n## status: Must fix\n\n\
                 Bounds §SWR-brake-latency and covers §SWR-fault-log.\n",
            ),
        ])
    }

    pub fn run(&self, args: &[&str]) -> Run {
        let out = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .output()
            .expect("run grund");
        Run {
            code: out.status.code(),
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        }
    }

    /// Run `grund args…`, require exit `code`, and return the run.
    pub fn expect(&self, code: i32, args: &[&str]) -> Run {
        let run = self.run(args);
        assert_eq!(
            run.code,
            Some(code),
            "grund {} exited {:?}: stdout:\n{}\nstderr:\n{}",
            args.join(" "),
            run.code,
            run.stdout,
            run.stderr
        );
        run
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
