//! Test module: the mapping cases of §FS-config-migrate.mapping for the
//! top-level, `[reference]` and `[id]` keys.

use super::{Case, Outcome::*, case};

pub(super) const CASES: &[Case] = &[
    case(
        "grund_config_version",
        "",
        &[Mapped("grund_config_version = 2\n")],
    ),
    case(
        "project_name",
        then_fs!("project_name = \"demo\"\n"),
        &[Mapped("project_name = \"demo\"")],
    ),
    case(
        "project_description",
        then_fs!("project_description = \"A demo\"\n"),
        &[Mapped("[presentation]\ndescription = \"A demo\"")],
    ),
    case(
        "kinds",
        "kinds = []\n",
        &[Lacks("[schema.kinds.FS]"), Lacks("[schema.kinds.GRUND]")],
    ),
    case(
        "[reference] conversation",
        then_fs!("[reference]\nconversation = \"link\"\n"),
        &[Mapped("conversation = \"link\"")],
    ),
    case(
        "[reference] grounding_level",
        then_fs!("[reference]\nrequire_grounding = true\ngrounding_level = 2\n"),
        &[Mapped("[rules.citations.grounding]\nmust = \"h2\"\n")],
    ),
    case(
        "[reference] grounding_level",
        fs_with!("require_grounding = true\n[reference]\ngrounding_level = 2\n"),
        &[Refused(
            "asks for a unit only --require-grounding switches on",
        )],
    ),
    case(
        "[reference] grounding_level",
        then_fs!("[reference]\ngrounding_level = 2\n"),
        &[Invalid("nothing turns grounding on")],
    ),
    case(
        "[reference] inline_note_layout",
        then_fs!("[reference]\ninline_note_layout = \"citation-first-colon\"\n"),
        &[Mapped(
            "[schema.notes.layout]\nmay = \"citation-first-colon\"\n",
        )],
    ),
    case(
        "[reference] inline_note_layout_check",
        then_fs!(
            "[reference]\ninline_note_layout = \"citation-first-colon\"\ninline_note_layout_check = \"error\"\n"
        ),
        &[Mapped(
            "[schema.notes.layout]\nmust = \"citation-first-colon\"\n",
        )],
    ),
    case(
        "[reference] inline_note_max_columns",
        then_fs!("[reference]\ninline_note_max_columns = 80\n"),
        &[Mapped("[schema.notes.columns]\nmust = 80\n")],
    ),
    case(
        "[reference] inline_note_max_lines",
        then_fs!("[reference]\ninline_note_max_lines = 5\n"),
        &[Mapped("[schema.notes.lines]\nmust = 5\n")],
    ),
    case(
        "[reference] inline_note_suggested_lines",
        then_fs!("[reference]\ninline_note_suggested_lines = 2\n"),
        &[Mapped("[schema.notes.lines]\nmust = 3\nmay = 2\n")],
    ),
    case(
        "[reference] warn_on_suggested",
        then_fs!("[reference]\ninline_note_suggested_lines = 2\nwarn_on_suggested = true\n"),
        &[Mapped("[schema.notes.lines]\nmust = 3\nwarn = 2\n")],
    ),
    case(
        "[reference] inline_style",
        then_fs!("[reference]\ninline_style = \"citation-only\"\n"),
        &[Mapped("[schema.notes.text]\nmust = false\n")],
    ),
    case(
        "[reference] lead_size_warning",
        then_fs!("[reference]\nlead_size_warning = { max = 200, unit = \"words\" }\n"),
        &[Mapped("[schema.leads.words]\nwarn = 200\n")],
    ),
    case(
        "[reference] lead_size_warning",
        then_fs!("[reference]\nlead_size_warning = { max = 20, unit = \"lines\" }\n"),
        &[Refused("lead_size_warning counts lines")],
    ),
    case(
        "[reference] marker",
        then_fs!("[reference]\nmarker = \"@\"\n"),
        &[Mapped("marker = \"@\"\n")],
    ),
    case(
        "[reference] require_grounding",
        then_fs!("[reference]\nrequire_grounding = true\n"),
        &[Mapped("[rules.citations.grounding]\nmust = \"file\"\n")],
    ),
    case(
        "[reference] shorthand",
        then_fs!("[reference]\nshorthand = \"accepted\"\n"),
        &[Mapped("shorthand = \"accepted\"\n")],
    ),
    case(
        "[reference] strict",
        then_fs!("[reference]\nstrict = true\n"),
        &[Lacks("strict")],
    ),
    case(
        "[reference] strict",
        then_fs!("[reference]\nstrict = false\n"),
        &[Refused("strict = false recognizes bare citations")],
    ),
    case(
        "[reference] trigger",
        then_fs!("[reference]\ntrigger = \"%%\"\n"),
        &[Mapped("trigger = \"%%\"\n")],
    ),
    case(
        "[id] format",
        then_fs!("[id]\nformat = \"{kind}-{slug}\"\n"),
        &[Mapped("id_format = \"{kind}-{slug}\"\n")],
    ),
    case(
        "[id] named_sections",
        then_fs!("[id]\nnamed_sections = true\n"),
        &[Lacks("named_sections")],
    ),
    case(
        "[id] named_sections",
        then_fs!("[id]\nnamed_sections = false\n"),
        &[Refused("named_sections = false turns named sections off")],
    ),
    case(
        "[id] number_pattern",
        then_fs!("[id]\nformat = \"{kind}-{number}\"\nnumber_pattern = \"[0-9]{3}\"\n"),
        &[Mapped("number_pattern = \"[0-9]{3}\"\n")],
    ),
    case(
        "[id] section_heading_levels",
        then_fs!("[id]\nsection_heading_levels = \"warn\"\n"),
        &[Mapped("heading_depth = \"warn\"\n")],
    ),
    case(
        "[id] section_separator",
        then_fs!("[id]\nsection_separator = \":\"\n"),
        &[Mapped("section_separator = \":\"\n")],
    ),
    case(
        "[id] slug_pattern",
        then_fs!("[id]\nslug_pattern = \"[a-z]+\"\n"),
        &[Mapped("slug_pattern = \"[a-z]+\"\n")],
    ),
];
