//! §FS-config-migrate.5.2: the extensions an `empty-scan` message lists, in byte order on
//! both runs. Their order follows how each suffix is mapped, which a built-in row named in
//! place of a version-1 definition moves (§FS-config-v2.schema.sources.migration), not
//! what is read, so the report gate compares the list as the set it is.

/// `message` with the list after `matched grund's extensions (` sorted; any other message
/// is returned unchanged (§FS-check.2.2.2).
pub(super) fn extensions_in_byte_order(message: &str) -> String {
    const OPEN: &str = "matched grund's extensions (";
    let Some(start) = message.find(OPEN).map(|at| at + OPEN.len()) else {
        return message.to_string();
    };
    let Some(len) = message[start..].find(')') else {
        return message.to_string();
    };
    let mut list: Vec<&str> = message[start..start + len].split(", ").collect();
    list.sort_unstable();
    format!(
        "{}{}{}",
        &message[..start],
        list.join(", "),
        &message[start + len..]
    )
}
