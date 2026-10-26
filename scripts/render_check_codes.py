#!/usr/bin/env python3
"""Render the `check` code catalog into the files grund-core embeds (§AR-config.9.1).

The table of §FS-errors.5.5 is the one place a code's meaning, severity and
switch are written down; the installed `grund` prints them from what this
script renders, so the binary says what the specification says and no meaning
is typed in Rust (§DF-config-explain.2.3). Each cell is read by the grammar of
§FS-errors.5.5.1, and a row it cannot read is refused rather than guessed at:

- `crates/grund-core/assets/explain/check-codes.json`: every selectable row,
  its cells split by version and read into what `grund config explain` prints
  (§FS-config-explain.4) and what the off line of §FS-check.1.4.1 is built from;
- `crates/grund-core/src/model/codes.rs`: one typed handle per selectable code,
  citing its section, the sorted list of them and the `out-of-scope-` pairing
  of §FS-check.3.14, so a finding is built only from a code the catalog has
  (§AR-system.2.2);
- `crates/grund-core/src/model/remedies.rs`: one typed filler per *Fix* template
  or case, taking its slots, so a message's remedy is built only from its row
  and worded nowhere in the checker (§AR-config.9.1).

A row marked `planned — ` is specified ahead of its release and is in neither.
Edit the catalog, then run the script:

    python scripts/render_check_codes.py          # rewrite both files
    python scripts/render_check_codes.py --check  # exit 1 if either is stale
"""

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PAGE = ROOT / "docs" / "functional-spec" / "FS-errors.md"
ASSET = ROOT / "crates" / "grund-core" / "assets" / "explain" / "check-codes.json"
HANDLES = ROOT / "crates" / "grund-core" / "src" / "model" / "codes.rs"
REMEDIES = ROOT / "crates" / "grund-core" / "src" / "model" / "remedies.rs"

HEADING = re.compile(r"^### 5\.5 ")
COLUMNS = ("Code", "Meaning", "Severity", "Ramp", "Enabled by", "Fix", "Note", "Check")
CODE = re.compile(r"^`([a-z][a-z0-9-]*)`$")
SPLIT = re.compile(r"^v2: (.*) · v1: (.*)$")
ABSENT = re.compile(r"^not in version ([12])$")
CHECK = re.compile(r"^\[§([A-Za-z0-9.-]+)\]\([^)]*\)$")
PLANNED = "planned — "
NONE = "none — "
CASES = " ‖ "
LABEL = re.compile(r"^([a-z][a-z0-9]*(?:-[a-z0-9]+)*): (.*)$")
BRACE = re.compile(r"\{([^{}]*)\}")
SLOT = re.compile(r"^[a-z]+(?: [a-z]+)*$")
REFERENCE = re.compile(r"^fix of `([a-z][a-z0-9-]*)`"
                       r"(?:, case `([a-z][a-z0-9]*(?:-[a-z0-9]+)*)`)?$")
WHOLE = re.compile(r"^\{fix of `([a-z][a-z0-9-]*)`\}$")
KEYWORDS = {"as", "type", "match", "ref", "mod", "use", "where", "in", "fn", "impl", "move",
            "self", "loop", "for", "if", "else", "const", "static", "struct", "enum", "trait"}
OUT_OF_SCOPE = "out-of-scope-"


class CatalogError(Exception):
    """A row the grammar of §FS-errors.5.5.1 cannot read."""


def rows():
    """The catalog's rows as `{column: cell}`, in table order."""
    lines = PAGE.read_text(encoding="utf-8").splitlines()
    start = next(index for index, line in enumerate(lines) if HEADING.match(line)) + 1
    end = next(index for index in range(start, len(lines)) if lines[index].startswith("#"))
    table = [line for line in lines[start:end] if line.startswith("|")]
    header = cells(table[0])
    if tuple(header) != COLUMNS:
        raise CatalogError(f"the catalog's columns are {header}, not {list(COLUMNS)}")
    return [dict(zip(COLUMNS, cells(line))) for line in table[2:]]


def cells(line):
    return [cell.strip() for cell in line.strip().strip("|").split("|")]


def printed(text, code, column):
    """§FS-errors.5.5.1: a cell is printed without its backticks, and names no
    specification ID and holds no link."""
    if "§" in text or "](" in text:
        raise CatalogError(f"`{code}`: its {column} cell names an ID or holds a link: {text}")
    return text.replace("`", "")


