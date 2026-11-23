"""The coverage gate's tables, as the fold pass rewrites them: §AR-goal-measurement.1.

`tests/integration/functional_spec_coverage_policy.rs` holds two kinds of row the pass keeps
in step with a rewrite: an exception row excuses a leaf from test evidence, and a lead-promise
row makes the gate demand an exact citation of a parent whose lead holds a promise of its own
(§REQ-spec-content.grain). A fold removes coordinates either may name, and a record's
`lead-promises` list adds parents to the second.
"""

from __future__ import annotations

import re

from spec_fold_map import FoldMap, Spec

ROW = re.compile(r'^(?P<indent>\s*)Exception \{ id: "(?P<id>[^"]+)", reason: "(?P<reason>(?:[^"\\]|\\.)*)" \},\s*$')
PROMISES = re.compile(
    r"^pub\(super\) const LEAD_PROMISES: &\[LeadPromise<'static>\] = &\[\n(?P<body>.*?)^\];", re.M | re.S
)
PROMISE_ROW = re.compile(r'^\s*LeadPromise \{ id: "(?P<id>[^"]+)", reason: "(?P<reason>(?:[^"\\]|\\.)*)" \},\s*$')


def retarget_exceptions(text: str, fold: FoldMap, after: Spec, evidence: set[str], path: str, hand: list[str]) -> str:
    """Delete the coverage exception rows of removed leaves.

    The gate holds evidence of leaves alone (§AR-goal-measurement.1), so a target the fold
    leaves a leaf with no evidence takes one row in the place of its children's, with the
    reason they gave; the row is reported, because evidence is read here by pattern, not by
    the scanner the gate runs.
    """
    out, orphaned = [], {}
    for line in text.split("\n"):
        row = ROW.match(line)
        target = fold.lookup(row.group("id")) if row else None
        if target is None:
            out.append(line)
            continue
        slot = orphaned.setdefault(target, [len(out), row.group("indent"), []])
        if row.group("reason") not in slot[2]:
            slot[2].append(row.group("reason"))
    kept = {row.group("id") for row in map(ROW.match, out) if row}
    for target, (index, indent, reasons) in sorted(orphaned.items(), key=lambda item: -item[1][0]):
        if after.is_leaf(target) and target not in evidence and target not in kept:
            out.insert(index, f'{indent}Exception {{ id: "{target}", reason: "{"; ".join(reasons)}" }},')
            hand.append(f"{path}: added the exception row of `{target}`, a leaf with no evidence; check its reason")
    for target in sorted(set(fold.targets.values()) & kept & evidence):
        hand.append(f"{path}: `{target}` keeps an exception row but now has evidence; retire the row")
    return "\n".join(out)


def natural(coordinate: str) -> list[tuple[int, int, str]]:
    """A coordinate's sort key: numbered components as numbers, so `2.10` follows `2.9`."""
    return [(0, int(part), "") if part.isdigit() else (1, 0, part) for part in coordinate.split(".")]


def rust_string(text: str) -> str:
    return text.replace("\\", "\\\\").replace('"', '\\"')


def lead_promises(text: str, promises: list, fold: FoldMap, after: Spec, evidence: set[str],
                  path: str, hand: list[str], added: list[str]) -> str:
    """Add the records' lead promises to `LEAD_PROMISES`, sorted and each kept once, and follow
    the rows of parents the fold removed.

    A removed parent's promise is now its target's, so its row moves there while the target
    keeps children, and a row goes when its section is now a leaf, which the gate holds
    already; each is reported. A row whose parent no test cites exactly is reported too: the
    gate fails on it.
    """
    table = PROMISES.search(text)
    if table is None:
        for promise in promises:
            hand.append(f"{path}: no LEAD_PROMISES table; add `{promise.id}` ({promise.reason}) by hand")
        return text
    lines = [line for line in table.group("body").split("\n") if line.strip()]
    rows = {row.group("id"): row.group("reason") for row in map(PROMISE_ROW.match, lines) if row}
    if len(rows) != len(lines):
        hand.append(f"{path}: LEAD_PROMISES holds a line that is no row; add the records' lead promises by hand")
        return text
    for parent in list(rows):
        target = fold.lookup(parent) or parent
        if not after.has(target):
            hand.append(f"{path}: the lead promise `{parent}` names no section of the tree; the gate fails on it")
            continue
        if target == parent and not after.is_leaf(parent):
            continue
        reason = rows.pop(parent)
        if after.has(target) and not after.is_leaf(target):
            rows.setdefault(target, reason)
            hand.append(f"{path}: the lead promise of the removed `{parent}` is now `{target}`'s; check its reason")
        else:
            hand.append(f"{path}: dropped the lead promise of `{parent}`: `{target}` is now a leaf, "
                        "which the gate holds already")
    for promise in promises:
        if promise.id not in rows:
            rows[promise.id] = rust_string(promise.reason)
            added.append(f"{promise.id}: {promise.reason} ({promise.where})")
    for parent in sorted(rows, key=natural):
        if parent not in evidence:
            hand.append(f"{path}: no test cites the lead promise `{parent}` exactly; "
                        "the coverage gate fails until one does")
    body = "".join(f'    LeadPromise {{ id: "{parent}", reason: "{rows[parent]}" }},\n'
                   for parent in sorted(rows, key=natural))
    return text[: table.start("body")] + body + text[table.end("body"):]
