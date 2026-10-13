# Requirements

Hard requirements — what the project must never break, kept apart from the behavior it specifies. One file per requirement; each H1 declares a `REQ-<NNN>-<slug>` ID and the body is the requirement.

Cite a requirement from the goal it protects, and cite it from every spec point that must keep it, so a change that would break it meets it first.

By convention every requirement under this directory is linked from this index, and `grund check` verifies it: each ID appears here once, as a `§`-marked Markdown link — write the row with the marker and `grund fmt --write` keeps the link and its anchor current; a row without the marker is prose, and `grund check` will say so. Grouping, ordering, and the prose around the link set are yours.

| ID | Subject |
|---|---|

This index is navigational — citations should target the requirement ID directly, never this file.
