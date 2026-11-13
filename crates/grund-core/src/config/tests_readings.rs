//! Test module: whether a name is read as Markdown, or has its docstrings read, is the
//! configuration's one answer (§FS-fmt.2.3.1, §AR-config.1.6) — by version, for a mapped, an
//! unclaimed and an unadmitted name, and the same for a name written in the config as for a tree
//! file of the same spelling.

use super::*;
use crate::testing::{test_root, write};
use std::path::Path;

fn load(name: &str, toml: &str) -> Config {
    let root = test_root(name);
    write(&root.join("grund.toml"), toml);
    load_config(&root).expect("the control project loads")
}

fn v2(languages: &str, extensions: &str) -> String {
    format!(
        "grund_config_version = 2\nproject_name = \"g1206\"\n[schema.sources]\n\
         languages = [{languages}]\n{extensions}"
    )
}

/// The reading of `name` as the config writes it, and of a tree file spelled the same.
fn both(config: &Config, name: &str) -> (Readings, Readings) {
    let written = Readings::of_written(&config.schema().sources, Path::new(name));
    let file = config.root.join("docs").join(name);
    let reading = config.reading(&file);
    let tree = Readings {
        markdown: reading.markdown,
        docstrings: reading.python,
    };
    (written, tree)
}

/// §FS-fmt.2.3.1: version 2 reads a name by the language that claims it, and a name no admitted
/// language claims is neither Markdown nor has docstrings, whatever its suffix.
#[test]
fn version_2_reads_by_the_language() {
    let mapped = load(
        "readings_v2_mapped",
        &v2(
            "\"markdown\", \"rust\", \"python\"",
            "[schema.sources.extensions]\ntxt = \"markdown\"\nmd = \"rust\"\n",
        ),
    );
    for (name, markdown, docstrings) in [
        ("a.txt", true, false),
        ("a.md", false, false),
        ("a.py", false, true),
    ] {
        let (written, tree) = both(&mapped, name);
        assert_eq!(
            written,
            Readings {
                markdown,
                docstrings
            },
            "mapped {name}"
        );
        assert_eq!(
            written, tree,
            "mapped {name}: the config and the tree agree"
        );
    }
    let unadmitted = load("readings_v2_unadmitted", &v2("\"rust\"", ""));
    for name in ["a.md", "a.py", "a.unclaimed"] {
        let (written, tree) = both(&unadmitted, name);
        let none = Readings {
            markdown: false,
            docstrings: false,
        };
        assert_eq!(written, none, "unadmitted {name}");
        assert_eq!(
            written, tree,
            "unadmitted {name}: the config and the tree agree"
        );
    }
}

/// §FS-fmt.2.3.1, §FS-config.5.2: version 1's reading is its name, `.md` Markdown and `.py`
/// docstrings, and a name it does not claim is neither.
#[test]
fn version_1_reads_by_the_name() {
    let config = load(
        "readings_v1",
        "grund_config_version = 1\n[scan]\ninclude = [\"docs\"]\nextensions = [\"md\", \"py\", \"txt\"]\n",
    );
    for (name, markdown, docstrings) in [
        ("a.md", true, false),
        ("a.py", false, true),
        ("a.txt", false, false),
    ] {
        let (written, tree) = both(&config, name);
        assert_eq!(
            written,
            Readings {
                markdown,
                docstrings
            },
            "v1 {name}"
        );
        assert_eq!(written, tree, "v1 {name}: the config and the tree agree");
    }
}
