## Querying the Structure

Once the tree checks clean, read its structure with grund's queries rather than by parsing heading lines: `grund list --glob` takes a coordinate pattern — `*` inside one component, `**` any number of whole components, `{a,b}` each alternative — and `--total` counts what it selects, while `grund show --batch --glob` reads the same coordinates in one scan. Each answer's `.result.sections[]` lists the unit's subpoints as `path`, `title` and `depth`:

```bash
grund list --glob '<KIND>-*.<chapter>' --total
grund show --batch --glob '<KIND>-*.<chapter>' --toc --format json
```

The [Querying grund guide](https://github.com/agent-grounds/grund/blob/main/docs/user-facing/querying.md) has the rest: a point and its subtree with `<ID>{,.**}`, bodies in bulk with `show --batch --brief|--full`, `--glob` or `--all`, units by rule subject with `list --selector`, `refs --descendants`, who cites a set with `refs --glob` and what a point cites with `refs --from`, a checklist review that ends with `cover FILE --lines N-M`, and the whole citation graph from `cover --format json`, or the part a pattern names with `cover --glob`.
