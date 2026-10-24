# Documentation rules

The rules this documentation keeps beyond the citation directions of
`grund.toml`. Each title is the sentence `grund check` enforces.

## RULE-goals-measured: Each QG must be cited by at least one QS.

arc42 names the quality goals in section 1.2 and makes each one measurable in
section 10.2. A goal that no scenario cites has no measure, so nobody can tell
whether the architecture meets it.

## RULE-successor-named: The status chapter of each ADR whose status is Superseded must cite at least one ADR.

A superseded record names the record that replaced it, on the first line under
its status, as adr-tools and MADR both do. A record whose status says anything
else replaces nothing and needs no such link, so the rule does not reach it.
