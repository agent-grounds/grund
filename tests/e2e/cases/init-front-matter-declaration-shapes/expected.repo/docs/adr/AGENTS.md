<!-- BEGIN GRUND MANAGED BLOCK -->
## Grounding with grund (block v53)

These rules cover the files under `docs/adr/`; the rest of this project's grund guidance is in the root `AGENTS.md`.

### Declaration shapes

`grund check` holds each declaration of these kinds to these fields, as `grund schema` lists them. Each heading is shown under a declaration written `# <ID>: …`; a field always sits one level below its declaration's heading:

- **ADR**
  - `---` YAML or `+++` TOML front matter at the top of the file (must)
    - `status: <one of proposed, accepted, rejected, deprecated>` (must)
    - `date: <value>` (must)
    - `decision-makers: <value>` (may)
    - no other key (must)
  - `## context: <title>` (must)
  - `## decision: <title>` (must)
  - no other chapter except numbered ones (must)
<!-- END GRUND MANAGED BLOCK -->
