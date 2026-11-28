//! Test module: the report gate compares an `empty-scan` message's extensions in byte order
//! (§FS-config-migrate.5.2), so a suffix a built-in row reads in place of a version-1
//! definition moves no finding (§FS-config-v2.schema.sources.migration).

use super::config_migrate_extensions::extensions_in_byte_order;

fn listed(list: &str) -> String {
    format!("nothing to scan — no files under `x` matched grund's extensions ({list}).")
}

#[test]
fn the_list_is_compared_in_byte_order() {
    assert_eq!(
        extensions_in_byte_order(&listed("md, rs, go")),
        listed("go, md, rs")
    );
    assert_eq!(
        extensions_in_byte_order(&listed("rs, go, md")),
        extensions_in_byte_order(&listed("md, rs, go"))
    );
}

#[test]
fn another_message_is_unchanged() {
    let pruned = "nothing to scan — every file under `m` that grund would read is excluded by \
                  the [schema.sources] exclude pattern `gen`.";
    assert_eq!(extensions_in_byte_order(pruned), pruned);
}
