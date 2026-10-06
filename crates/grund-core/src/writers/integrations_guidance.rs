//! User preferences, agent gates and managed ownership (§FS-distribution.3.3.6).

use super::integrations_api::{event, located, save, tuple_error};
use super::*;
use anyhow::Result;
use serde_json::{Value, json};
use std::path::PathBuf;

pub(super) struct Guidance {
    path: PathBuf,
    text: String,
    scan: UserConfigScan,
    pub(super) cautions: Vec<Value>,
}

pub(super) fn load() -> Result<Guidance> {
    let path = user_grund_config_path().ok_or_else(|| {
        located(
            std::path::Path::new(USER_CONFIG_TARGET),
            "cannot resolve user configuration directory".into(),
        )
    })?;
    let text = read_optional_text(&path).map_err(tuple_error)?;
    let scan = scan_user_config(&text);
    let cautions = scan
        .problems
        .iter()
        .map(|(line, message)| {
            json!({"severity":"warning","channel":null,
        "code":"user-config","path":crate::model::format_path(&path),"line":line,"column":null,
        "message":message,"sites":[],"authority":[]})
        })
        .collect();
    Ok(Guidance {
        path,
        text,
        scan,
        cautions,
    })
}

pub(super) fn install(
    g: Guidance,
    requested: Option<ConversationRendering>,
    requested_target: Option<ConversationTarget>,
    scoped_agent: Option<&str>,
    events: &mut Vec<Value>,
) -> Result<()> {
    let Guidance {
        path,
        text,
        mut scan,
        ..
    } = g;
    let effective = requested
        .or(scan.preference)
        .unwrap_or(ConversationRendering::Plain);
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
    let mut outcome = merge_outcomes(first, second);
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
        outcome = merge_outcomes(outcome, result);
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
    let mut plans = vec![(path, Some((updated, outcome)), None)];
    for target in GLOBAL_AGENT_INSTRUCTION_TARGETS {
        let path = expand_target(target.file).ok_or_else(|| {
            located(
                std::path::Path::new(target.file),
                "cannot resolve home directory".into(),
            )
        })?;
        let in_use = expand_target(target.home).is_some_and(|p| p.is_dir()) || path.is_file();
        if !in_use {
            plans.push((path, None, Some(format!("no {}", target.home))));
            continue;
        }
        let overridden = scan
            .agent_targets
            .iter()
            .find(|(name, _)| name == target.agent)
            .map(|(_, value)| *value);
        let requested = overridden.unwrap_or(machine_target);
        let gated = target.link_support.resolve(requested);
        let existing = read_optional_text(&path).map_err(tuple_error)?;
        let (updated, outcome) = install_agent_guidance_block(&existing, effective, gated)
            .map_err(|e| located(&path, e))?;
        let note = if effective == ConversationRendering::Plain {
            "plain".to_owned()
        } else {
            format!(
                "link → {}{}",
                gated.name(),
                if gated != requested {
                    "; unverified here"
                } else if overridden.is_some() {
                    "; agent override"
                } else {
                    ""
                }
            )
        };
        plans.push((path, Some((updated, outcome)), Some(note)));
    }
    for (path, write, note) in plans {
        let verb = if let Some((updated, outcome)) = write {
            if outcome != BlockOutcome::Unchanged {
                save(&path, &updated)?;
            }
            block_outcome_verb(outcome)
        } else {
            "skipped"
        };
        events.push(event(&path, verb, note));
    }
    Ok(())
}