def sides(cell):
    """The version 2 and version 1 sides of a cell, the same text where it does
    not split."""
    split = SPLIT.match(cell)
    return (split.group(1), split.group(2)) if split else (cell, cell)


def absent(side, version, code, column):
    """Whether a side is `not in version N`, which only that version may say."""
    match = ABSENT.match(side)
    if match and int(match.group(1)) != version:
        raise CatalogError(f"`{code}`: its version {version} {column} says {side}")
    return bool(match)


def severity(side, version, code):
    """What one side of a Severity cell says the finding is."""
    if absent(side, version, code, "Severity"):
        return {"kind": "absent", "text": printed(side, code, "Severity")}
    if side in ("error", "warning"):
        return {"kind": "fixed", "text": side}
    if side == "none — suggestion":
        return {"kind": "suggestion", "text": "suggestion"}
    if side.startswith("none — spent "):
        return {"kind": "spent", "text": printed(side.removeprefix("none — "), code, "Severity")}
    if side.startswith("none — "):
        raise CatalogError(f"`{code}`: a Severity cell of none says neither suggestion nor spent: {side}")
    if side.startswith("by strength: "):
        return {"kind": "strength", "text": printed(side.removeprefix("by strength: "), code, "Severity")}
    return {"kind": "setting", "text": printed(side, code, "Severity")}


def clause(text, code):
    """One clause and what it is turned on by: a setting, a flag, either, or a
    condition of the tree the check itself judges."""
    if not text:
        raise CatalogError(f"`{code}`: an Enabled by cell has an empty clause")
    setting, flag = " is set" in text, " is passed" in text
    by = "either" if setting and flag else "setting" if setting else "flag" if flag else "condition"
    return {"text": printed(text, code, "Enabled by"), "by": by}


def enabled(side, version, code):
    """What one side of an Enabled by cell says turns the code on."""
    if side == "—":
        return {"kind": "always"}
    if absent(side, version, code, "Enabled by"):
        return {"kind": "absent", "text": printed(side, code, "Enabled by")}
    if side.startswith(PLANNED):
        raise CatalogError(f"`{code}`: only a whole cell is planned, not one version's side")
    body, _, remark = side.partition("; ")
    read = {"kind": "clauses", "clauses": [clause(text, code) for text in body.split(" and ")]}
    if remark:
        read["remark"] = printed(remark, code, "Enabled by")
    return read


def verbatim(text, code, column):
    """§FS-errors.5.5.1: a *Fix* side keeps its backticks, which are the message's
    own bytes, and still names no ID and holds no link."""
    if "§" in text or "](" in text:
        raise CatalogError(f"`{code}`: its {column} cell names an ID or holds a link: {text}")
    return text


def template_parts(template, code):
    """A template's text and braces in order: `("text", words)`, `("slot", name)` and
    `("fix", (code, label))`, the label `None` for a reference to a whole template,
    refusing a brace without its partner or that is neither."""
    parts, at = [], 0
    for match in BRACE.finditer(template):
        parts.append(("text", template[at:match.start()]))
        inside = match.group(1)
        reference = REFERENCE.match(inside)
        if reference:
            parts.append(("fix", reference.groups()))
        elif SLOT.match(inside):
            parts.append(("slot", inside))
        else:
            raise CatalogError(f"`{code}`: its Fix holds a brace that is neither a slot nor a "
                               f"reference: {{{inside}}}")
        at = match.end()
    parts.append(("text", template[at:]))
    if any("{" in text or "}" in text for kind, text in parts if kind == "text"):
        raise CatalogError(f"`{code}`: its Fix holds a brace without its partner: {template}")
    return parts


def fix_side(side, version, code):
    """One side of a *Fix* cell, read and its references left in place."""
    if not side:
        raise CatalogError(f"`{code}`: its Fix cell is blank")
    if absent(side, version, code, "Fix"):
        return {"kind": "absent", "text": side}
    whole = WHOLE.match(side)
    if whole:
        return {"kind": "whole", "code": whole.group(1)}
    if side.startswith(NONE):
        why = verbatim(side.removeprefix(NONE), code, "Fix")
        if not why.strip():
            raise CatalogError(f"`{code}`: its Fix is none and says no why")
        return {"kind": "none", "text": why}
    if CASES in side:
        cases = []
        for case in side.split(CASES):
            match = LABEL.match(case)
            if not match:
                raise CatalogError(f"`{code}`: a Fix case has no `<label>: `: {case}")
            label, template = match.groups()
            if any(label == known["label"] for known in cases):
                raise CatalogError(f"`{code}`: its Fix names the case `{label}` twice")
            template_parts(verbatim(template, code, "Fix"), code)
            cases.append({"label": label, "text": template})
        return {"kind": "cases", "cases": cases}
    template_parts(verbatim(side, code, "Fix"), code)
    return {"kind": "template", "text": side}


