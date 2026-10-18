//! User preferences, agent gates and managed ownership (§FS-distribution.3.3.6).

use super::integrations_agents::{
    ConversationRendering, GLOBAL_AGENT_INSTRUCTION_TARGETS, agent_override_table,
};
use super::integrations_api::{Fault, event, expand, outcome, save};
use super::integrations_install::{BlockOutcome, install_agent_guidance_block, merge_outcomes};
use super::integrations_output::{
    IntegrationEvent, IntegrationNote, IntegrationOutcome, IntegrationsOpts,
};
use super::integrations_user_config::{
    USER_CONFIG_TARGET, UserConfigScan, install_reference_key, read_optional_text,
    scan_user_config, user_grund_config_path,
};
use crate::model::{Finding, format_path};
use std::path::{Path, PathBuf};

pub(super) struct Guidance {
    path: PathBuf,
    text: String,
    scan: UserConfigScan,
    pub(super) cautions: Vec<Finding>,
}

/// Read the user configuration once, warning about every line grund did not
/// act on: only failing to reach the file is a failure (§FS-integrations.4.3.5).
pub(super) fn load() -> Result<Guidance, Fault> {
    let path = user_grund_config_path().ok_or_else(|| {
        Fault::at(
            Path::new(USER_CONFIG_TARGET),
            "cannot resolve user configuration directory",
        )
    })?;
    let text = read_optional_text(&path)?;
    let scan = scan_user_config(&text);
    let cautions = scan
        .problems
        .iter()
        .map(|(line, message)| Finding {
            severity: "warning",
            code: "user-config",
            path: Some(format_path(&path)),
            line: Some(*line),
            column: None,
            message: message.clone(),
            sites: Vec::new(),
            authority: Vec::new(),
        })
        .collect();
    Ok(Guidance {
        path,
        text,
        scan,
        cautions,
    })
}

/// Persist the conversation preference and synchronize it into every agent's
/// global instructions (§FS-integrations.4.3.8). An override moves the request,
/// never the gate's verdict (§FS-integrations.4.4.1).
pub(super) fn install(
    g: Guidance,
    opts: &IntegrationsOpts,
    events: &mut Vec<IntegrationEvent>,
) -> Result<(), Fault> {
    let Guidance {
        path,
        text,
        mut scan,
        ..
    } = g;
    let requested_target = opts.conversation_target;
    let scoped_agent = opts.agent.map(|agent| agent.name());
    let effective = opts
        .conversation
        .or(scan.preference)
        .unwrap_or(ConversationRendering::Plain);
    // A scoped write leaves the base exactly as it was (§FS-integrations.4.4.2).
    let machine_target = if scoped_agent.is_some() {
        scan.target.unwrap_or_default()
    } else {
        requested_target.or(scan.target).unwrap_or_default()
    };
    let (updated, first) = install_reference_key(
        &text,
        "reference",
        "conversation",
        effective.name(),
        scan.preference == Some(effective),
    );
    let (mut updated, second) = install_reference_key(
        &updated,
        "reference",
        "conversation_target",
        machine_target.name(),
        scan.target == Some(machine_target),
    );
    let mut spliced = merge_outcomes(first, second);
    if let (Some(agent), Some(target)) = (scoped_agent, requested_target) {
        let previous = scan
            .agent_targets
            .iter()
            .find(|(name, _)| name == agent)
            .map(|(_, value)| *value);
        let (text, result) = install_reference_key(
            &updated,
            &agent_override_table(agent),
            "conversation_target",
            target.name(),
            previous == Some(target),
        );
        updated = text;
        spliced = merge_outcomes(spliced, result);
        if let Some(entry) = scan
            .agent_targets
            .iter_mut()
            .find(|(name, _)| name == agent)
        {
            entry.1 = target;
        } else {
            scan.agent_targets.push((agent.into(), target));
        }
    }
    // §FS-integrations.4.3.8: plan every instruction splice before writing any guidance.
    let mut plans = vec![(path, Some((updated, spliced)), None)];
    for target in GLOBAL_AGENT_INSTRUCTION_TARGETS {
        let path = expand(target.file)?;
        let in_use = expand(target.home).is_ok_and(|p| p.is_dir()) || path.is_file();
        if !in_use {
            let note = IntegrationNote::AgentHomeMissing { home: target.home };
            plans.push((path, None, Some(note)));
            continue;
        }
        let overridden = scan
            .agent_targets
            .iter()
            .find(|(name, _)| name == target.agent)
            .map(|(_, value)| *value);
        let requested = overridden.unwrap_or(machine_target);
        let gated = target.link_support.resolve(requested);
        let existing = read_optional_text(&path)?;
        let (updated, spliced) = install_agent_guidance_block(&existing, effective, gated)
            .map_err(|message| Fault::at(&path, message))?;
        let note = IntegrationNote::Guidance {
            rendering: effective,
            target: gated,
            requested,
            overridden: overridden.is_some(),
        };
        plans.push((path, Some((updated, spliced)), Some(note)));
    }
    for (path, write, note) in plans {
        let happened = if let Some((updated, spliced)) = write {
            if spliced != BlockOutcome::Unchanged {
                save(&path, &updated)?;
            }
            outcome(spliced)
        } else {
            IntegrationOutcome::Skipped
        };
        events.push(IntegrationEvent {
            note,
            ..event(path, happened)
        });
    }
    Ok(())
}
