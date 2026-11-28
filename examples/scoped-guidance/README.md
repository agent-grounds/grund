# Scoped guidance: each kind's rules in the `AGENTS.md` above its places

This runnable mini-repository shows where `grund init` writes a version-2 project's agent guidance ([§FS-init.2.3.5](../../docs/functional-spec/FS-init.md#235-citation-directions)). It is for a repository whose kinds live in their own directories: an agent editing a spec should find the spec's rules beside it, not read every kind's rules from the root file to find them. Open `repo/grund.toml` first:

```toml
grund_config_version = 2
project_name = "scoped-guidance-example"
[schema.kinds.GOAL]
files = ["docs/goals.md"]
[schema.kinds.FS]
folders = ["docs/spec/core", "docs/spec/plugins"]
[schema.kinds.test]
citable = false
folders = ["tests/e2e", "tests/integration"]
[schema.kinds.fixture]
citable = false
folders = ["tests/fixtures"]
[rules.citations.FS]
must = ["GOAL"]
[rules.citations.test]
must = ["FS"]
[rules.citations.fixture]
default = "must-not"
```

## Run it

From the repository root, on a copy of the fixture:

```bash
cp -r examples/scoped-guidance/repo ~/scoped && git -C ~/scoped init -q
grund init ~/scoped
echo $?    # 0
```

The run writes four files, which `expected.repo/` records byte for byte, and its report names each of them ([§FS-init.2.3.5](../../docs/functional-spec/FS-init.md#235-citation-directions)):

- `docs/spec/AGENTS.md`, the deepest directory holding both of FS's folders ([§FS-init.2.3.5](../../docs/functional-spec/FS-init.md#235-citation-directions)). Its managed block says that each FS declaration must cite GOAL, and nothing else ([§FS-init.2.3.5](../../docs/functional-spec/FS-init.md#235-citation-directions)).
- `tests/AGENTS.md`, the directory above `tests/e2e/` and `tests/integration/`, with the test row's rule.
- `tests/fixtures/AGENTS.md`, with the fixture row's rule. That row is closed to every citation: its default is `must-not` and it lists no target ([§FS-config.3.9.4.1](../../docs/functional-spec/FS-config.md#3941-a-row-closed-to-every-citation)). So its bullet is the one sentence `Each file in **tests/fixtures/** must not cite anything.`, with no `that cites anything` and no grounding setting. The test row's bullet keeps both, because a test that cites nothing slips past `must = ["FS"]` until grounding is on, while a fixture that cites nothing already keeps its rule ([§FS-init.2.3.5](../../docs/functional-spec/FS-init.md#235-citation-directions)).
- `AGENTS.md` at the root: the project-wide guidance, with none of the three bullets, and a `### Scoped guidance` list naming the three files ([§FS-init.2.3.5](../../docs/functional-spec/FS-init.md#235-citation-directions)).

GOAL has no rule, so `docs/AGENTS.md` is not written ([§FS-init.2.3.5](../../docs/functional-spec/FS-init.md#235-citation-directions)). Stdout is empty.

## Which agents read the scoped files by themselves

Claude Code, Gemini CLI, Cursor and Windsurf load a scoped file when they work on a file below it; Claude and Gemini read it through a `CLAUDE.md` or `GEMINI.md` symlink that `init` mirrors where the root has one ([§FS-init.2.3.5](../../docs/functional-spec/FS-init.md#235-citation-directions)). Codex and Pi read instruction files only from the root down to the directory they started in, and Copilot and Zed are not verified. Those agents follow the root block's `### Scoped guidance` list and read the file before writing there. The table, with the documentation each row was read from, is [§DF-scoped-guidance.loading](../../docs/decisions/functional/DF-scoped-guidance.md#loading-which-agents-load-a-scoped-file-by-themselves).

## The trade-off

- **Benefit:** an agent writing a spec reads the spec's rules, and its declaration shape once the kind declares fields ([§FS-init.2.3.5](../../docs/functional-spec/FS-init.md#235-citation-directions)), and not every other kind's.
- **Cost:** guidance is spread over several files, and an agent that does not load nested files must take the extra read the pointer asks for. A team that wants everything in the root file sets `[presentation] rules = "root"` ([§FS-init.2.3.5](../../docs/functional-spec/FS-init.md#235-citation-directions)).
