"""The JSON Schema of `grund.toml` says what the readers accept (§FS-config.4).

Three things keep `schemas/grund.schema.json` honest: the reference page is the
schema rendered, every key the schema names is one the reader of its version
accepts, and every committed configuration `grund config validate` accepts
validates against the schema. The validator below covers the draft-07 keywords
the schema uses, so the test needs nothing beyond the standard library.
"""

import functools
import json
import os
import re
import subprocess
import sys
import tempfile
import tomllib
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
SCHEMA = json.loads((REPO_ROOT / "schemas" / "grund.schema.json").read_text(encoding="utf-8"))
EXE = "grund.exe" if os.name == "nt" else "grund"
UNKNOWN = re.compile(r"unknown (config )?(key|section)")

sys.path.insert(0, str(REPO_ROOT / "scripts"))
import render_config_reference  # noqa: E402


@functools.lru_cache(maxsize=None)
def _grund():
    """The binary under test: the gate's build, or this checkout's built once."""
    built = os.environ.get("GRUND_BUILT")
    if built:
        return built
    target = Path(os.environ.get("CARGO_TARGET_DIR") or REPO_ROOT / "target")
    subprocess.run([os.environ.get("CARGO", "cargo"), "build", "-p", "grund", "--locked"],
                   cwd=REPO_ROOT, check=True)
    return str(target / "debug" / EXE)


def validate(directory):
    """`grund config validate` on one project: (exit code, stderr)."""
    run = subprocess.run([_grund(), "config", "validate", str(directory)],
                         capture_output=True, text=True, cwd=REPO_ROOT)
    return run.returncode, run.stderr


# ---- a draft-07 validator for the keywords the schema uses

def resolve(node):
    while "$ref" in node:
        node = SCHEMA["definitions"][node["$ref"].rsplit("/", 1)[-1]]
    return node


def type_matches(value, kind):
    if kind == "integer":
        return isinstance(value, int) and not isinstance(value, bool)
    return isinstance(value, {"string": str, "boolean": bool, "array": list, "object": dict}[kind])


def errors(value, node, path="$"):
    node = resolve(node)
    found = []
    if node.get("not") == {}:
        return [f"{path}: refused"]
    if "if" in node:
        branch = "then" if not errors(value, node["if"], path) else "else"
        found += errors(value, node[branch], path)
    if "type" in node and not type_matches(value, node["type"]):
        return found + [f"{path}: not {node['type']}"]
    if "const" in node and value != node["const"]:
        found.append(f"{path}: not {node['const']!r}")
    if "enum" in node and value not in node["enum"]:
        found.append(f"{path}: {value!r} not in {node['enum']}")
    if "minimum" in node and value < node["minimum"]:
        found.append(f"{path}: below {node['minimum']}")
    if "maximum" in node and value > node["maximum"]:
        found.append(f"{path}: above {node['maximum']}")
    if "anyOf" in node and all(errors(value, alt, path) for alt in node["anyOf"]):
        found.append(f"{path}: matches no alternative")
    if isinstance(value, list):
        if len(value) < node.get("minItems", 0) or len(value) > node.get("maxItems", len(value)):
            found.append(f"{path}: wrong length")
        for i, item in enumerate(value):
            found += errors(item, node.get("items", {}), f"{path}[{i}]")
    if isinstance(value, dict):
        props = node.get("properties", {})
        extra = node.get("additionalProperties", True)
        found += [f"{path}: missing {key}" for key in node.get("required", []) if key not in value]
        for key, item in value.items():
            if key in props:
                found += errors(item, props[key], f"{path}.{key}")
            elif extra is False:
                found.append(f"{path}: unexpected key {key}")
            elif isinstance(extra, dict):
                found += errors(item, extra, f"{path}.{key}")
    return found


# ---- configs that name one schema key each

def toml_value(value):
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, str):
        return json.dumps(value, ensure_ascii=False)
    if isinstance(value, list):
        return "[" + ", ".join(toml_value(v) for v in value) + "]"
    if isinstance(value, dict):
        return "{ " + ", ".join(f"{k} = {toml_value(v)}" for k, v in value.items()) + " }"
    return str(value)


