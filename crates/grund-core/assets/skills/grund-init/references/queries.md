## Querying the Structure

Once the tree checks clean, read its structure with grund's queries rather than by parsing heading lines: `grund list --glob` takes a coordinate pattern — `*` inside one component, `**` any number of whole components, `{a,b}` each alternative — and `--total` counts what it selects, while `grund show --batch --glob` reads the same coordinates in one scan. Each answer's `.result.sections[]` lists the unit's subpoints as `path`, `title` and `depth`:

```bash
grund list --glob '<KIND>-*.<chapter>' --total
grund show --batch --glob '<KIND>-*.<chapter>' --toc --format json
```

For the declarations nothing cites yet, run `grund list --unused` once rather than one `grund refs` per declared ID: it leaves out E2E cases unless `--kind E2E` selects them, and `--format json` prints one object per uncited declaration.

The [Querying grund guide](https://github.com/agent-grounds/grund/blob/main/docs/user-facing/querying.md) has the rest: a point and its subtree with `<ID>{,.**}`, bodies in bulk with `show --batch --brief|--full`, `--glob` or `--all`, units by rule subject with `list --selector`, the declarations nothing cites yet with `list --unused`, `refs --descendants`, who cites a set with `refs --glob` and what a point cites with `refs --from`, a checklist review that ends with `cover FILE --lines N-M`, and the whole citation graph from `cover --format json`, or the part a pattern names with `cover --glob`.
