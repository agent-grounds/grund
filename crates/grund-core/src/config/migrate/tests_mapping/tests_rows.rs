//! Test module: the mapping cases of §FS-config-migrate.mapping for `[[kinds]]`,
//! `[scan]`, `[output]`, `[fmt]`, `[workspace]`, `[citations]` and the implicit
//! defaults.

use super::{Case, FS, Outcome::*, case, with};

pub(super) const CASES: &[Case] = &[
    with(
        "[[kinds]] citable",
        fs_with!("[[kinds]]\nkind = \"test\"\nfolder = \"tests\"\ncitable = false\n"),
        &[("tests/a.rs", "// FS-a\n")],
        &[Mapped(
            "[schema.kinds.test]\nfolders = [\"tests\"]\ncitable = false\n",
        )],
    ),
    with(
        "[[kinds]] fetch",
        "[[kinds]]\nkind = \"TICKET\"\nfolder = \"tickets\"\nfetch = \"scripts/fetch\"\n",
        &[("tickets/", "")],
        &[Mapped("fetch = \"scripts/fetch\"\n")],
    ),
    case(
        "[[kinds]] file",
        "[[kinds]]\nkind = \"GRUND\"\nfile = \"docs/FS-a.md\"\n",
        &[Mapped("[schema.kinds.GRUND]\nfiles = [\"docs/FS-a.md\"]\n")],
    ),
    case(
        "[[kinds]] folder",
        FS,
        &[Mapped("[schema.kinds.FS]\nfolders = [\"docs\"]\n")],
    ),
    case(
        "[[kinds]] format",
        fs_with!("format = \"{kind}-{slug}\"\n"),
        &[Mapped(
            "[schema.kinds.FS]\nfolders = [\"docs\"]\nid_format = \"{kind}-{slug}\"\n",
        )],
    ),
    case(
        "[[kinds]] grounding_level",
        fs_with!("require_grounding = true\ngrounding_level = 2\n"),
        &[Mapped("[rules.citations.FS.grounding]\nmust = \"h2\"\n")],
    ),
    case(
        "[[kinds]] grounding_level",
        fs_with!("grounding_level = 2\n"),
        &[Invalid("nothing turns grounding on for it")],
    ),
    case(
        "[[kinds]] index",
        fs_with!("index = false\n"),
        &[Mapped(
            "[schema.kinds.FS]\nfolders = [\"docs\"]\nindex = false\n",
        )],
    ),
    case("[[kinds]] kind", FS, &[Mapped("[schema.kinds.FS]\n")]),
    case(
        "[[kinds]] kind",
        fs_with!("[[kinds]]\nkind = \"impl\"\ncitable = false\n"),
        &[Refused("renames the complement")],
    ),
    case(
        "[[kinds]] require_grounding",
        fs_with!("require_grounding = true\n"),
        &[Mapped("[rules.citations.FS.grounding]\nmust = \"file\"\n")],
    ),
    with(
        "[[kinds]] resolve",
        "[[kinds]]\nkind = \"TICKET\"\nfolder = \"tickets\"\nfetch = \"scripts/fetch\"\nresolve = \"should\"\n",
        &[("tickets/", "")],
        &[Mapped("[rules.resolution]\nTICKET = \"warn\"\n")],
    ),
    case(
        "[[kinds]] rules",
        "[[kinds]]\nkind = \"RULE\"\nfolder = \"docs\"\nrules = true\n",
        &[Mapped("form = \"rule\"\n")],
    ),
    with(
        "[[kinds]] scan",
        fs_with!(
            "[[kinds]]\nkind = \"skill\"\nfolder = \"skills\"\ncitable = false\nscan = false\n"
        ),
        &[("skills/a.md", "x\n")],
        &[Mapped("citable = false\nscan = false\n")],
    ),
    case(
        "[[kinds]] title",
        fs_with!("title = \"Specs\"\n"),
        &[Mapped("[presentation.kinds.FS]\ntitle = \"Specs\"\n")],
    ),
    case(
        "[[kinds]] value_chapter",
        fs_with!("value_chapter = \"values\"\n"),
        &[Refused("value_chapter would become a field")],
    ),
    case(
        "[[kinds]] values",
        fs_with!("values = true\n"),
        &[Mapped("form = \"value\"\n")],
    ),
    case(
        "[scan] comment_prefixes",
        then_fs!("[scan]\ncomment_prefixes = [\"//\", \"#\", \";\", \"--\"]\n"),
        &[Mapped(
            "comment_prefixes = [\"//\", \"#\", \";\", \"--\"]\n",
        )],
    ),
    case(
        "[scan] comment_prefixes",
        then_fs!("[scan]\ncomment_prefixes = [\"//\", \"#\"]\n"),
        &[Refused(
            "whose doc comments no prefix of [scan] comment_prefixes spells",
        )],
    ),
    case(
        "[scan] docstring_python",
        then_fs!("[scan]\ndocstring_python = false\n"),
        &[
            Mapped("py = \"v1_plain\"\n"),
            Lacks("docstrings = \"python\""),
        ],
    ),
    case(
        "[scan] exclude",
        then_fs!("[scan]\nexclude = [\"vendor\"]\n"),
        &[Mapped("exclude = [\"vendor/\"]\n")],
    ),
    case(
        "[scan] exclude",
        then_fs!("[scan]\nexclude = [\"a/vendor\"]\n"),
        &[Refused("contains `/`, so version 1 never matched it")],
    ),
    case(
        "[scan] extensions",
        then_fs!("[scan]\nextensions = [\"md\", \"rs\"]\n"),
        &[Mapped("languages = [\"v1_markdown\", \"v1_c_family\"]\n")],
    ),
    case(
        "[scan] extensions",
        fs_with!("values = true\n[scan]\nextensions = [\"md\", \"rs\", \"json\"]\n"),
        &[Refused("scans `json` as text while a values kind reads it")],
    ),
    with(
        "[scan] include",
        then_fs!("[scan]\ninclude = [\"docs\", \"src\"]\n"),
        &[("src/a.rs", "// x\n")],
        &[Mapped("[schema.kinds.code]\nfolders = [\"src\"]\n")],
    ),
    case(
        "[scan] include",
        then_fs!("[scan]\ninclude = [\"docs\", \"nope\"]\n"),
        &[Refused("names `nope`, which does not exist")],
    ),
    // §FS-config-migrate.3.1.3: the default list's missing entries are refused
    // too, in one block whose repair authors the list without them.
    with(
        "[scan] include",
        then_fs!("# default include\n"),
        &[("requirements.md", "Text.\n"), ("src/a.rs", "// x\n")],
        &[Refused(
            "[scan] include is not set, so version 1 reads its default, which names `e2e`, which does not exist",
        )],
    ),
    case(
        "[scan] respect_gitignore",
        then_fs!("[scan]\nrespect_gitignore = false\n"),
        &[Mapped("respect_gitignore = false\n")],
    ),
    case(
        "[output] color",
        then_fs!("[output]\ncolor = \"never\"\n"),
        &[Notice("nothing reads it")],
    ),
    case(
        "[output] format",
        then_fs!("[output]\nformat = \"json\"\n"),
        &[Notice("pass --format json to keep it")],
    ),
    case(
        "[output] relative_paths",
        then_fs!("[output]\nrelative_paths = false\n"),
        &[Notice("pass --path-base=invocation to keep it")],
    ),
    case(
        "[fmt.cross_refs] anchor_format",
        then_fs!("[fmt.cross_refs]\nanchor_format = \"github\"\n"),
        &[Mapped("anchors = \"github\"\n")],
    ),
    case(
        "[fmt.cross_refs] anchor_format",
        then_fs!("[fmt.cross_refs]\nanchor_format = \"gitlab\"\n"),
        &[Refused(
            "anchor_format = \"gitlab\" has no version 2 spelling",
        )],
    ),
    case(
        "[fmt.cross_refs] enabled",
        then_fs!("[fmt.cross_refs]\nenabled = true\n"),
        &[Mapped("links = \"index\"\n")],
    ),
    case(
        "[fmt.cross_refs] enabled",
        then_fs!("[fmt.cross_refs]\nenabled = false\n"),
        &[Refused("enabled = false has no version 2 spelling")],
    ),
    case(
        "[fmt] exclude",
        then_fs!("[fmt]\nexclude = [\"docs/FS-a.md\"]\n"),
        &[Mapped("exclude = [\"docs/FS-a.md\"]\n")],
    ),
    with(
        "[workspace] members",
        then_fs!("[workspace]\nmembers = [\"sub\"]\n"),
        &[("sub/grund.toml", "grund_config_version = 2\n")],
        &[Mapped("[workspace]\nmembers = [\"sub\"]\n")],
    ),
    with(
        "[workspace] optional_members",
        then_fs!("[workspace]\nmembers = [\"sub\"]\noptional_members = [\"opt\"]\n"),
        &[("sub/grund.toml", "grund_config_version = 2\n")],
        &[Mapped("optional_members = [\"opt\"]\n")],
    ),
    with(
        "[workspace] include_root",
        then_fs!("[workspace]\nmembers = [\"sub\"]\ninclude_root = false\n"),
        &[("sub/grund.toml", "grund_config_version = 2\n")],
        &[Mapped("include_root = false\n")],
    ),
    case(
        "[citations] default",
        then_fs!("[citations]\ndefault = \"may\"\n"),
        &[Mapped("[rules.citations]\ndefault = \"may\"\n")],
    ),
    case(
        "[citations] default",
        then_fs!("[citations]\ndefault = \"should\"\n"),
        &[
            Notice("version 1 gives a positive default no force, so it is written as may"),
            Mapped("[rules.citations]\ndefault = \"may\"\n"),
        ],
    ),
    with(
        "[citations.<KIND>] default",
        CITATIONS_KIND,
        CITATIONS_PATHS,
        &[Mapped("[rules.citations.FS]\ndefault = \"may\"\n")],
    ),
    with(
        "[citations.<KIND>] must",
        CITATIONS_KIND,
        CITATIONS_PATHS,
        &[Mapped("must = [\"A\"]\n")],
    ),
    with(
        "[citations.<KIND>] should",
        CITATIONS_KIND,
        CITATIONS_PATHS,
        &[Mapped("should = [\"C\"]\n")],
    ),
    with(
        "[citations.<KIND>] may",
        CITATIONS_KIND,
        CITATIONS_PATHS,
        &[Mapped("may = [\"FS\"]\n")],
    ),
    with(
        "[citations.<KIND>] should-not",
        CITATIONS_KIND,
        CITATIONS_PATHS,
        &[Mapped("should-not = [\"D\"]\n")],
    ),
    with(
        "[citations.<KIND>] must-not",
        CITATIONS_KIND,
        CITATIONS_PATHS,
        &[Mapped("must-not = [\"B\"]\n")],
    ),
    with(
        "built-in kinds",
        "",
        &[("docs/functional-spec/FS-b.md", "# FS-b: B\n")],
        &[Mapped(
            "[schema.kinds.GRUND]\nfiles = [\"docs/grund.md\"]\n",
        )],
    ),
    with(
        "legacy FS home",
        "",
        &[("docs/functional-spec/FS-b.md", "# FS-b: B\n")],
        &[Mapped(
            "[schema.kinds.FS]\nfolders = [\"docs/functional-spec\"]\n",
        )],
    ),
    with(
        "name-keyed index",
        fs_with!("[[kinds]]\nkind = \"E2E\"\nfolder = \"e2e\"\n"),
        &[("e2e/", "")],
        &[Mapped(
            "[schema.kinds.E2E]\nfolders = [\"e2e\"]\nindex = false\n",
        )],
    ),
    with(
        "fetch resolves must",
        "[[kinds]]\nkind = \"TICKET\"\nfolder = \"tickets\"\nfetch = \"scripts/fetch\"\n",
        &[("tickets/", "")],
        &[Mapped("fetch = \"scripts/fetch\"\n")],
    ),
    case(
        "kind grounding inherits",
        then_fs!("[reference]\nrequire_grounding = true\n"),
        &[
            Mapped("[rules.citations.grounding]\nmust = \"file\"\n"),
            Lacks("[rules.citations.FS.grounding]"),
        ],
    ),
    with(
        "complement name",
        then_fs!("[scan]\ninclude = [\"docs\", \"src\"]\n"),
        &[("src/a.rs", "// x\n")],
        &[Mapped("[schema.kinds.code]\n")],
    ),
    case("complement name", FS, &[Lacks("[schema.kinds.code]")]),
    case("nesting", FS, &[Mapped("[schema.kinds.FS]\n")]),
    with(
        "nesting",
        fs_with!("[[kinds]]\nkind = \"AR\"\nfolder = \"docs/ar\"\n"),
        &[("docs/ar/AR-x.md", "# AR-x: X\n")],
        &[Refused("home `docs/ar` is inside `FS` home `docs`")],
    ),
    case("strict", FS, &[Lacks("strict")]),
    case(
        "named sections",
        // The comment keeps `named` from setting the key: the default is the case.
        concat!(
            "# no named_sections\n",
            "[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\n"
        ),
        &[Refused("[id] named_sections is not set")],
    ),
    case(
        "scan exclusions",
        FS,
        &[Written(
            "[schema.sources] exclude = [\"target/\", \"node_modules/\"",
        )],
    ),
    case("version", FS, &[Mapped("grund_config_version = 2\n")]),
    case(
        "any-comment declarations",
        FS,
        &[Written("[schema.sources] declarations = \"any-comment\"")],
    ),
    case(
        "guidance at the root",
        FS,
        &[Written("[presentation] rules = \"root\"")],
    ),
];

const CITATIONS_KIND: &str = concat!(
    "[[kinds]]\nkind = \"FS\"\nfolder = \"docs\"\n",
    "[[kinds]]\nkind = \"A\"\nfolder = \"a\"\n[[kinds]]\nkind = \"B\"\nfolder = \"b\"\n",
    "[[kinds]]\nkind = \"C\"\nfolder = \"c\"\n[[kinds]]\nkind = \"D\"\nfolder = \"d\"\n",
    "[citations.FS]\ndefault = \"may\"\nmay = [\"FS\"]\nmust = [\"A\"]\n",
    "must-not = [\"B\"]\nshould = [\"C\"]\nshould-not = [\"D\"]\n",
);
const CITATIONS_PATHS: &[(&str, &str)] = &[("a/", ""), ("b/", ""), ("c/", ""), ("d/", "")];