def emit(config):
    """A config dict as the TOML subset both readers take: one line per key."""
    lines = [f"{k} = {toml_value(v)}" for k, v in config.items() if not isinstance(v, (dict, list))
             or isinstance(v, list) and not (v and isinstance(v[0], dict))]

    def table(prefix, body):
        scalars = {k: v for k, v in body.items() if not isinstance(v, dict) or v.get("__inline")}
        if scalars:
            lines.append(f"[{prefix}]")
            lines.extend(f"{k} = {toml_value({x: y for x, y in v.items() if x != '__inline'} if isinstance(v, dict) else v)}"
                         for k, v in scalars.items())
        for k, v in body.items():
            if isinstance(v, dict) and not v.get("__inline"):
                table(f"{prefix}.{k}", v)

    for key, value in config.items():
        if isinstance(value, list) and value and isinstance(value[0], dict):
            for row in value:
                lines.append(f"[[{key}]]")
                lines.extend(f"{k} = {toml_value(v)}" for k, v in row.items())
        elif isinstance(value, dict):
            table(key, value)
    return "\n".join(lines) + "\n"


def sample(node):
    node = resolve(node)
    if "examples" in node:
        return node["examples"][0]
    if "default" in node:
        return node["default"]
    if "enum" in node:
        return node["enum"][0]
    if "anyOf" in node:
        return sample(node["anyOf"][0])
    kind = node.get("type")
    if kind == "object":
        return {"__inline": True, **{k: sample(v) for k, v in node["properties"].items()}}
    if kind == "array" and "enum" in node.get("items", {}):
        return node["items"]["enum"][:1]
    return {"string": "x", "boolean": True, "integer": node.get("minimum", 1), "array": []}[kind]


PLACEHOLDERS = {"<KIND>": "FS", "<NAME>": "FS", "<field>": "summary", "<suffix>": "mdx",
                "<name>": "notes", "<key>": "status"}


def leaf_keys(node, path=()):
    """(path, schema) for every key the schema names below `node`, refused keys aside."""
    node = resolve(node)
    if node.get("type") == "array" and resolve(node.get("items", {})).get("type") == "object":
        yield from leaf_keys(node["items"], path + ("[]",))
        return
    for key, value in node.get("properties", {}).items():
        child = resolve(value)
        if child.get("not") == {}:
            continue
        if child.get("type") == "object" and not child.get("x-grund-inline") or "items" in child and \
                resolve(child["items"]).get("type") == "object":
            yield from leaf_keys(value, path + (key,))
        else:
            yield path + (key,), child
    extra = node.get("additionalProperties")
    if isinstance(extra, dict):
        name = PLACEHOLDERS[node["x-grund-key"]]
        child = resolve(extra)
        if child.get("type") == "object":
            yield from leaf_keys(extra, path + (name,))
        else:
            yield path + (name,), child


BASE = {
    "v1": {"grund_config_version": 1, "kinds": [{"kind": "FS", "folder": "docs"}]},
    "v2": {"grund_config_version": 2,
           "schema": {"sources": {"languages": ["markdown"]}, "kinds": {"FS": {"folders": ["docs"]}}}},
}


def table_schema(version, path):
    """The schema of the table that holds the key at `path`."""
    node = resolve(SCHEMA["definitions"][version])
    for part in path[:-1]:
        if part == "[]":
            node = resolve(node["items"])
        else:
            properties = node.get("properties", {})
            node = resolve(properties[part] if part in properties else node["additionalProperties"])
    return node


def config_naming(version, path, value):
    config = json.loads(json.dumps(BASE[version]))
    node = config
    for i, part in enumerate(path[:-1]):
        if part == "[]":
            continue
        if path[i + 1:i + 2] == ("[]",):
            node = node.setdefault(part, [{}])[0]
        else:
            node = node.setdefault(part, {})
    node[path[-1]] = value
    # A key table follows its field's header (§FS-fields.front-matter.config),
    # which a table holding no key of its own would not write.
    for i, part in enumerate(path[:-1]):
        if part == "keys" and path[i - 1:i] == ("front-matter",):
            holder = config
            for step in path[:i]:
                holder = holder[step]
            holder.setdefault("presence", "may")
    # A key is named in a table that otherwise holds only what the table requires.
    table = table_schema(version, path)
    for key in table.get("required", []):
        if key not in node:
            node[key] = sample(table["properties"][key])
    return config


