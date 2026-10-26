<!-- BEGIN GRUND MANAGED BLOCK -->
## Grounding with grund (block v45)

These rules cover the files under `tests/e2e/`; the rest of this project's grund guidance is in the root `AGENTS.md`.

### Citation directions

`must`/`never` are `grund check` errors; `should`/`avoid` are suggestions (`grund check --suggestions`).

- Each file in **tests/e2e/** that cites anything must cite FS. A file there that cites nothing is not checked; `[rules.citations.e2e.grounding] must = "file"` makes it a finding.
Anything not listed above is allowed.
<!-- END GRUND MANAGED BLOCK -->
