//! Black-box: a CRLF file's front matter, a `---` (YAML) or a `+++` (TOML) block, reads as its LF
//! form does, when its kind declares the `front-matter` field (§FS-fields.front-matter.reading,
//! §FS-fields.front-matter.constructs). The pin lives here, not under `tests/e2e/cases/`, because
//! `.gitattributes` checks every case fixture out with LF endings.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const CONFIG: &str = "grund_config_version = 2
project_name = \"madr\"

[schema.kinds.ADR]
folders = [\"docs/adr\"]
index = false

[schema.kinds.ADR.fields.front-matter]
presence = \"must\"
closed = \"must\"

[schema.kinds.ADR.fields.front-matter.keys.status]
presence = \"must\"
one_of = [\"proposed\", \"accepted\"]

[schema.kinds.ADR.fields.front-matter.keys.date]
presence = \"must\"

[schema.kinds.page]
citable = false
files = [\"README.md\"]
";

/// A misspelt status and a stray key, so both a value and a key's line are judged.
const RECORD: &str = "---\nstatus: Accepted\ndate: 2026-02-03\nreviewer: Ana Ruiz\n---\n\n\
                      # ADR-cache: Cache availability\n\nText.\n";

/// The same record in TOML, with a `[links]` table whose slice runs over two lines.
const TOML_RECORD: &str = "+++\nstatus = \"Accepted\"\ndate = 2026-02-03\nreviewer = \"Ana Ruiz\"\n\n\
                           [links]\nnext = \"ADR-seed\"\n+++\n\n\
                           # ADR-cache: Cache availability\n\nText.\n";

struct Fixture(PathBuf);

impl Fixture {
    fn new(record: &str) -> Self {
        static SERIAL: AtomicUsize = AtomicUsize::new(0);
        let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("grund-1048-{}-{serial}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        for (path, contents) in [
            ("grund.toml", CONFIG),
            ("README.md", "# Readme\n\nThe decision is §ADR-cache.\n"),
            ("docs/adr/cache.md", record),
        ] {
            let path = dir.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, contents).unwrap();
        }
        Self(dir)
    }

    /// `grund args…` in the fixture: its exit code and stdout.
    fn run(&self, args: &[&str]) -> (Option<i32>, String) {
        let out = Command::new(env!("CARGO_BIN_EXE_grund"))
            .args(args)
            .current_dir(&self.0)
            .output()
            .expect("run grund");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.is_empty(),
            "grund {}: stderr:\n{stderr}",
            args.join(" ")
        );
        (
            out.status.code(),
            String::from_utf8_lossy(&out.stdout).into_owned(),
        )
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// The same findings at the same lines, and no slice ends in a carriage return.
#[test]
fn a_crlf_block_reads_as_its_lf_form() {
    let lf = Fixture::new(RECORD);
    let crlf = Fixture::new(&RECORD.replace('\n', "\r\n"));
    let expected = "docs/adr/cache.md:2: error: ADR-cache.front-matter.status is \"Accepted\"; \
                    kind ADR admits proposed, accepted; change the value to one of them\n\
                    docs/adr/cache.md:4: error: ADR-cache.front-matter.reviewer is not a key of \
                    kind ADR's front matter, which is closed; its keys: status, date; rename it \
                    to one of them, remove it, or add it to the field's keys\n";
    assert_eq!(lf.run(&["check"]), (Some(1), expected.to_owned()));
    assert_eq!(crlf.run(&["check"]), (Some(1), expected.to_owned()));
    for key in [
        "ADR-cache.front-matter.status",
        "ADR-cache.front-matter.date",
    ] {
        let (code, slice) = crlf.run(&[key]);
        assert_eq!(code, Some(0), "grund {key}");
        assert!(!slice.contains('\r'), "grund {key} printed {slice:?}");
        assert_eq!((code, slice), lf.run(&[key]), "grund {key}");
    }
}

/// A `+++` block too: the same findings at the same lines, no slice ends in a carriage return,
/// and a table's slice is its LF form with the file's own line endings inside it.
#[test]
fn a_crlf_toml_block_reads_as_its_lf_form() {
    let lf = Fixture::new(TOML_RECORD);
    let crlf = Fixture::new(&TOML_RECORD.replace('\n', "\r\n"));
    let expected = "docs/adr/cache.md:2: error: ADR-cache.front-matter.status is \"Accepted\"; \
                    kind ADR admits proposed, accepted; change the value to one of them\n\
                    docs/adr/cache.md:4: error: ADR-cache.front-matter.reviewer is not a key of \
                    kind ADR's front matter, which is closed; its keys: status, date; rename it \
                    to one of them, remove it, or add it to the field's keys\n\
                    docs/adr/cache.md:6: error: ADR-cache.front-matter.links is not a key of \
                    kind ADR's front matter, which is closed; its keys: status, date; rename it \
                    to one of them, remove it, or add it to the field's keys\n";
    assert_eq!(lf.run(&["check"]), (Some(1), expected.to_owned()));
    assert_eq!(crlf.run(&["check"]), (Some(1), expected.to_owned()));
    for key in [
        "ADR-cache.front-matter.status",
        "ADR-cache.front-matter.date",
    ] {
        let (code, slice) = crlf.run(&[key]);
        assert_eq!(code, Some(0), "grund {key}");
        assert!(!slice.contains('\r'), "grund {key} printed {slice:?}");
        assert_eq!((code, slice), lf.run(&[key]), "grund {key}");
    }
    let key = "ADR-cache.front-matter.links";
    let (code, slice) = crlf.run(&[key]);
    assert_eq!(code, Some(0), "grund {key}");
    assert!(
        !slice.trim_end_matches('\n').ends_with('\r'),
        "grund {key} printed {slice:?}"
    );
    assert_eq!(
        (code, slice.replace("\r\n", "\n")),
        lf.run(&[key]),
        "grund {key}"
    );
}
