#!/usr/bin/env python3
"""Render the `grund.toml` reference page from the JSON Schema (§FS-config.4).

The schema in `schemas/grund.schema.json` is the one place the keys are written
down for people and editors; `docs/user-facing/configuration.md` is rendered
from it, so the two cannot drift. So is the copy `grund config explain` words
each key from, which is the schema with its specification metadata removed
(§AR-config.9.1). Edit the schema, then run this script:

    python scripts/render_config_reference.py          # rewrite the page and the copy
    python scripts/render_config_reference.py --check  # exit 1 if either is stale
"""

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = ROOT / "schemas" / "grund.schema.json"
PAGE = ROOT / "docs" / "user-facing" / "configuration.md"
PROJECTION = ROOT / "crates" / "grund-core" / "assets" / "explain" / "grund.schema.json"

# What points into the specification, which an installed binary cannot open
# (§FS-config-explain.3): the keys that carry it, and a description that cites
# an ID or links anywhere. `§alias/<ID>` spells a citation and cites nothing.
SPEC_METADATA = ("x-grund-spec", "x-grund-note")
NAMES_A_SPEC = re.compile(r"§[A-Z]|\]\(")
SCHEMA_URL = "https://raw.githubusercontent.com/agent-grounds/grund/main/schemas/grund.schema.json"

INTRO = f"""<!-- Rendered from schemas/grund.schema.json by scripts/render_config_reference.py: edit the schema, then run the script. -->

# The `grund.toml` reference

Every key `grund.toml` accepts, table by table: first version 1, which a file
without `grund_config_version` is read as and keeps loading as, then version 2,
which a file selects with `grund_config_version = 2` and which `grund init` writes,
holding only what differs from a default
([§FS-init.2.4.3](../functional-spec/FS-init.md#243-every-written-key-differs-from-its-default)). A key you leave out
takes the default shown. `grund config validate` checks a file against these keys and
`grund config show` prints the configuration in effect, defaults included, in the
file's own version; for version 2 its tables keyed by a kind follow the order the
file writes its `[schema.kinds]` rows
([§FS-config.4.2.4](../functional-spec/FS-config.md#424-the-order-show-prints-in)).
`grund config explain '[TABLE] key'` gives one key's entry from the grund you run,
in your file's version, with no page needed ([config explain](explain.md)).

## Editor support

The same keys are published as a JSON Schema,
[`schemas/grund.schema.json`](../../schemas/grund.schema.json), so a TOML language
server can complete and check them as you type. With [Taplo](https://taplo.tamasfe.dev/),
which is also what the Even Better TOML extension for VS Code runs, name it on the first
line of `grund.toml`:

```toml
#:schema {SCHEMA_URL}
```

or once for every project, in your `.taplo.toml`:

```toml
[[rule]]
include = ["**/grund.toml"]

[rule.schema]
path = "{SCHEMA_URL}"
```

Replace `main` with the tag of the `grund` you run, such as `v0.17.0`, to match its
keys exactly. The schema knows each key's type, values and default; the rules that
relate keys to each other, and to the files on disk, are `grund config validate`'s.
"""


def resolve(schema, node):
    """Follow a local `$ref` to its definition."""
    while "$ref" in node:
        name = node["$ref"].rsplit("/", 1)[-1]
        node = schema["definitions"][name]
    return node


def toml_value(value):
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, str):
        return json.dumps(value, ensure_ascii=False)
    if isinstance(value, list):
        return "[" + ", ".join(toml_value(v) for v in value) + "]"
    return str(value)


def type_of(node):
    if node.get("not") == {}:
        return "refused"
    if "anyOf" in node:
        return " or ".join(type_of(alt) if "const" not in alt else f"`{toml_value(alt['const'])}`"
                           for alt in node["anyOf"])
    kind = node.get("type")
    if "enum" in node:
        return " \\| ".join(f"`{toml_value(v)}`" for v in node["enum"])
    if kind == "integer" and "minimum" in node and "maximum" in node:
        return f"integer, {node['minimum']}–{node['maximum']}"
    if kind == "integer" and "minimum" in node:
        return f"integer, at least {node['minimum']}"
    if kind == "array":
        items = node.get("items", {})
        if "enum" in items:
            return "array of " + " \\| ".join(f"`{toml_value(v)}`" for v in items["enum"])
        return f"array of {items.get('type', 'values')}s"
    if kind == "object" and node.get("x-grund-inline"):
        return "inline table `{ " + ", ".join(node["properties"]) + " }`"
    return kind or "value"


def default_of(node):
    if "default" in node:
        return f"`{toml_value(node['default'])}`"
    return node.get("x-grund-default", "—").replace("|", "\\|")


def cite(node, prefix="../functional-spec/"):
    spec = node.get("x-grund-spec")
    if not spec:
        return ""
    href = prefix + spec["href"].removeprefix("docs/functional-spec/")
    return f" [§{spec['id']}]({href})"


