# Functional Spec

Behavior — *what* this project does, its requirements and constraints. One file per spec; each H1 declares an `FS-<slug>` ID and the body is the behavior. Citations from elsewhere in the tree (`§FS-<slug>.<section>`) resolve into these files.

Cite a spec from the code and the end-to-end tests that realize it, so every behavior can be traced to what proves it.

By convention every spec under this directory is linked from this index, and `grund check` verifies it: each ID appears here once, as a `§`-marked Markdown link — write the row with the marker and `grund fmt --write` keeps the link and its anchor current; a row without the marker is prose, and `grund check` will say so. Grouping, ordering, and the prose around the link set are yours.

| ID | Subject |
|---|---|

This index is navigational — citations should target the spec ID directly, never this file.
