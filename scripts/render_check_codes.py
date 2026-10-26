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
  (§AR-system.2.2).

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

HEADING = re.compile(r"^### 5\.5 ")
COLUMNS = ("Code", "Meaning", "Severity", "Ramp", "Enabled by", "Fix", "Note", "Check")
CODE = re.compile(r"^`([a-z][a-z0-9-]*)`$")
SPLIT = re.compile(r"^v2: (.*) · v1: (.*)$")
ABSENT = re.compile(r"^not in version ([12])$")
CHECK = re.compile(r"^\[§([A-Za-z0-9.-]+)\]\([^)]*\)$")
PLANNED = "planned — "
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


def check_section(cell, code):
    """The section a row's Check cell links, which its handle cites."""
    match = CHECK.match(cell)
    if not match:
        raise CatalogError(f"`{code}`: its Check cell is not one section link: {cell}")
    return match.group(1)


def catalog():
    """Every selectable row, read, in code order."""
    read = []
    for row in rows():
        match = CODE.match(row["Code"])
        if not match:
            raise CatalogError(f"a Code cell is not one code: {row['Code']}")
        code = match.group(1)
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
        for version, index in (("2", 0), ("1", 1)):
            entry[version] = {
                "severity": severity(severities[index], int(version), code),
                "enabled": enabled(enablements[index], int(version), code),
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


def render():
    read = catalog()
    return {ASSET: render_asset(read), HANDLES: render_handles(read)}


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