class ConfigSchemaTests(unittest.TestCase):
    def test_reference_page_is_the_schema_rendered(self):
        page = (REPO_ROOT / "docs" / "user-facing" / "configuration.md").read_text(encoding="utf-8")
        self.assertEqual(page, render_config_reference.render(),
                         "docs/user-facing/configuration.md is stale: run "
                         "python scripts/render_config_reference.py")

    def test_the_embedded_schema_is_the_published_one_without_its_spec_metadata(self):
        """§AR-config.9.1: the copy `grund config explain` words each key from."""
        embedded = render_config_reference.PROJECTION.read_text(encoding="utf-8")
        self.assertEqual(embedded, render_config_reference.projection(),
                         "crates/grund-core/assets/explain/grund.schema.json is stale: run "
                         "python scripts/render_config_reference.py")
        for metadata in render_config_reference.SPEC_METADATA:
            self.assertNotIn(f'"{metadata}"', embedded)

    def test_a_description_citing_a_spec_id_or_linking_is_refused_not_cut(self):
        """§FS-config-explain.3: the description is printed as the schema writes it,
        so one the binary could not print whole stops the render."""
        project = render_config_reference.project
        for description in ("Skipped ([§FS-config.3](../functional-spec/FS-config.md)).",
                            "See §FS-config.3.", "See [the guide](sources.md)."):
            node = {"properties": {"exclude": {"description": description}}}
            with self.assertRaisesRegex(render_config_reference.SpecInDescription,
                                        "#/properties/exclude"):
                project(node)
        spelled = {"description": "Its alias in `§alias/<ID>` citations.",
                   "x-grund-spec": {"id": "FS-config.3"}}
        self.assertEqual(project(spelled), {"description": spelled["description"]})

    def test_every_key_the_schema_names_is_one_the_reader_accepts(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "docs").mkdir()
            for version in ("v1", "v2"):
                for path, node in leaf_keys(SCHEMA["definitions"][version]):
                    with self.subTest(version=version, key=".".join(path)):
                        text = emit(config_naming(version, path, sample(node)))
                        (root / "grund.toml").write_text(text, encoding="utf-8")
                        code, stderr = validate(root)
                        self.assertIsNone(UNKNOWN.search(stderr), f"{text}\n{stderr}")
                        # A refusal must be about this key, or it could be hiding it.
                        if code != 0:
                            self.assertIn(path[-1], stderr, f"{text}\n{stderr}")

    def test_every_committed_config_the_reader_accepts_validates(self):
        configs = [REPO_ROOT / "grund.toml"]
        # §FS-init.2.3.7: the templates' one copy is the asset `grund init` embeds.
        templates = sorted((REPO_ROOT / "crates/grund-core/assets/templates").glob("*/grund.toml"))
        self.assertEqual({"software-development", "none"}, {p.parent.name for p in templates},
                         "the template configs were not found where grund init reads them")
        configs += templates
        configs += sorted(p for base in ("examples", "tests/e2e/cases")
                          for p in (REPO_ROOT / base).rglob("grund.toml")
                          if "expected.repo" not in p.parts)
        checked = 0
        for config in configs:
            code, _ = validate(config.parent)
            if code != 0:
                continue
            try:
                data = tomllib.loads(config.read_text(encoding="utf-8"))
            except tomllib.TOMLDecodeError:
                continue
            checked += 1
            with self.subTest(config=str(config.relative_to(REPO_ROOT))):
                self.assertEqual(errors(data, SCHEMA), [])
        self.assertGreater(checked, 100, "too few committed configs were checked")


if __name__ == "__main__":
    unittest.main()
