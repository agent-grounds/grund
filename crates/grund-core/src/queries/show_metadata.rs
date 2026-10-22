//! Transport metadata sourced before rendering (§FS-distribution.3.2.2.2).
use crate::config::Config;
use crate::grammar::render_catalog_id;
use crate::model::{Catalog, Id};
use serde_json::{Value, json};

pub(crate) fn show_metadata(c: &Config, f: &Catalog, id: &Id, section: Option<&str>) -> Value {
    let manifest = f
        .declarations
        .get(id)
        .and_then(|ds| ds.first())
        .and_then(|d| d.e2e_case.as_ref())
        .map(|e| {
            // §AR-scanner.6.2: the parsed words, `[]` with no `command.args`.
            json!({"kind":"E2E","args":e.args,"expected_exit":e.expected_exit,
            "fixtures":e.fixtures.iter().map(|p|crate::model::format_path(p)).collect::<Vec<_>>()})
        });
    json!({"id":render_catalog_id(&c.grammar,&f.declarations,id),"section":section,
        "kind_title":c.kinds.iter().find(|k|k.kind==id.kind).and_then(|k|k.title.clone()),
        "manifest":manifest})
}
