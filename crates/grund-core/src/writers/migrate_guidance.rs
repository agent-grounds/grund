//! §FS-config-migrate.5.3: what `grund init` would write into the root
//! `AGENTS.md` for one config, rendered in memory and reduced to what the
//! guidance gate refuses a change to: the block version and the five generated
//! sections. Migration renders it for the v1 file and for the candidate and
//! refuses on a difference; the rest of the block describes the config, so it
//! passes, and `check` reports it until `grund init` runs. It goes through the renderers `init` writes with and `Expected`
//! carries, so the three cannot disagree (§AR-checker.2.7).

use super::guidance::{Guidance, RuleGuidance};
use crate::checker::{configured_rule_sentences, declared_workspace_vocabulary};
use crate::config::Config;
use crate::model::ExpectedSection;
use crate::scanner::{CANONICAL_AGENT_ENTRYPOINT, scan_tree};
use crate::workspace::{declared_member_schemas, populate_workspace_boundary};

/// The root block `init` would write, reduced to its version and its five
/// generated sections (§FS-config-migrate.5.3).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RootGuidance {
    pub(crate) version: u32,
    pub(crate) sections: Vec<ExpectedSection>,
}

/// §FS-config-migrate.5.3: the root `AGENTS.md` block of `config`, with the
/// chapter rules read from the tree as `init` reads them, or none where the
/// scan or a rule fails, which `check` reports on its own.
pub(crate) fn root_guidance(mut config: Config) -> RootGuidance {
    let _ = populate_workspace_boundary(&mut config);
    let config = &config;
    let scanned = config
        .kinds
        .iter()
        .any(|kind| kind.rules)
        .then(|| scan_tree(config.schema(), config.frame(), Some(&config.root), true).ok())
        .flatten()
        .map(|(findings, _)| findings);
    let rules = scanned.as_ref().and_then(|findings| {
        let vocab = declared_workspace_vocabulary(
            config.schema(),
            config.rules(),
            config.frame(),
            &declared_member_schemas(config),
        );
        configured_rule_sentences(findings, config.schema(), config.frame(), &vocab).ok()
    });
    let guidance = Guidance::new(
        config,
        scanned
            .as_ref()
            .zip(rules.as_ref())
            .map(|(findings, rules)| RuleGuidance {
                findings,
                rows: &rules.rows,
                subjects: &rules.subjects,
            }),
    );
    RootGuidance {
        version: guidance.version,
        sections: guidance.generated_sections(&config.root.join(CANONICAL_AGENT_ENTRYPOINT)),
    }
}