def referred(value, version, code, fixes, seen):
    """The side of `value`'s *Fix* a reference from `code` names: the same version's,
    or the other where that one is absent, refusing a code the catalog lacks or a
    reference back to one already being expanded."""
    if value not in fixes:
        raise CatalogError(f"`{code}`: its Fix refers to `{value}`, which the catalog lacks")
    if value in seen or value == code:
        raise CatalogError(f"`{code}`: its Fix refers to itself through `{value}`")
    target = fixes[value][version]
    if target["kind"] == "absent":
        target = fixes[value]["1" if version == "2" else "2"]
    return target


def expand(template, version, code, fixes, seen=()):
    """§FS-errors.5.5.1: a template with each `{fix of `<code>`}` replaced by that
    row's template on the same side, or its other side where that one is absent, and
    each `{fix of `<code>`, case `<label>`}` by that side's case so labelled."""
    out = []
    for kind, value in template_parts(template, code):
        if kind != "fix":
            out.append(value if kind == "text" else "{" + value + "}")
            continue
        other, label = value
        target = expanded(referred(other, version, code, fixes, seen), version, other,
                          fixes, seen + (code,))
        if label is None and target["kind"] != "template":
            raise CatalogError(f"`{code}`: its Fix refers to `{other}`, whose Fix is "
                               f"{target['kind']}, not one template")
        if label is not None:
            case = next((case for case in target.get("cases", []) if case["label"] == label),
                        None)
            if case is None:
                raise CatalogError(f"`{code}`: its Fix refers to `{other}`'s case `{label}`, "
                                   f"which that Fix does not name")
            out.append(case["text"])
        else:
            out.append(target["text"])
    return "".join(out)


def expanded(fix, version, code, fixes, seen=()):
    """A read *Fix* side with every reference in it expanded, a whole-side reference
    taking the side it names, cases and all."""
    if fix["kind"] == "whole":
        target = referred(fix["code"], version, code, fixes, seen)
        if target["kind"] not in ("template", "cases", "whole"):
            raise CatalogError(f"`{code}`: its Fix refers to `{fix['code']}`, whose Fix is "
                               f"{target['kind']}, not a remedy")
        return expanded(target, version, fix["code"], fixes, seen + (code,))
    if fix["kind"] == "template":
        return {"kind": "template", "text": expand(fix["text"], version, code, fixes, seen)}
    if fix["kind"] == "cases":
        return {"kind": "cases", "cases": [
            {"label": case["label"], "text": expand(case["text"], version, code, fixes, seen)}
            for case in fix["cases"]]}
    return fix


def note_side(side, version, code):
    """One side of a *Note* cell: `None` for `—` or `not in version N`, else its
    sentences as printed."""
    if side == "—" or absent(side, version, code, "Note"):
        return None
    if not side:
        raise CatalogError(f"`{code}`: its Note cell is blank, not `—`")
    return printed(side, code, "Note")


def check_section(cell, code):
    """The section a row's Check cell links, which its handle cites."""
    match = CHECK.match(cell)
    if not match:
        raise CatalogError(f"`{code}`: its Check cell is not one section link: {cell}")
    return match.group(1)


