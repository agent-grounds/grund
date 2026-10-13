#!/usr/bin/env python3
"""Generate the conformant repos the instruction benchmarks scan.

§AR-benchmarks.1.1: one generator makes every fixture. With no option it writes the plain
fixture of `--files` declarations; §AR-benchmarks.1.2: `--feature` adds the one feature a
fixture isolates, and `--members` makes a workspace of plain member projects.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
from pathlib import Path


# §AR-benchmarks.1.1: raise this whenever the generated output changes shape, so a
# changed workload starts a new series (§AR-benchmarks.7.2).
SCHEMA_VERSION = 1
MANIFEST = "manifest.json"
DEFAULT_FILE_COUNT = 10_000
DEFAULT_COMPONENT_COUNT = 100
# §AR-benchmarks.1.2: the features a fixture can isolate, each in a fixture of its own.
FEATURES = ("rules", "named-sections", "values", "external-facts", "cross-refs")
# §FS-section-lock: the fingerprints of the two cited relative sections every
# named-sections declaration repeats verbatim, as `grund fmt --write` records them.
SECTION_FINGERPRINTS = {"format.1": "8f332b6fe87bb346", "parse.1": "be879e46c73e3d05"}
# §FS-rules: the committed rule declarations the rules fixture holds every FS to.
RULES = {
    "RULE-summary": "Each FS must have exactly one summary chapter.",
    "RULE-summary-cites": "The summary chapter of each FS must cite at least one FS.",
    "RULE-requirements": "Each FS must have at most 1 requirements chapter.",
}


def id_for(index: int, feature: str | None = None) -> str:
    if feature == "cross-refs":
        return f"FS-feature-{index:05d}"
    return f"FS-{index:05d}-feature-{index:05d}"


def canonical_id(index: int) -> str:
    """The ID as grund writes it back: `{number}` at its default width of three."""
    return f"FS-{index:03d}-feature-{index:05d}"


def fixture_config(citations: bool = False, feature: str | None = None,
                   project: str | None = None) -> str:
    base = """grund_config_version = 1

[reference]
strict = true

[scan]
include = ["docs"]
exclude = ["target", "node_modules", ".git", "dist", "build", ".venv"]
extensions = ["md"]
respect_gitignore = false
"""
    if project is not None:
        base = base.replace("grund_config_version = 1\n",
                            f'grund_config_version = 1\nproject_name = "{project}"\n', 1)
    base += FEATURE_CONFIG.get(feature, "")
    if not citations:
        return base
    # A direction ruleset that exercises the classify + obligation + prohibition
    # passes without producing errors (each generated FS cites the next FS, so
    # the FS->FS obligation is met; no AR exists, so the prohibition never fires)
    # — `grund check` still exits 0 (§FS-config.3.9).
    return base + """
[citations]
default = "may"

[citations.FS]
should = ["FS"]
must-not = ["AR"]
"""


FEATURE_CONFIG = {
    "rules": """
[id]
named_sections = true

[[kinds]]
kind = "FS"
folder = "docs/functional-spec"

[[kinds]]
kind = "RULE"
folder = "docs/rules"
index = false
format = "{kind}-{slug}"
rules = true
""",
    "named-sections": """
[id]
named_sections = true
""",
    "values": """
[id]
named_sections = true

[[kinds]]
kind = "FS"
folder = "docs/functional-spec"
value_chapter = "values"
""",
    "external-facts": """
[[kinds]]
kind = "FS"
folder = "docs/functional-spec"

[[kinds]]
kind = "TICKET"
file = "docs/tickets.md"
format = "{kind}-{number}"
resolve = "should"
fetch = "scripts/fetch-ticket"
""",
    "cross-refs": """
