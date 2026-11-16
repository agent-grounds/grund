//! Every cell of the one unread-name reason, including hidden composition and admitted data
//! (§FS-scan.checks.unread-scan-root.message, §FS-declarations.stubs.target.3).

use super::*;

#[test]
fn unread_name_reasons_cover_versions_extensions_and_data() {
    for (ext, version, data, expected) in [
        (
            Some("txt"),
            1,
            None,
            "extension txt is not in [scan] extensions",
        ),
        (
            Some("txt"),
            2,
            None,
            "no language in [schema.sources] languages reads `.txt`",
        ),
        (
            None,
            1,
            None,
            "it has no extension, and [scan] extensions reads a file only by its extension",
        ),
        (
            None,
            2,
            None,
            "it has no extension, and [schema.sources] languages reads a file only by its extension",
        ),
        (
            Some("json"),
            2,
            Some("json"),
            "the scan reads `.json` as `json`, a data language only a value kind reads",
        ),
        (
            Some("txt"),
            2,
            Some("json"),
            "the scan reads `.txt` as `json`, a data language only a value kind reads",
        ),
        (
            Some("cfg"),
            2,
            Some("cfgdata"),
            "the scan reads `.cfg` as `cfgdata`, a data language only a value kind reads",
        ),
    ] {
        assert_eq!(unread_name_reason(ext, version, data), expected);
        let extension = Refused {
            hidden: false,
            extension: true,
        };
        assert_eq!(unread_reason(extension, ext, version, data), expected);
        let hidden = Refused {
            hidden: true,
            extension: true,
        };
        let tail = if ext.is_none() {
            "it has no extension"
        } else {
            expected
        };
        assert_eq!(
            unread_reason(hidden, ext, version, data),
            format!("its name begins with `.`, and {tail}")
        );
    }
    let hidden = Refused {
        hidden: true,
        extension: false,
    };
    assert_eq!(
        unread_reason(hidden, Some("md"), 2, None),
        "its name begins with `.`, and grund reads no hidden file"
    );
}
