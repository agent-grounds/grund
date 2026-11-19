/// §FS-config-migrate.6.5.1: the two commands a write closes with — the undo's
/// `git restore <file>` and the re-preview's `grund config migrate --to 2[ PATH]`
/// — spelled for the working directory when available, with the undo keeping
/// discovery's path otherwise (item 1). `--path-base` moves neither (item 4).
/// Each path is already one POSIX shell word (item 3).
struct MigrateCommands {
    /// The config the run replaced, spelled by the core's item 1 rules;
    /// `None` only where discovery no longer finds a config, and then the undo
    /// names no file rather than the report base's.
    restore: Option<String>,
    /// What follows `--to 2`: a space and `PATH` as given, or nothing (item 2),
    /// before the `--drop-comments` the report adds (§FS-config-migrate.6.5).
    path: String,
}

impl MigrateCommands {
    /// §FS-config-migrate.6.5.1 items 1-2: `given` is the `PATH` argument as it
    /// was typed, before it defaults to `.`. The core names the file by the
    /// discovery the run used: relative to the working directory when possible,
    /// absolute across Windows drives, or as discovery reached it when the
    /// process has no working directory. In that last case an absolute `PATH`
    /// gives an absolute undo, while a relative `PATH` may keep a relative undo
    /// that is not portable from a deleted directory. The core never uses the
    /// report path, which is spelled from another root.
    fn for_run(opts: &MigrateOpts, given: Option<&str>) -> Self {
        let dir = std::env::current_dir().ok();
        Self::spelled(opts.config_from(dir.as_deref()).as_deref(), given)
    }

    /// §FS-config-migrate.6.5.1 item 3: the pure spelling, from the config's
    /// spelling supplied by the core and the `PATH` as given, in the core's
    /// shell words, the ones its own printed commands are spelled in.
    fn spelled(restore: Option<&str>, given: Option<&str>) -> Self {
        // `git` reads a word that starts with `-` as an option and one that
        // starts with `:` as pathspec magic; `./` makes either a path.
        let restore = restore.map(|restore| path_word(restore, &['-', ':']));
        // `PATH` keeps the spelling it was given: grund takes none that starts with `-`.
        let path = given.map_or_else(String::new, |given| format!(" {}", path_word(given, &[])));
        Self { restore, path }
    }
}

#[cfg(test)]
#[path = "tests_config_migrate_commands.rs"]
mod tests_config_migrate_commands;
