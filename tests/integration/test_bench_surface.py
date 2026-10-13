"""§AR-benchmarks.1.6 — the performance-surface inventory,
`crates/grund-cli/benches/surface.json`, gives every command, every flag its help page
prints and every designated performance-sensitive mode (§AR-benchmarks.1.6.1) either
scenarios from the catalog of §AR-benchmarks.1.2 or a waiver with a reason.

Static reads only: the commands come from `SUBCOMMANDS`, the flags from the help pages'
source, the modes from the spec. No build and no Valgrind, so it runs in pre-commit.
"""

import json
import re
import unittest
from pathlib import Path


REPO = Path(__file__).resolve().parents[2]
SURFACE = REPO / "crates/grund-cli/benches/surface.json"
CATALOG = REPO / "crates/grund-cli/benches/scenarios.json"
LIB = REPO / "crates/grund-cli/src/lib.rs"
CLI_SRC = REPO / "crates/grund-cli/src"
HELP = CLI_SRC / "cli_help.rs"
AR_BENCHMARKS = REPO / "docs/architecture/AR-benchmarks.md"
EDIT = "edit crates/grund-cli/benches/surface.json (§AR-benchmarks.1.6)"

STRING = re.compile(r'"(?:\\.|[^"\\])*"')
FLAG = re.compile(r"--[a-z][a-z0-9-]*")
CALL = re.compile(r"\b(print_[a-z_]+)\(")
# §AR-benchmarks.1.6: every command takes `--help`, and it scans nothing.
NOT_INVENTORIED = {"--help"}


def subcommands():
    """The `SUBCOMMANDS` table, the list the CLI dispatches on."""
    table = re.search(r"const SUBCOMMANDS: &\[&str\] = &\[(.*?)\];", LIB.read_text(encoding="utf-8"),
                      re.S)
    return [literal[1:-1] for literal in STRING.findall(table.group(1))]


def block(text, start):
    """The `{ … }` block opening at `text[start]`, skipping string, char and comment text."""
    assert text[start] == "{", text[start:start + 40]
    depth, i = 0, start
    while i < len(text):
        c = text[i]
        if c == '"':
            i = STRING.match(text, i).end()
            continue
        if text.startswith("//", i):
            i = text.index("\n", i)
            continue
        if c == "'" and re.match(r"'(\\.|[^\\'])'", text[i:i + 4]):
            i = re.match(r"'(\\.|[^\\'])'", text[i:i + 4]).end() + i
            continue
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return text[start:i + 1]
        i += 1
    raise AssertionError("unbalanced block")


def functions():
    """Every `fn print_*` in the help sources, by name, to its parameter names and body."""
    found = {}
    for path in sorted(CLI_SRC.glob("cli_help*.rs")):
        text = path.read_text(encoding="utf-8")
        for match in re.finditer(r"\bfn (print_[a-z_]+)\(([^)]*)\)\s*\{", text):
            params = [p.split(":")[0].strip() for p in match.group(2).split(",") if p.strip()]
            found[match.group(1)] = (params, block(text, match.end() - 1))
    return found


def arguments(text, start):
    """The comma-separated arguments of the call whose `(` is at `text[start]`."""
    args, depth, current, i = [], 0, "", start + 1
    while True:
        c = text[i]
        if c == '"':
            end = STRING.match(text, i).end()
            current, i = current + text[i:end], end
            continue
        if c in "([{":
            depth += 1
        elif c in ")]}":
            if depth == 0:
                return args + [current.strip()] if current.strip() else args
            depth -= 1
        elif c == "," and depth == 0:
            args.append(current.strip())
            current, i = "", i + 1
            continue
        current += c
        i += 1


def printed(body, params, args):
    """`body` without the `if <param> { … }` blocks a `false` argument turns off."""
    for name, value in zip(params, args):
        if value != "false":
            continue
        for match in reversed(list(re.finditer(r"\bif " + re.escape(name) + r"\s*\{", body))):
            end = match.start() + len(match.group(0)) - 1
            body = body[:match.start()] + body[end + len(block(body, end)):]
    return body


def help_arm(command):
    """The source of `print_subcommand_help`'s arm for `command`, or None."""
    body = functions()["print_subcommand_help"][1]
    match = re.search(r'^\s*"' + re.escape(command) + r'"\s*=>\s*', body, re.M)
    if match is None:
        return None
    if body[match.end()] == "{":
        return block(body, match.end())
    return body[match.end():body.index("\n", match.end())]


def help_flags(command):
    """Every `--flag` the help page of `command` prints, through the helpers it calls."""
    source = help_arm(command)
    if source is None:
        return None
    bodies, flags, pending = functions(), set(), [source]
    while pending:
        text = pending.pop()
        for literal in STRING.findall(text):
            flags.update(FLAG.findall(literal))
        for match in CALL.finditer(text):
            if match.group(1) in bodies:
                params, body = bodies[match.group(1)]
                pending.append(printed(body, params, arguments(text, match.end() - 1)))
    return flags - NOT_INVENTORIED


def designated_modes():
    """The mode keys §AR-benchmarks.1.6.1 lists, in order."""
    text = AR_BENCHMARKS.read_text(encoding="utf-8")
    section = re.search(r"^#### 1\.6\.1 .*?$(.*?)^#", text, re.M | re.S)
    assert section is not None, "AR-benchmarks has no section 1.6.1"
    return re.findall(r"^- `([a-z][a-z0-9-]*)`:", section.group(1), re.M)


def surface():
    return json.loads(SURFACE.read_text(encoding="utf-8"))


def catalog_rows():
    return {row["name"]: row for row in json.loads(CATALOG.read_text(encoding="utf-8"))["scenarios"]}