def cell(text):
    return text.replace("|", "\\|").replace("\n", " ")


def is_table(node):
    return node.get("type") == "object" and not node.get("x-grund-inline") or node.get("type") == "array" and \
        node.get("items", {}).get("type") == "object"


def render_table(schema, header, node, out, seen):
    """Emit one TOML table's keys and notes, then every table nested under it."""
    node = resolve(schema, node)
    array = node.get("type") == "array"
    body = resolve(schema, node["items"]) if array else node
    rows = [(k, resolve(schema, v)) for k, v in body.get("properties", {}).items()
            if not is_table(resolve(schema, v))]
    values = body.get("additionalProperties")
    if isinstance(values, dict) and not is_table(resolve(schema, values)):
        rows.append((body.get("x-grund-key", "<key>"), resolve(schema, values)))
    if header and rows:
        title = f"[[{header}]]" if array else f"[{header}]"
        out.append(f"### `{title}`\n")
        description = node.get("description") or body.get("description")
        if description:
            out.append(cell(description) + cite(node if "x-grund-spec" in node else body) + "\n")
        out.append("| Key | Type | Default | What it sets |")
        out.append("|---|---|---|---|")
        for key, value in rows:
            out.append(f"| `{key}` | {type_of(value)} | {default_of(value)} | "
                       f"{cell(value.get('description', ''))}{cite(value)} |")
        out.append("")
        # `x-grund-note`: what the table's keys mean together, after the keys.
        note = node.get("x-grund-note") or body.get("x-grund-note")
        if note:
            out.append(note + "\n")
    for key, value in body.get("properties", {}).items():
        child = resolve(schema, value)
        if is_table(child):
            render_child(schema, f"{header}.{key}" if header else key, value, out, seen)
    if isinstance(values, dict) and is_table(resolve(schema, values)):
        render_child(schema, f"{header}.{body.get('x-grund-key', '<key>')}", values, out, seen)


def render_child(schema, header, value, out, seen):
    ref = value.get("$ref")
    if ref and ref in seen:
        out.append(f"### `[{header}]`\n")
        out.append(f"The same keys as `[{seen[ref]}]`.\n")
        return
    if ref:
        seen[ref] = header
    render_table(schema, header, value, out, seen)


def render_version(schema, name, title, out):
    version = schema["definitions"][name]
    out.append(f"## {title}\n")
    out.append(version["description"] + "\n")
    top = [(k, resolve(schema, v)) for k, v in version["properties"].items() if not is_table(resolve(schema, v))]
    out.append("### Top level\n")
    out.append("| Key | Type | Default | What it sets |")
    out.append("|---|---|---|---|")
    for key, value in top:
        out.append(f"| `{key}` | {type_of(value)} | {default_of(value)} | "
                   f"{cell(value.get('description', ''))}{cite(value)} |")
    out.append("")
    seen = {}
    for key, value in version["properties"].items():
        if is_table(resolve(schema, value)):
            render_child(schema, key, value, out, seen)


def render():
    schema = json.loads(SCHEMA.read_text(encoding="utf-8"))
    out = [INTRO]
    render_version(schema, "v1", "Version 1", out)
    render_version(schema, "v2", "Version 2", out)
    return "\n".join(out).rstrip("\n") + "\n"


class SpecInDescription(ValueError):
    """A description the installed binary would print naming what it cannot open."""


def project(node, path="#"):
    """§FS-config-explain.3: `node` without its specification metadata, refusing a
    description that names a spec ID or holds a link rather than cutting it."""
    if isinstance(node, list):
        return [project(item, f"{path}/{index}") for index, item in enumerate(node)]
    if not isinstance(node, dict):
        return node
    description = node.get("description")
    if isinstance(description, str) and NAMES_A_SPEC.search(description):
        raise SpecInDescription(f"{path}: the description names a spec ID or holds a link; "
                                f"say it in words, the binary prints it: {description!r}")
    return {key: project(value, f"{path}/{key}") for key, value in node.items()
            if key not in SPEC_METADATA}


def projection():
    schema = json.loads(SCHEMA.read_text(encoding="utf-8"))
    return json.dumps(project(schema), indent=2, ensure_ascii=False) + "\n"


def main(argv):
    try:
        rendered = {PAGE: render(), PROJECTION: projection()}
    except SpecInDescription as refusal:
        print(f"{SCHEMA.relative_to(ROOT)}: {refusal}", file=sys.stderr)
        return 1
    if "--check" in argv:
        stale = [path for path, text in rendered.items()
                 if not path.exists() or path.read_text(encoding="utf-8") != text]
        for path in stale:
            print(f"{path.relative_to(ROOT)} is stale: run python scripts/render_config_reference.py",
                  file=sys.stderr)
        return 1 if stale else 0
    for path, text in rendered.items():
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
