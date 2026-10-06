//! Complete host records, independent of CLI projection (§FS-distribution.3.0.1).

use crate::*;
use serde_json::{Value, json};

pub(super) trait Data {
    fn data(&self) -> Value;
}

macro_rules! scalar {
    ($($ty:ty),*) => { $(impl Data for $ty {
        fn data(&self) -> Value { json!(self) }
    })* };
}
scalar!(String, str, bool, usize, u32);
impl<T: Data + ?Sized> Data for &T {
    fn data(&self) -> Value {
        (*self).data()
    }
}
impl<T: Data> Data for Option<T> {
    fn data(&self) -> Value {
        self.as_ref().map(Data::data).unwrap_or(Value::Null)
    }
}
impl<T: Data> Data for Vec<T> {
    fn data(&self) -> Value {
        Value::Array(self.iter().map(Data::data).collect())
    }
}
impl Data for std::path::PathBuf {
    fn data(&self) -> Value {
        crate::model::format_path(self).data()
    }
}
macro_rules! record {
    ($ty:ty; $($field:ident),* $(,)?) => {
        impl Data for $ty {
            fn data(&self) -> Value { json!({ $(stringify!($field): self.$field.data()),* }) }
        }
    };
}
record!(FindingSite; path, line);
record!(Report; errors, warnings, suggestions);
// §FS-distribution.3.3.1: channel is explicit, and suggestion severity is nullable.
impl Data for Finding {
    fn data(&self) -> Value {
        let suggestion = self.severity == "suggestion";
        json!({"severity": if suggestion { None } else { Some(self.severity) },
            "channel": if suggestion { Some("suggestion") } else { None },
            "code": self.code, "path": self.path, "line": self.line,
            "column": self.column, "message": self.message,
            "sites": self.sites.data(), "authority": self.authority})
    }
}
record!(ApiScanError; path, message);
record!(ShowSection; path, title, depth);
record!(ShowOutput; body, path, line, json, sections);
record!(RefHit; project, path, line, column, id, section, marker, text,
    enclosing_declaration, enclosing_section);
record!(ListEntry; project, id, section, section_separator, kind, path, line, title,
    stub, defines, refs, duplicate, value_roots);
record!(ListValueRoot; id, valid);
record!(ListSummary; project, kind, title, home, count);
record!(CoverCitation; project, path, line, column, id, section, marker, text,
    enclosing_declaration, enclosing_section);
record!(CoverEntry; project, path, citations);
record!(CoverTextCitation; line, column, text);
record!(CoverTextEntry; path, citations);
record!(FmtChange; path, line, label);
record!(IdProposal; id, kind, number, slug, folder, file, e2e_case_dir,
    file_holds_single_declaration);
record!(ListSizeEntry; project, id, section, section_separator, kind, path, line,
    stub, defines, duplicate, measurements);
impl Data for ListSizeMeasurement {
    fn data(&self) -> Value {
        json!({"unit": self.unit.as_str(),
        "lead": self.lead, "full": self.full})
    }
}
record!(InitEvent; verb, path);
record!(InitNext; docs, entrypoint, fs_home, scan_reads_file);
impl Data for InitFsHome {
    fn data(&self) -> Value {
        match self {
            Self::File {
                path,
                heading_name,
                heading_marker,
            } => json!({
                "kind": "file", "path": path, "heading_name": heading_name,
                "heading_marker": heading_marker}),
            Self::Folder { path } => json!({"kind": "folder", "path": path,
                "heading_name": null, "heading_marker": null}),
        }
    }
}
impl Data for InitOutput {
    fn data(&self) -> Value {
        json!({"events": self.events.data(),
        "errors": self.errors.data(), "notes": self.notes, "next": self.next.data(),
        "run_cautions": self.warnings.data(), "has_pending_changes": self.has_pending_changes()})
    }
}
