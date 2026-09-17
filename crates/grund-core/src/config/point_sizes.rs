use anyhow::{Result, anyhow};
use std::path::Path;

use super::parse::{bail_config, parse_string, parse_usize};
// §AR-system.4: `format_path` renders a path for a report and is the renderer's
// (§AR-system.2.9) — read through the crate root until `output` is a module.
use crate::format_path;

/// The closed built-in point-size vocabulary, its opt-in warning policy and
/// what counting in one of its units means (§FS-list.3.4, §FS-config.3.1).
/// These config-facing types stay in this component so the general model remains
/// reserved for scan and declaration data (§AR-system.2.2, §AR-system.2.3).
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PointSizeUnit {
    Lines,
    Words,
    Bytes,
}

impl PointSizeUnit {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lines => "lines",
            Self::Words => "words",
            Self::Bytes => "bytes",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "lines" => Some(Self::Lines),
            "words" => Some(Self::Words),
            "bytes" => Some(Self::Bytes),
            _ => None,
        }
    }
}

/// Byte-defined size counting with no locale or Unicode-table input
/// (§FS-list.3.4). It sits beside the unit rather than beside either caller: it
/// is what `PointSizeUnit` *means*, and the query that slices a lead
/// (§AR-system.2.7) and the budget rule that judges one (§FS-check.4.13) must
/// count the same bytes.
pub(crate) fn measure_point_text(text: &str, unit: PointSizeUnit) -> usize {
    let ascii_space = |byte: &u8| matches!(*byte, b'\t'..=b'\r' | b' ');
    match unit {
        PointSizeUnit::Lines => text
            .as_bytes()
            .split(|byte| *byte == b'\n')
            .filter(|line| line.iter().any(|byte| !ascii_space(byte)))
            .count(),
        PointSizeUnit::Words => text
            .as_bytes()
            .split(ascii_space)
            .filter(|word| !word.is_empty())
            .count(),
        PointSizeUnit::Bytes => text.len(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LeadSizeWarning {
    pub max: usize,
    pub unit: PointSizeUnit,
}

/// Parse the one inline table in grund's line-oriented config surface
/// (§FS-config.3.1). Keeping this parser specific makes duplicate, missing, and
/// extra fields loud without silently widening the rest of the TOML subset.
pub(super) fn parse_lead_size_warning(
    path: &Path,
    line: usize,
    value: &str,
) -> Result<LeadSizeWarning> {
    if !(value.starts_with('{') && value.ends_with('}')) {
        return bail_config(
            path,
            line,
            "lead_size_warning must be `{ max = <N>, unit = \"lines|words|bytes\" }`".to_string(),
        );
    }
    let inner = value[1..value.len() - 1].trim();
    let mut max = None;
    let mut unit = None;
    for field in inner.split(',') {
        let Some((key, raw)) = field.split_once('=') else {
            return bail_config(path, line, "invalid lead_size_warning field".to_string());
        };
        let key = key.trim();
        let raw = raw.trim();
        match key {
            "max" => {
                if max.is_some() {
                    return bail_config(
                        path,
                        line,
                        "duplicate lead_size_warning field `max`".to_string(),
                    );
                }
                max = Some(parse_usize(path, line, raw).map_err(|_| {
                    anyhow!(
                        "{}:{}: lead_size_warning max must be a non-negative integer",
                        format_path(path),
                        line
                    )
                })?);
            }
            "unit" => {
                if unit.is_some() {
                    return bail_config(
                        path,
                        line,
                        "duplicate lead_size_warning field `unit`".to_string(),
                    );
                }
                let raw_unit = parse_string(path, line, raw).map_err(|_| {
                    anyhow!(
                        "{}:{}: lead_size_warning unit must be a string",
                        format_path(path),
                        line
                    )
                })?;
                unit = PointSizeUnit::parse(&raw_unit);
                if unit.is_none() {
                    return bail_config(
                        path,
                        line,
                        format!(
                            "unknown lead_size_warning unit `{raw_unit}` (expected lines, words, or bytes)"
                        ),
                    );
                }
            }
            "" => {
                return bail_config(path, line, "invalid lead_size_warning field".to_string());
            }
            other => {
                return bail_config(
                    path,
                    line,
                    format!("unknown lead_size_warning field `{other}`"),
                );
            }
        }
    }
    let Some(max) = max else {
        return bail_config(
            path,
            line,
            "lead_size_warning requires field `max`".to_string(),
        );
    };
    let Some(unit) = unit else {
        return bail_config(
            path,
            line,
            "lead_size_warning requires field `unit`".to_string(),
        );
    };
    Ok(LeadSizeWarning { max, unit })
}