def catalog():
    """Every selectable row, read, in code order."""
    read, fixes = [], {}
    for row in rows():
        match = CODE.match(row["Code"])
        if not match:
            raise CatalogError(f"a Code cell is not one code: {row['Code']}")
        fixed = sides(row["Fix"])
        fixes[match.group(1)] = {version: fix_side(fixed[index], int(version), match.group(1))
                                 for version, index in (("2", 0), ("1", 1))}
    for row in rows():
        code = CODE.match(row["Code"]).group(1)
        if row["Enabled by"].startswith(PLANNED):
            continue
        meaning = printed(row["Meaning"], code, "Meaning")
        if not meaning:
            raise CatalogError(f"`{code}`: no Meaning")
        entry = {"code": code, "meaning": meaning}
        ramp = row["Ramp"]
        entry["ramp"] = None if ramp == "—" else printed(ramp, code, "Ramp")
        severities = sides(row["Severity"])
        enablements = sides(row["Enabled by"])
        notes = sides(row["Note"])
        for version, index in (("2", 0), ("1", 1)):
            entry[version] = {
                "severity": severity(severities[index], int(version), code),
                "enabled": enabled(enablements[index], int(version), code),
                "fix": expanded(fixes[code][version], version, code, fixes),
                "note": note_side(notes[index], int(version), code),
            }
        counts = {len(entry[v]["enabled"].get("clauses", [])) for v in "21"
                  if entry[v]["enabled"]["kind"] == "clauses"}
        if len(counts) > 1:
            raise CatalogError(f"`{code}`: its versions name different numbers of clauses")
        entry["check"] = check_section(row["Check"], code)
        read.append(entry)
    codes = [entry["code"] for entry in read]
    if codes != sorted(codes):
        raise CatalogError("the catalog is not sorted by code")
    if len(set(codes)) != len(codes):
        raise CatalogError("the catalog lists a code twice")
    return read


def render_asset(read):
    shipped = [{key: value for key, value in entry.items() if key != "check"} for entry in read]
    return json.dumps(shipped, ensure_ascii=False, indent=1) + "\n"


def constant(code):
    return code.upper().replace("-", "_")


def render_handles(read):
    codes = [entry["code"] for entry in read]
    out = [
        "//! The `check` code handles, rendered from the §FS-errors.5.5 catalog by",
        "//! `scripts/render_check_codes.py`: edit the catalog, then run the script.",
        "//!",
        "//! A finding is built only from a handle below, so a code the catalog does not",
        "//! have cannot be reported (§AR-system.2.2). What each code means and what turns",
        "//! it on is the rendered catalog's, read by `catalog.rs` (§AR-config.9.1).",
        "",
        "/// One code of the §FS-errors.5.5 catalog. Its field is the model's own, so the",
        "/// codes are the constants below and the run's three cautions that are not",
        "/// check findings, which `report.rs` names beside the record that carries them.",
        "#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]",
        "pub struct Code(pub(super) &'static str);",
        "",
        "impl Code {",
    ]
    for entry in read:
        out.append(f"    /// §{entry['check']}")
        line = f'    pub const {constant(entry["code"])}: Code = Code("{entry["code"]}");'
        if len(line) > 100:
            out.append(f"    pub const {constant(entry['code'])}: Code =")
            out.append(f'        Code("{entry["code"]}");')
        else:
            out.append(line)
    out.append("")
    out.append("    /// Every selectable code, sorted: the selector vocabulary of §FS-check.1.")
    out.append("    pub const ALL: &'static [Code] = &[")
    for code in codes:
        out.append(f"        Code::{constant(code)},")
    out.append("    ];")
    out.append("")
    out.append("    pub const fn as_str(self) -> &'static str {")
    out.append("        self.0")
    out.append("    }")
    out.append("")
    out.append("    /// The handle a selector or a query spells, or `None` for a code the catalog")
    out.append("    /// does not have.")
    out.append("    pub fn named(code: &str) -> Option<Code> {")
    out.append("        Code::ALL")
    out.append("            .binary_search_by(|known| known.0.cmp(code))")
    out.append("            .ok()")
    out.append("            .map(|index| Code::ALL[index])")
    out.append("    }")
    out.append("")
    out.append("    /// The code a citation outside the default scope reports in place of this one")
    out.append("    /// under `--full`, where it has one (§FS-check.3.14).")
    out.append("    pub fn out_of_scope(self) -> Option<Code> {")
    out.append("        match self.0 {")
    for code in codes:
        if OUT_OF_SCOPE + code in codes:
            out.append(f'            "{code}" => Some(Code::{constant(OUT_OF_SCOPE + code)}),')
    out.append("            _ => None,")
    out.append("        }")
    out.append("    }")
    out.append("}")
    out.append("")
    out.append("/// A code reads as the name it is printed by, so a finding can be asked about by")
    out.append("/// name; only a constant above makes one.")
    out.append("impl PartialEq<&str> for Code {")
    out.append("    fn eq(&self, other: &&str) -> bool {")
    out.append("        self.0 == *other")
    out.append("    }")
    out.append("}")
    return "\n".join(out) + "\n"


