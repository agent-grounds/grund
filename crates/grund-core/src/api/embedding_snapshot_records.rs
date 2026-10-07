//! Source fields retained by the owned scanner snapshot (§FS-distribution.3.2.2.3).
use super::embedding_data::Data;
use crate::*;
use serde_json::{Value, json};

impl Data for DeclarationSource {
    fn data(&self) -> Value {
        match self {
            Self::Text => json!({"kind":"text"}),
            Self::Json {
                member_slice,
                key_column,
                key_text,
            } => {
                json!({"kind":"json","member_slice":member_slice,"key_column":key_column,"key_text":key_text})
            }
        }
    }
}
impl Data for ValueComponent {
    fn data(&self) -> Value {
        json!({"decoded":self.decoded,"kind":match self.kind {
            ValueComponentKind::Number=>"number",ValueComponentKind::String=>"string"},
            "source_slice":self.source_slice,"column":self.column})
    }
}
impl Data for SectionInfo {
    fn data(&self) -> Value {
        json!({"title":self.title,"line":self.line,"heading_level":self.heading_level,
            "value":self.value.as_ref().map(Data::data),
            "value_root":self.value_root.as_ref().map(|v|json!({"valid":v.valid,
                "origin":match v.origin {ValueRootOrigin::Chapter=>json!({"kind":"chapter"}),
                ValueRootOrigin::Marker {column}=>json!({"kind":"marker","column":column})}}))})
    }
}
impl Data for InlineCitationSite {
    fn data(&self) -> Value {
        json!({"first_line":self.first_line,"last_line":self.last_line,
            "max_columns":self.max_columns,"has_note":self.has_note,"layout_violations":self.layout_violations})
    }
}
impl Data for E2eCase {
    fn data(&self) -> Value {
        json!({"dir":self.dir.data(),"args":self.args,"expected_exit":self.expected_exit,
            "fixtures":self.fixtures.data(),"spec_refs":self.spec_refs.iter()
                .map(|s|json!({"namespace":s.namespace,"kind":s.kind})).collect::<Vec<_>>()})
    }
}
