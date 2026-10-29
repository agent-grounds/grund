//! The *Fix* fillers, rendered from the §FS-errors.5.5 catalog by
//! `scripts/render_check_codes.py`: edit the catalog, then run the script.
//!
//! A message's remedy is built only by its code's filler, which fills the row's
//! template from the embedded catalog, so no remedy is worded in the checker and
//! a misspelt slot or case does not compile (§AR-config.9.1). The fillers live
//! one file per specification page their rows' *Check* cells name, and are
//! re-exported here, so a caller names each as `remedies::<filler>`.

mod check;
mod config_v2;
mod declarations;
mod fields;
mod rules;
mod scan;
mod section_lock;
mod values;

pub(crate) use check::*;
pub(crate) use config_v2::*;
pub(crate) use declarations::*;
pub(crate) use fields::*;
pub(crate) use rules::*;
pub(crate) use scan::*;
pub(crate) use section_lock::*;
pub(crate) use values::*;
