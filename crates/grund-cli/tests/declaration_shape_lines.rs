//! Process-level coverage of §FS-schema.4.1: the heading a field's line shows under a
//! declaration written `# <ID>: …` (§FS-schema.4.1.1), its code spans and placeholders
//! (§FS-schema.4.1.2), front matter (§FS-schema.4.1.3), a value kind's file form
//! (§FS-schema.4.1.4), the issue's RFC kind (§FS-schema.4.1.5), and the legend's depth line
//! (§FS-schema.8.4), with the block's section (§FS-init.2.3.5) and its revision pairs
//! (§FS-init.2.3.7), kept with the end-to-end tests.

#[path = "../../../tests/e2e/declaration_shape_lines.rs"]
mod contract;
