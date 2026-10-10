//! A binding aimed at a whole value root: the component run it is compared
//! against, the space rule that keeps the join one-to-one, and what a
//! disagreement reports (§FS-values.3.1.2, §FS-values.5.2.2).

use super::value_mismatch::{rendered_value_path, value_mismatch, value_mismatch_text};
use crate::config::{Display, Frame, Schema};
use crate::model::{
    Declaration, Diagnostic, Site, ValueBinding, ValueComponent, first_unequal_component,
    joined_value_components, root_literal_parts,
};

/// One component of a root's run: its section path, its declaration line, and
/// its value.
type RunComponent<'a> = (String, usize, &'a ValueComponent);

/// The finding a root-aimed binding over a valid root earns, if any
/// (§FS-values.3.1.2). The space refusal precedes comparison, so a root it
/// refuses is never also a mismatch (§FS-values.5.1).
pub(super) fn root_binding_finding(
    binding: &ValueBinding,
    schema: &Schema,
    frame: Frame<'_>,
    declaration: &Declaration,
    path: Option<&str>,
) -> Option<Diagnostic> {
    let run = root_components(declaration, path)?;
    let site = |line| Site {
        path: declaration.file.clone(),
        line,
    };
    let root = rendered_value_path(schema, frame, binding, path);
    if let Some((index, (_, line, _))) = run
        .iter()
        .enumerate()
        .find(|(_, (_, _, component))| component.decoded.contains(' '))
    {
        return Some(space_refusal(
            binding,
            &root,
            index + 1,
            site(*line),
            frame.display,
        ));
    }
    let declared = run
        .iter()
        .map(|(_, _, component)| *component)
        .collect::<Vec<_>>();
    let parts = root_literal_parts(&binding.authored);
    if parts.len() != declared.len() {
        // §FS-values.5.2.2: no component is at fault, so the finding names the
        // root, at component 1's site, which every valid root has.
        return Some(value_mismatch_text(
            binding,
            &root,
            &binding.authored.decoded,
            &joined_value_components(&declared),
            "",
            site(run[0].1),
            frame.display,
        ));
    }
    let index = first_unequal_component(&parts, &declared)?;
    let (coordinate, line, component) = &run[index];
    Some(value_mismatch(
        binding,
        &rendered_value_path(schema, frame, binding, Some(coordinate)),
        &parts[index],
        component,
        site(*line),
        frame.display,
    ))
}

/// The components `.1` through `.N` a valid root owns, in declared order; a
/// whole declaration's are its top-level coordinates (§FS-values.2.1,
/// §FS-values.2.4.3). `None` for an empty run or a component with no value,
/// which a valid root never has, so nothing is compared against a partial run.
fn root_components<'a>(
    declaration: &'a Declaration,
    path: Option<&str>,
) -> Option<Vec<RunComponent<'a>>> {
    let mut run = Vec::new();
    for index in 1usize.. {
        let coordinate = match path {
            Some(path) => format!("{path}.{index}"),
            None => index.to_string(),
        };
        let Some(info) = declaration.sections.get(&coordinate) else {
            break;
        };
        let value = info.value.as_ref()?;
        run.push((coordinate, info.line, value));
    }
    (!run.is_empty()).then_some(run)
}

/// §FS-values.3.1.2: a value whose component contains the separator cannot be
/// bound at its root, so the refusal names the first such component and carries
/// its declaration site as its one `sites` entry (§FS-errors.5.4).
fn space_refusal(
    binding: &ValueBinding,
    root: &str,
    component: usize,
    site: Site,
    display: Display<'_>,
) -> Diagnostic {
    let declared_site = format!("{}:{}", display.path(&site.path), site.line);
    Diagnostic {
        code: "invalid-value-binding",
        path: Some(binding.file.clone()),
        line: Some(binding.line),
        column: Some(binding.column),
        message: format!(
            "invalid value binding: {root} cannot be bound at its root because component \
             {component} contains an ASCII space at {declared_site}"
        ),
        sites: vec![site],
        authority: Vec::new(),
    }
}
