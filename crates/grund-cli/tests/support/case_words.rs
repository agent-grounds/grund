// §FS-examples.5.3: the absent-manifest command has one definition.
pub const DEFAULT_COMMAND: &str = "check {repo}";

/// The words a POSIX shell would split a case's `command.args` into, honouring
/// its quotes and nothing else (§FS-examples.5.3): whitespace outside quotes
/// separates arguments, `'…'` and `"…"` keep what they enclose in one argument
/// with the quotes removed, and parts with no whitespace between them join. A
/// `--rule` sentence (§FS-rules.8) is several words by grammar, and
/// `id FS 'Password reset'` a title of two, so a manifest split on whitespace
/// alone could express neither. What a shell would read differently — a
/// backslash outside single quotes, a quote never closed — is refused, naming
/// the case and `command.args` the way `command.cwd` does, rather than run as
/// an argv its author did not write.
fn split_command(name: &str, command: &str) -> Vec<String> {
    let refuse = |what: &str| -> ! { panic!("{name}: command.args: {what}: {:?}", command.trim()) };
    let mut args = Vec::new();
    // `None` until a word starts, so a bare `''` is still one (empty) argument.
    let mut word: Option<String> = None;
    let mut quote: Option<char> = None;
    for character in command.chars() {
        match (quote, character) {
            (Some(open), c) if c == open => quote = None,
            (None | Some('"'), '\\') => refuse(
                "a backslash outside single quotes is not an escape here; \
                 quote the argument instead, in single quotes to keep the backslash",
            ),
            (Some(_), c) => word.get_or_insert_default().push(c),
            (None, '\'' | '"') => {
                quote = Some(character);
                word.get_or_insert_default();
            }
            (None, c) if c.is_whitespace() => args.extend(word.take()),
            (None, c) => word.get_or_insert_default().push(c),
        }
    }
    match quote {
        Some('\'') => refuse("unclosed single quote"),
        Some(_) => refuse("unclosed double quote"),
        None => args.extend(word),
    }
    args
}