[id]
format = "{kind}-{slug}"
""",
}


def anchor(ident: str, index: int) -> str:
    """GitHub's slug of a cross-refs declaration's heading, as `grund fmt` links it."""
    return f"{ident.lower()}-feature-{index:05d}"


def link(ident: str, index: int, path: str) -> str:
    return f"[§{ident}]({path}#{anchor(ident, index)})"


def tickets(file_count: int) -> int:
    return max(1, file_count // 10)


def declaration_body(index: int, file_count: int, component_count: int,
                     feature: str | None = None) -> str:
    ident = id_for(index, feature)
    following = index + 1 if index < file_count else 1
    next_ident = id_for(following, feature)
    component = (index - 1) % component_count
    heading = f"# {ident}: Feature {index:05d}\n\n"
    belongs = f"Feature {index:05d} belongs to synthetic component {component:03d} "
    if feature == "rules":
        return (heading + belongs + "and is held to the committed rules.\n\n"
                f"## summary: Summary\n\nIt cites §{next_ident} so every declaration is used.\n\n"
                "## requirements: Requirements\n\nIt holds what it says.\n")
    if feature == "named-sections":
        return (heading + belongs + f"and reads its input (§{next_ident}.format.1) and parses "
                f"it (§{next_ident}.parse.1).\n\n"
                "## format: Formatting\n\n### .1: Username\n\nTrim surrounding whitespace.\n\n"
                "### .2: Password\n\nKeep every character as typed.\n\n"
                "## parse: Parsing\n\n### .1: Tokens\n\nSplit on commas.\n")
    if feature == "values":
        return (heading + belongs + f"and holds its limit to `{index}` "
                f"(§{ident}.values.limit.1) and its peer's to `{following}` "
                f"(§{next_ident}.values.limit.1).\n\n"
                f"## values: Values\n\n### values.limit: Limit\n\n"
                f"#### values.limit.1: {index}\n#### values.limit.2: ms\n")
    if feature == "external-facts":
        ticket = (index - 1) % tickets(file_count) + 1
        return (heading + belongs + f"and cites §{next_ident} so every declaration is used; "
                f"it is tracked in §TICKET-{ticket}.\n")
    if feature == "cross-refs":
        target = f"../component-{(following - 1) % component_count:03d}/{next_ident}.md"
        return (heading + belongs + f"and cites {link(next_ident, following, target)} so every "
                "declaration is used.\n")
    return heading + belongs + f"and cites §{next_ident} so every declaration is used.\n"


def feature_files(root: Path, file_count: int, feature: str | None) -> None:
    """The files beside the declarations that the feature needs to scan clean."""
    if feature == "rules":
        (root / "docs" / "rules").mkdir(parents=True)
        for ident, sentence in RULES.items():
            (root / "docs" / "rules" / f"{ident}.md").write_text(
                f"# {ident}: {sentence}\n\nA rule of the benchmark fixture.\n", encoding="utf-8")
        (root / "docs" / "conventions.md").write_text(
            "# Conventions\n\n" + "".join(f"- §{ident}\n" for ident in RULES), encoding="utf-8")
    elif feature == "named-sections":
        locks = root / ".grund" / "sections"
        locks.mkdir(parents=True)
        for index in range(1, file_count + 1):
            ident = canonical_id(index)
            (locks / f"{ident}.lock").write_text("".join(
                f"{ident}.{path} {fingerprint}\n"
                for path, fingerprint in SECTION_FINGERPRINTS.items()), encoding="utf-8")
    elif feature == "external-facts":
        # §FS-fetch.4: the snapshots `grund fetch` saved; the fetcher never runs on a scan.
        (root / "scripts").mkdir()
        (root / "scripts" / "fetch-ticket").write_text("#!/bin/sh\nexit 99\n", encoding="utf-8")
        (root / "docs" / "tickets.md").write_text("# Tickets\n" + "".join(
            f"\n## TICKET-{number}: Ticket {number}\n\nThe snapshot of ticket {number}.\n"
            for number in range(1, tickets(file_count) + 1)), encoding="utf-8")


def fixture_digest(root: Path) -> str:
    """§AR-benchmarks.1.1: SHA-256 over the sorted relative paths and their contents,
    the manifest itself left out."""
    digest = hashlib.sha256()
    files = sorted(
        (path.relative_to(root).as_posix(), path)
        for path in root.rglob("*")
        if path.is_file() and path.relative_to(root).as_posix() != MANIFEST
    )
    for relative, path in files:
        data = path.read_bytes()
        digest.update(f"{relative}\0{len(data)}\0".encode("utf-8"))
        digest.update(data)
    return digest.hexdigest()


def write_manifest(root: Path) -> None:
    """The manifest sits at the fixture root, outside the scanned `docs/` tree."""
    manifest = {"schema_version": SCHEMA_VERSION, "digest": fixture_digest(root)}
    (root / MANIFEST).write_text(json.dumps(manifest, sort_keys=True) + "\n", encoding="utf-8")


def generate_fixture(
    root: Path, file_count: int, component_count: int, citations: bool = False,
    feature: str | None = None, members: int = 0,
) -> None:
    if file_count < 1:
        raise ValueError("--files must be at least 1")
    if component_count < 1:
        raise ValueError("--components must be at least 1")
    if feature is not None and feature not in FEATURES:
        raise ValueError(f"--feature must be one of {', '.join(FEATURES)}")

    if root.exists():
        shutil.rmtree(root)

    if members:
        generate_workspace(root, file_count, component_count, members)
    else:
        generate_project(root, file_count, component_count, citations, feature)
    write_manifest(root)


def generate_workspace(root: Path, file_count: int, component_count: int, members: int) -> None:
    """§FS-workspace: plain member projects that cite into each other by alias."""
    if members < 2 or file_count < members:
        raise ValueError("--members must be at least 2 and at most --files")
    names = [f"m{number:02d}" for number in range(1, members + 1)]
    root.mkdir(parents=True)
    listed = ", ".join(f'"members/{name}"' for name in names)
    (root / "grund.toml").write_text(
        'grund_config_version = 1\nproject_name = "bench"\n\n'
        f"[workspace]\nmembers = [{listed}]\ninclude_root = false\n", encoding="utf-8")
    for number, name in enumerate(names):
        member = root / "members" / name
        generate_project(member, file_count // members, component_count, project=name)
        peer = names[(number + 1) % members]
        first = member / "docs" / "functional-spec" / "component-000" / f"{id_for(1)}.md"
        first.write_text(first.read_text(encoding="utf-8")
                         + f"\nIt leans on §{peer}/{id_for(1)} in the next member.\n",
                         encoding="utf-8")


def generate_project(root: Path, file_count: int, component_count: int,
                     citations: bool = False, feature: str | None = None,
                     project: str | None = None) -> None:
    root.mkdir(parents=True)
    (root / "grund.toml").write_text(fixture_config(citations, feature, project),
                                     encoding="utf-8")

    # §FS-check.3.18: the FS kind's default index requires every declaration to
    # be listed as a link, or `grund check` errors on this fixture instead of
    # measuring it — so every generated declaration gets a matching entry here.
    index_lines = ["# Functional spec\n\n"]

    for index in range(1, file_count + 1):
        component = (index - 1) % component_count
        directory = root / "docs" / "functional-spec" / f"component-{component:03d}"
        directory.mkdir(parents=True, exist_ok=True)
        ident = id_for(index, feature)
        path = directory / f"{ident}.md"
        path.write_text(
            declaration_body(index, file_count, component_count, feature),
            encoding="utf-8",
        )
        target = f"component-{component:03d}/{ident}.md"
        if feature == "cross-refs":
            index_lines.append(f"- {link(ident, index, target)}\n")
        else:
            index_lines.append(f"- [§{ident}]({target})\n")

    index_path = root / "docs" / "functional-spec" / "README.md"
    index_path.write_text("".join(index_lines), encoding="utf-8")
    feature_files(root, file_count, feature)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root",
        type=Path,
        default=Path("target/bench-fixtures/large-conformant-repo"),
        help="fixture root to replace and regenerate",
    )
    parser.add_argument(
        "--files",
        type=int,
        default=DEFAULT_FILE_COUNT,
        help="number of Markdown declaration files to generate",
    )
    parser.add_argument(
        "--components",
        type=int,
        default=DEFAULT_COMPONENT_COUNT,
        help="number of component directories to spread files across",
    )
    parser.add_argument(
        "--citations",
        action="store_true",
        help="emit a [citations] direction ruleset so the fixture exercises the "
        "citation-direction checks",
    )
    parser.add_argument(
        "--feature",
        choices=FEATURES,
        help="the one feature the fixture isolates (§AR-benchmarks.1.2)",
    )
    parser.add_argument(
        "--members",
        type=int,
        default=0,
        help="make a workspace of this many plain member projects sharing --files",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    generate_fixture(args.root, args.files, args.components, args.citations, args.feature,
                     args.members)
    print(f"generated {args.files} files under {args.root}")


if __name__ == "__main__":
    main()