def entry_problems(where, entry, rows, command, flag=None):
    """What makes `entry` invalid under §AR-benchmarks.1.6, as messages."""
    if not isinstance(entry, dict):
        return [f"{where}: an entry is an object with `scenarios` or `waiver`; {EDIT}"]
    has_scenarios, has_waiver = "scenarios" in entry, "waiver" in entry
    if has_scenarios == has_waiver:
        return [f"{where}: needs exactly one of `scenarios` and `waiver`; {EDIT}"]
    if has_waiver:
        reason = entry["waiver"]
        if not isinstance(reason, str) or not reason.strip():
            return [f"{where}: the waiver has no reason; {EDIT}"]
        return []
    names = entry["scenarios"]
    if not isinstance(names, list) or not names:
        return [f"{where}: `scenarios` is a non-empty list of catalog names; {EDIT}"]
    problems = []
    for name in names:
        row = rows.get(name)
        if row is None:
            problems.append(f"{where}: scenario {name!r} is not in benches/scenarios.json; {EDIT}")
        elif command is not None and row["args"][0] != command:
            problems.append(f"{where}: scenario {name!r} runs `{row['args'][0]}`, not `{command}`;"
                            f" {EDIT}")
        elif flag is not None and flag not in row["args"]:
            problems.append(f"{where}: scenario {name!r} does not pass {flag}; {EDIT}")
    return problems


class TheSurfaceIsReadFromTheCli(unittest.TestCase):
    """§AR-benchmarks.1.6: the commands and flags come from the CLI definition."""

    def test_every_subcommand_has_a_help_page_to_read_flags_from(self):
        missing = [command for command in subcommands() if help_arm(command) is None]
        self.assertEqual(missing, [], "print_subcommand_help has no arm for these SUBCOMMANDS")

    def test_the_flag_reader_sees_the_flags_the_help_pages_print(self):
        self.assertTrue({"--brief", "--full", "--toc", "--path-base"} <= help_flags("show"))
        self.assertTrue({"--watch", "--suggestions", "--format", "--path-base"}
                        <= help_flags("check"))
        self.assertTrue({"--check", "--write"} <= help_flags("fmt"))
        self.assertNotIn("--help", help_flags("check"))
        # `print_run_flag_help(.., false)` does not print its `--format` line.
        self.assertNotIn("--format", help_flags("fmt"))
        self.assertNotIn("--format", help_flags("config"))

    def test_the_spec_designates_the_modes(self):
        # The modes the inventory started with; adding one is a spec edit, not a test edit.
        self.assertLessEqual({"parallel-scan", "workspace", "lsp", "python", "node"},
                             set(designated_modes()))


class TheInventoryCoversTheSurface(unittest.TestCase):
    """§AR-benchmarks.1.6: every item names scenarios or carries a waiver, and nothing is stale."""

    def setUp(self):
        self.assertTrue(SURFACE.is_file(),
                        f"no performance-surface inventory at {SURFACE.relative_to(REPO)}"
                        " (§AR-benchmarks.1.6)")
        self.surface = surface()
        self.rows = catalog_rows()
        self.commands = self.surface.get("commands", {})
        self.modes = self.surface.get("modes", {})

    def test_the_inventory_has_commands_and_modes(self):
        self.assertEqual(sorted(self.surface), ["commands", "modes"])

    def test_every_subcommand_has_an_entry(self):
        missing = [command for command in subcommands() if command not in self.commands]
        self.assertEqual(missing, [], f"commands with no scenario and no waiver; {EDIT}")

    def test_every_flag_is_covered(self):
        uncovered = []
        for command in subcommands():
            entry = self.commands.get(command, {})
            if "waiver" in entry:
                continue  # §AR-benchmarks.1.6: a waived command waives its flags with it.
            carried = {arg for row in self.rows.values() if row["args"][0] == command
                       for arg in row["args"]}
            listed = entry.get("flags", {})
            uncovered += [f"{command} {flag}" for flag in sorted(help_flags(command) or ())
                          if flag not in carried and flag not in listed]
        self.assertEqual(uncovered, [], f"flags with no scenario and no waiver; {EDIT}")

    def test_every_designated_mode_has_an_entry(self):
        missing = [mode for mode in designated_modes() if mode not in self.modes]
        self.assertEqual(missing, [], f"modes with no scenario and no waiver; {EDIT}")

    def test_every_entry_is_valid(self):
        problems = []
        for command, entry in sorted(self.commands.items()):
            body = {key: value for key, value in entry.items() if key != "flags"} \
                if isinstance(entry, dict) else entry
            problems += entry_problems(f"command {command}", body, self.rows, command)
            for flag, flag_entry in sorted(entry.get("flags", {}).items()):
                problems += entry_problems(f"flag {command} {flag}", flag_entry, self.rows,
                                           command, flag)
        for mode, entry in sorted(self.modes.items()):
            problems += entry_problems(f"mode {mode}", entry, self.rows, None)
        self.assertEqual(problems, [])

    def test_no_entry_is_stale(self):
        known = set(subcommands())
        stale = [f"command {command}" for command in sorted(self.commands) if command not in known]
        for command in sorted(set(self.commands) & known):
            flags = help_flags(command) or set()
            stale += [f"flag {command} {flag}"
                      for flag in sorted(self.commands[command].get("flags", {}))
                      if flag not in flags]
        designated = set(designated_modes())
        stale += [f"mode {mode}" for mode in sorted(self.modes) if mode not in designated]
        self.assertEqual(stale, [], f"entries for items that no longer exist; {EDIT}")


if __name__ == "__main__":
    unittest.main()
