//! How a command for the user to paste spells a path (§FS-config-migrate.6.5.1).
//! A report path is spelled from the report base (§FS-cli.3.4); a command is
//! pasted where grund ran, so each path in it is spelled from the working
//! directory, with the links on both sides resolved, and written as one POSIX
//! shell word. Every command a migrate report prints reads the spelling here —
//! the undo, the `strict = false` repair and a `member:` line's command — and so
//! does the member-root hint of §FS-workspace.5.1.1, so the four cannot drift.

use std::borrow::Cow;
use std::path::{Path, PathBuf};

use crate::model::{format_path, relative_from_base};
use crate::sources::canonical_path;

/// §FS-config-migrate.6.5.1 item 1: `target` spelled from the working directory
/// `dir`, for a command pasted there rather than from the report base. Both are
/// resolved as canonical identity resolves them (§FS-config.1.5) — `dir` whole,
/// `target` up to its last component unless it is a directory — so a link on
/// either side adds no `..`, while a config that is a link stays the file git
/// tracks. The spelling is `/`-separated, climbs with `..` to a target above
/// `dir`, is absolute where no relative spelling exists, as across Windows
/// drives, and is empty where `target` is `dir`. With `dir=None`, for a process
/// with no working directory, `target` is formatted with `/` as the caller
/// reached it: for migrate, as discovery reached it from `PATH`.
pub(crate) fn command_path(dir: Option<&Path>, target: &Path) -> String {
    let Some(dir) = dir else {
        return format_path(target);
    };
    let dir = canonical_path(dir).unwrap_or_else(|_| dir.to_path_buf());
    format_path(&relative_from_base(&dir, &resolved(target)))
}

/// `path` with its directory links resolved: a directory whole, anything else
/// through its parent, and as written where that does not resolve.
fn resolved(path: &Path) -> PathBuf {
    if path.is_dir() {
        return canonical_path(path).unwrap_or_else(|_| path.to_path_buf());
    }
    match (path.parent(), path.file_name()) {
        (Some(parent), Some(name)) if !parent.as_os_str().is_empty() => {
            canonical_path(parent).map_or_else(|_| path.to_path_buf(), |parent| parent.join(name))
        }
        _ => path.to_path_buf(),
    }
}

/// §FS-config-migrate.6.5.1 items 1 and 3: the directory `target` as the
/// argument of a command pasted in `dir` — `None` where it is that directory,
/// so the command names none, and otherwise [`command_path`]'s spelling as one
/// shell word, with `./` before a leading `-`, since grund takes no `PATH` that
/// starts with one. The `strict = false` repair names the refused project's
/// root this way (§FS-config-migrate.3.1.2), and a `member:` line the member's
/// (§FS-config-migrate.8).
pub(crate) fn command_directory(dir: Option<&Path>, target: &Path) -> Option<String> {
    let spelled = command_path(dir, target);
    (!spelled.is_empty()).then(|| path_word(&spelled, &['-']))
}

/// §FS-config-migrate.6.5.1 item 3: the spelled `path` as one shell word the
/// command reads as a path, written with `./` before it where it would start
/// with one of `leading`: `-` for any command, which would read an option, and
/// `:` too for `git`, which would read pathspec magic. With no `leading`, the
/// path is [`shell_word`] alone, as the re-preview writes `PATH` as given. A
/// frontend's seam for the undo and the re-preview it spells, hidden with
/// `config_migrate` (§FS-distribution.3.1.2).
#[doc(hidden)]
pub fn path_word(path: &str, leading: &[char]) -> String {
    if path.starts_with(leading) {
        shell_word(&format!("./{path}")).into_owned()
    } else {
        shell_word(path).into_owned()
    }
}

/// §FS-config-migrate.6.5.1 item 3: `text` as one POSIX shell word — as it is
/// when every character is an ASCII letter, a digit or one of `_ - . / , : @ % + =`,
/// otherwise between single quotes with each `'` written `'\''`. The empty
/// word is quoted too, so it is still a word.
pub(crate) fn shell_word(text: &str) -> Cow<'_, str> {
    let safe = |c: char| c.is_ascii_alphanumeric() || "_-./,:@%+=".contains(c);
    if !text.is_empty() && text.chars().all(safe) {
        Cow::Borrowed(text)
    } else {
        Cow::Owned(format!("'{}'", text.replace('\'', r"'\''")))
    }
}

#[cfg(test)]
#[path = "tests_command_paths.rs"]
mod tests;
