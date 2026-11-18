/// §FS-config-migrate.6.5.1: the two commands a write closes with — the undo's
/// `git restore <file>` and the re-preview's `grund config migrate --to 2[ PATH]`
/// — spelled for the directory `grund config migrate` was run in rather than
/// from the report base, so `--path-base` moves neither (item 4). Each path is
/// already one POSIX shell word (item 3).
struct MigrateCommands {
    /// The config the run replaced, as `git restore` takes it there (item 1).
    restore: String,
    /// What follows `--to 2`: a space and `PATH` as given, or nothing (item 2).
    path: String,
}

impl MigrateCommands {
    /// §FS-config-migrate.6.5.1 items 1-2: `given` is the `PATH` argument as it
    /// was typed, before it defaults to `.`. The core names the file by the
    /// discovery the run used; without a working directory to spell it from,
    /// the report's own spelling is the one left.
    fn for_run(opts: &MigrateOpts, given: Option<&str>, report: &MigrateReport) -> Self {
        let restore = std::env::current_dir()
            .ok()
            .and_then(|dir| opts.config_from(&dir))
            .unwrap_or_else(|| report.config.clone());
        Self::spelled(&restore, given)
    }

    /// §FS-config-migrate.6.5.1 item 3: the pure spelling, from the config's
    /// path relative to the working directory and the `PATH` as given.
    fn spelled(restore: &str, given: Option<&str>) -> Self {
        // `git` reads a word that starts with `-` as an option.
        let restore = if restore.starts_with('-') {
            shell_word(&format!("./{restore}")).into_owned()
        } else {
            shell_word(restore).into_owned()
        };
        let path = given.map_or_else(String::new, |given| format!(" {}", shell_word(given)));
        Self { restore, path }
    }
}

/// §FS-config-migrate.6.5.1 item 3: `text` as one POSIX shell word — as it is
/// when every character is an ASCII letter, a digit or one of `_ - . / , : @ % + =`,
/// otherwise between single quotes with each `'` written `'\''`. The empty
/// word is quoted too, so it is still a word.
fn shell_word(text: &str) -> std::borrow::Cow<'_, str> {
    let safe = |c: char| c.is_ascii_alphanumeric() || "_-./,:@%+=".contains(c);
    if !text.is_empty() && text.chars().all(safe) {
        std::borrow::Cow::Borrowed(text)
    } else {
        std::borrow::Cow::Owned(format!("'{}'", text.replace('\'', r"'\''")))
    }
}

#[cfg(test)]
#[path = "tests_config_migrate_commands.rs"]
mod tests_config_migrate_commands;
