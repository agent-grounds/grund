//! Loaded configuration vocabulary for repair guidance (§FS-show.3.5.1).

use super::{Frame, Schema};

/// §FS-check.1.3.7, §FS-check.3.14.6: the loaded model's ordinary source scope.
pub(crate) fn source_scope_name(version: u32) -> &'static str {
    if version == 2 {
        "configured places and source scope"
    } else {
        "[scan] include"
    }
}

/// §FS-show.3.5.1: the effective template together with the key that owns it.
pub(crate) struct FormatGuidance<'a> {
    pub(crate) key: String,
    pub(crate) template: &'a str,
}

impl FormatGuidance<'_> {
    /// §FS-show.3.5.1: the sentence naming the effective template, which every
    /// query refusal adapter reaches through the api's `invalid_id_hint`, the one
    /// place that asks first whether a config below the run is the cause.
    pub(crate) fn hint(&self) -> String {
        format!(
            "hint: this repo's {} is `{}` (run `grund config show`); `grund list` shows the IDs that exist",
            self.key, self.template
        )
    }

    /// §FS-check.checks.off-format-citation: `<key> = "<template>"`, then the
    /// pattern of each component the template holds, in its order and by the
    /// bare key, because the patterns are schema-wide.
    pub(crate) fn with_patterns(&self, schema: &Schema) -> String {
        let mut components = [
            ("{number}", "number_pattern", &schema.ids.number_pattern),
            ("{slug}", "slug_pattern", &schema.ids.slug_pattern),
        ]
        .into_iter()
        .filter_map(|(placeholder, key, pattern)| {
            let at = self.template.find(placeholder)?;
            Some((at, format!("{key} = \"{pattern}\"")))
        })
        .collect::<Vec<_>>();
        components.sort();
        let patterns = components
            .into_iter()
            .map(|(_, pattern)| pattern)
            .collect::<Vec<_>>();
        let mut clause = format!("{} = \"{}\"", self.key, self.template);
        if !patterns.is_empty() {
            clause.push_str(" with ");
            clause.push_str(&patterns.join(" and "));
        }
        clause
    }
}

/// §FS-show.3.5.1: use the selected kind's override, or the global template
/// when no kind was selected. Recognition here only selects advice; it never
/// changes the resolver's decision about the operand.
pub(crate) fn format_guidance<'a>(
    schema: &'a Schema,
    frame: Frame<'_>,
    operand: Option<&str>,
) -> FormatGuidance<'a> {
    let name = operand.and_then(|text| frame.grammar().kind_for_guidance(text));
    kind_format_guidance(schema, frame, name)
}

/// §FS-show.3.5.1: [`format_guidance`] for a kind the caller already selected,
/// as §FS-check.checks.off-format-citation names the kind its token opens with.
pub(crate) fn kind_format_guidance<'a>(
    schema: &'a Schema,
    frame: Frame<'_>,
    name: Option<&str>,
) -> FormatGuidance<'a> {
    let overridden = name.and_then(|name| {
        schema.kinds().find_map(|(kind, settings)| {
            (kind == name)
                .then_some(settings.id_format.as_deref())
                .flatten()
                .map(|template| (kind, template))
        })
    });
    match (frame.version, overridden) {
        (2, Some((kind, template))) => FormatGuidance {
            key: format!("[schema.kinds.{kind}] id_format"),
            template,
        },
        (_, Some((_, template))) => FormatGuidance {
            key: "[[kinds]] format".into(),
            template,
        },
        (version, None) => FormatGuidance {
            key: if version == 2 {
                "[schema] id_format"
            } else {
                "[id] format"
            }
            .into(),
            template: &schema.ids.format,
        },
    }
}
