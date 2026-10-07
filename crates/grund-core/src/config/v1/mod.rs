//! The version-1 reader (§AR-config.3): what a `grund.toml` that omits
//! `grund_config_version`, or sets it to `1`, means as a `Project`. The
//! defaults it lowers over are `defaults.rs`, applied here and nowhere else
//! (§AR-config.2 rule 2).

mod defaults;

pub(super) use defaults::default_project;
