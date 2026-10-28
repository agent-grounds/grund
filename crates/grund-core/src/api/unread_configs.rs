//! The project configs below a run that loaded none, named the one way both
//! messages that read them name them (§FS-check.2.2.2.5.1): the empty-scan
//! warning (§FS-check.2.2.2.5) and the invalid-ID hint (§FS-show.3.5.1). One
//! search answers both, so a check and a query over one tree cannot name
//! different projects (§AR-system.principle).

use super::scope_cautions::{display_scope, quoted_list};
use crate::config::{Config, Display};
use crate::scanner::project_configs_below;

/// §FS-check.2.2.2.5.1: how many configs are named before the rest are counted.
const NAMED: usize = 3;

/// The configs a run that loaded none did not read: the first [`NAMED`] of them
/// spelled from the report base, each file beside the directory it governs, and
/// how many the search found in all.
pub(super) struct UnreadConfigs {
    configs: Vec<String>,
    dirs: Vec<String>,
    total: usize,
}

impl UnreadConfigs {
    /// §FS-check.2.2.2.5.2: the configs below `config`'s root, or `None` — at once,
    /// without walking, for a run that loaded a config, and after the search for a
    /// run that finds none. Paths are spelled as `display` spells every other
    /// path of the run (§FS-workspace.5.3).
    pub(super) fn below(config: &Config, display: Display<'_>) -> Option<Self> {
        if !config.run().found_no_config {
            return None;
        }
        let found = project_configs_below(config.schema(), config.frame());
        let (dirs, configs) = found
            .iter()
            .take(NAMED)
            .map(|(dir, file)| (display_scope(display, dir), display_scope(display, file)))
            .unzip();
        (!found.is_empty()).then_some(Self {
            configs,
            dirs,
            total: found.len(),
        })
    }

    /// §FS-check.2.2.2.5.1: `` `suite/grund.toml` declares a project this run does
    /// not read ``, the names joined in path order with `and N more` past three.
    pub(super) fn names(&self) -> String {
        let mut names: Vec<String> = self.configs.iter().map(|c| format!("`{c}`")).collect();
        if self.total > NAMED {
            names.push(format!("{} more", self.total - NAMED));
        }
        let last = names.pop().unwrap_or_default();
        let names = if names.is_empty() {
            last
        } else {
            format!("{} and {last}", names.join(", "))
        };
        match self.total {
            1 => format!("{names} declares a project this run does not read"),
            _ => format!("{names} declare projects this run does not read"),
        }
    }

    /// §FS-check.2.2.2.5: the routes that read them, for `grund check`, which also
    /// takes the first directory as its path.
    pub(super) fn check_routes(&self) -> String {
        self.routes(&format!(" (or `grund check {}`)", self.dirs[0]))
    }

    /// §FS-show.3.5.1: the routes a query is offered, which no `grund check` answers.
    pub(super) fn query_routes(&self) -> String {
        self.routes("")
    }

    /// §FS-check.2.2.2.5.1: running from a config's directory, then a root
    /// `[workspace]` whose `members` lists every directory when every config is
    /// named, since a list of the three named would leave the rest unread.
    fn routes(&self, check: &str) -> String {
        let from = match self.dirs.as_slice() {
            [dir] => format!("`{dir}/`"),
            _ => "one of their directories".to_owned(),
        };
        let workspace = if self.total > NAMED {
            "lists their directories in members and has include_root = false".to_owned()
        } else {
            format!(
                "has members = [{}] and include_root = false",
                quoted_list(&self.dirs)
            )
        };
        format!(
            "run grund from {from}{check}, or add a grund.toml here whose [workspace] {workspace}"
        )
    }
}