def slots(fix):
    """The slots of a read, expanded *Fix* side's templates, by case: `{label: [slot]}`,
    the label `None` for a single template."""
    templates = ([(None, fix["text"])] if fix["kind"] == "template" else
                 [(case["label"], case["text"]) for case in fix.get("cases", [])])
    return {label: [name for kind, name in template_parts(text, "") if kind == "slot"]
            for label, text in templates}


def fillers(entry, codes):
    """What `remedies.rs` renders for one code: `(name, label, version, slots)` per
    template or case its sides hold, the version `None` where both sides say one thing.
    An `out-of-scope-` code has none: its message is its in-scope code's, filled by
    that code's filler and then reported under the other code (§FS-check.3.14)."""
    code = entry["code"]
    if code.startswith(OUT_OF_SCOPE) and code.removeprefix(OUT_OF_SCOPE) in codes:
        return []
    present = [v for v in "21" if entry[v]["fix"]["kind"] in ("template", "cases")]
    if not present:
        return []
    kinds = {entry[v]["fix"]["kind"] for v in present}
    if len(kinds) > 1:
        # One version's template and the other's cases: the cases are what a message
        # picks from, and the side with one template answers each of them.
        present = [v for v in present if entry[v]["fix"]["kind"] == "cases"]
    by_side = [slots(entry[v]["fix"]) for v in present]
    versioned = len(kinds) > 1 or len(present) > 1 and entry["2"]["fix"] != entry["1"]["fix"]
    labels = []
    for side in by_side:
        labels.extend(label for label in side if label not in labels)
    out = []
    for label in labels:
        names = []
        for side in by_side:
            names.extend(name for name in side.get(label, []) if name not in names)
        name = code.replace("-", "_") + ("" if label is None else "_" + label.replace("-", "_"))
        out.append((name, label, None if versioned else int(present[0]), names))
    return out


def argument(slot):
    name = slot.replace(" ", "_")
    return name + "_" if name in KEYWORDS else name


def render_remedies(read):
    out = [
        "//! The *Fix* fillers, rendered from the §FS-errors.5.5 catalog by",
        "//! `scripts/render_check_codes.py`: edit the catalog, then run the script.",
        "//!",
        "//! A message's remedy is built only by its code's filler below, which fills the",
        "//! row's template from the embedded catalog, so no remedy is worded in the",
        "//! checker and a misspelt slot or case does not compile (§AR-config.9.1).",
        "",
        "use std::fmt::Display;",
        "",
        "use super::catalog::fill;",
        "use super::codes::Code;",
    ]
    codes = {entry["code"] for entry in read}
    for entry in read:
        for name, label, version, names in fillers(entry, codes):
            case = "None" if label is None else f'Some("{label}")'
            params = ([] if version is not None else ["version: u32"]) + [
                f"{argument(slot)}: impl Display" for slot in names]
            pairs = ", ".join(f'("{slot}", &{argument(slot)})' for slot in names)
            which = f"`{entry['code']}`'s *Fix*" + ("" if label is None else f", case `{label}`")
            out.append("")
            out.append(f"/// {which} (§{entry['check']}).")
            out.append(f"pub(crate) fn {name}({', '.join(params)}) -> String {{")
            out.append(f"    let slots: &[(&str, &dyn Display)] = &[{pairs}];")
            at = "version" if version is None else str(version)
            out.append(f"    fill(Code::{constant(entry['code'])}, {at}, {case}, slots)")
            out.append("}")
    return "\n".join(out) + "\n"


def render():
    read = catalog()
    return {ASSET: render_asset(read), HANDLES: render_handles(read),
            REMEDIES: render_remedies(read)}


def main(argv):
    try:
        rendered = render()
    except CatalogError as error:
        print(f"{PAGE.relative_to(ROOT)}: {error}", file=sys.stderr)
        return 1
    if "--check" in argv:
        stale = [path for path, text in rendered.items()
                 if not path.is_file() or path.read_text(encoding="utf-8") != text]
        for path in stale:
            print(f"{path.relative_to(ROOT)} is stale: run python scripts/render_check_codes.py",
                  file=sys.stderr)
        return 1 if stale else 0
    for path, text in rendered.items():
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
