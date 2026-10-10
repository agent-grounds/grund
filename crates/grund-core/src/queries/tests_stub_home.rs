//! Test module: a stub whose target lies outside scan scope reads the target's
//! declaration by its ID, and reads it from an editor's overlay where there is
//! one, as the scanned target would (§FS-show.2.3.7). The e2e cases pin the
//! disk read; an overlay reaches `show` only through the engine.

use super::*;
use crate::config::load_config;
use crate::model::{Id, ShowRenderMode, TextOverlays};
use crate::testing::{scan_tree, test_root, write};

const CONFIG: &str = "grund_config_version = 1\n\n[reference]\nstrict = true\n\n\
[id]\nformat = \"{kind}-{slug}\"\n\n[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\n\
index = false\n\n[scan]\ninclude = [\"docs\"]\n";

/// On disk, `FS-second` is declared on line 6, its stub on line 1.
const ON_DISK: &str = concat!(
    "/// FS-first: First\n",
    "///\n",
    "/// First lead.\n",
    "pub fn first() {}\n",
    "\n",
    "/// FS-second: Second\n",
    "///\n",
    "/// Second lead.\n",
    "pub fn second() {}\n",
);

/// The unsaved edit moves `FS-second` to line 3 and gives it a section on line 7.
const UNSAVED: &str = concat!(
    "// An unsaved edit.\n",
    "\n",
    "/// FS-second: Second\n",
    "///\n",
    "/// Edited lead.\n",
    "///\n",
    "/// ## 1. Edited detail\n",
    "///\n",
    "/// Edited detail.\n",
    "pub fn second() {}\n",
);

fn second() -> Id {
    Id {
        kind: "FS".to_string(),
        num: None,
        slug: Some("second".to_string()),
    }
}

#[test]
fn unscanned_stub_target_reads_the_overlay_by_its_id() {
    let root = test_root("stub_home_unscanned_overlay");
    write(&root.join("grund.toml"), CONFIG);
    write(
        &root.join("docs/second.md"),
        "# FS-second: [../source.rs](../source.rs)\n",
    );
    write(&root.join("source.rs"), ON_DISK);
    let config = load_config(&root).expect("load config");
    let (findings, _) = scan_tree(&config, None, false).expect("scan");
    let overlays = TextOverlays::from([(config.root.join("source.rs"), UNSAVED.to_string())]);
    let show = |section| {
        show_declaration_with_overlays(
            &config,
            &config,
            &findings,
            &second(),
            section,
            ShowRenderMode::Default,
            false,
            &overlays,
        )
        .expect("§FS-show.2.3.7: a stub that is not broken reads its target")
        .0
    };

    let lead = show(None);
    assert_eq!((lead.body.as_str(), lead.line), ("Edited lead.\n", 3));
    let detail = show(Some("1"));
    assert_eq!(
        (detail.body.as_str(), detail.line),
        ("## 1. Edited detail\n\nEdited detail.\n", 7)
    );
}
