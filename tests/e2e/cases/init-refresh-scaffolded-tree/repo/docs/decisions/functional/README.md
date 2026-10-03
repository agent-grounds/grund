# Functional decisions

Product-behavior decisions and the tradeoffs behind them — *why* the behavior is the one it is. One file per decision; each H1 declares a `DF-<NNN>-<slug>` ID and the body is the record.

Cite a decision from the spec point it settles, so the rule and the argument for it stay one hop apart. A decision that nothing cites is a note, and `grund check` will say so.

By convention every decision under this directory is linked from this index, and `grund check` verifies it: each ID appears here once, as a `§`-marked Markdown link — write the row with the marker and `grund fmt --write` keeps the link and its anchor current; a row without the marker is prose, and `grund check` will say so. Grouping, ordering, and the prose around the link set are yours.

| ID | Subject |
|---|---|

This index is navigational — citations should target the decision ID directly, never this file.
