# DF-nested-alias-rule-targets: a rule target pinned at its own nested alias matches like the bare kind

**Status:** Accepted
**Date:** 2026-10-12

## 1. Context

agent-grounds/grund#627 reported a rule that failed over a declaration that visibly cited its object. In a workspace where member `api` sits inside member `group`, so that its alias path is `group/api` ([§FS-workspace.6.1](../../functional-spec/FS-workspace.md#61-nested-workspaces)), `API-a` cited `AR-b`, a declaration of the same member, and the member's rule `Each API must cite at least one group/api/AR.` reported `API-a must cite group/api/AR`. The same rule with a bare `AR` passed.

The ground already said the target matches. [§FS-config.3.9.3](../../functional-spec/FS-config.md#393-alias-matching) spells a pinned entry with the same whole alias path a citation uses and splits it "at the last `/` exactly as a citation is", and [§FS-rules.2](../../functional-spec/FS-rules.md#2-subject-selectors) reuses that grammar unchanged for rule object kinds. The rule validator follows it, so the rule is accepted; the engine's matcher then split the rendered target at its first `/`, compared the alias `group` with the rule's own member `group/api`, and matched nothing. The same split made the twin rule `Each API must not cite any group/api/AR.` pass over the very citation it forbids.

## 2. Decision

### 2.1 Match on the parsed target

The engine's matcher reads each target through the parser the validator already accepted it through, and matches on the namespace it returns. A target pinned at the rule's own member by its whole alias path, nested or not, matches exactly what the bare kind matches ([§FS-rules.2](../../functional-spec/FS-rules.md#2-subject-selectors)). Both relations read that one matcher, outbound (`cite`) and inbound (`be cited by`), so both are corrected together. Bare kinds, `*/KIND`, a pin to another member, and flat workspaces, where no alias holds a `/`, judge as before.

### 2.2 The new failures take the deprecation path

Matching more declarations makes some verdicts pass and others fail. The ones that newly pass — an `at least` count or per-target coverage the matched sites now satisfy — land at once, because nothing that passed before breaks; that is the precedent of [§DF-section-citation-counts-in-rules.2.3](DF-section-citation-counts-in-rules.md#23-the-new-failures-take-the-deprecation-path). The ones that newly fail — a prohibition, and an `at most N` or `exactly N` count, outbound or inbound — are verdict changes under [§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered), so they take [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)'s route: one release window in which such a finding is a warning naming `grund 0.18.0` in its own bytes, then an error ([§FS-rules.7.9](../../functional-spec/FS-rules.md#79-a-target-pinned-at-its-own-nested-alias-warns-until-0180)). The window is the one [§FS-rules.7.8](../../functional-spec/FS-rules.md#78-a-newly-counted-section-citation-warns-until-0180) already opened, through the same engine channel; a finding both windows reach stays a warning until the later of their releases and carries both clauses.

### 2.3 Why not the correction route

The correction route, section 5 of [§REQ-backwards-compatibility](../../requirements/REQ-backwards-compatibility.md#req-backwards-compatibility-an-upgrade-never-changes-a-verdict-quietly), would let the new failures land at once, but only where the old verdict violated a hard requirement that already applied when the verdict shipped, in `grund 0.14.0`. None did. [§REQ-no-missed-citation](../../requirements/REQ-no-missed-citation.md#req-no-missed-citation-every-citation-the-run-reads-is-checked) is about recognising and validating citations, and `API-a`'s citation of `AR-b` was recognised and resolved; stretching it to how a rule matches a resolved citation is the invented prohibition the route's fifth condition refuses, as [§DF-section-citation-counts-in-rules.2.4](DF-section-citation-counts-in-rules.md#24-why-not-the-correction-route) found for the same reason. [§REQ-one-answer](../../requirements/REQ-one-answer.md#req-one-answer-every-command-gives-the-same-answer-about-one-tree) landed after the verdict shipped and is never the prior prohibition for a difference that predates it, and [§FS-config.3.9.3](../../functional-spec/FS-config.md#393-alias-matching) is a specification point, not a hard requirement. The route is considered and not used: this record claims no correction.

## 3. Alternatives considered

**Store the parsed target in the rule's targets.** It would remove the second reading altogether, but it changes every consumer of the rule's target set and is the coordinate-lexer unification agent-grounds/grund#626 already owns. This record fixes the verdict; that issue removes the copy.

**Ship the new failures as errors.** [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) forbids it, and [§DF-nested-alias-rule-targets.2.3](DF-nested-alias-rule-targets.md#23-why-not-the-correction-route) above is why the correction route does not open.

## 4. Consequences

A nested member's rule that pinned its own alias path and failed only because the pin matched nothing now passes, and the workaround — writing the bare kind instead of the pinned one — is no longer needed, though it keeps working. A tree whose prohibition or upper bound the pin silently evaded keeps exiting `0`, but prints a warning per finding instead of `success`, and exits `1` from `0.18.0`. Only a workspace that nests one member inside another and pins the inner member's own alias path in that member's rules is touched; a repository that writes bare kinds, flat aliases or `*/` is byte-for-byte unchanged.

## release-note: Release note

- [§FS-rules.2](../../functional-spec/FS-rules.md#2-subject-selectors), [§FS-config.3.9.3](../../functional-spec/FS-config.md#393-alias-matching), [§FS-rules.7.9](../../functional-spec/FS-rules.md#79-a-target-pinned-at-its-own-nested-alias-warns-until-0180), [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): **a rule target pinned at its own nested member's alias path now matches that member's declarations.** In member `group/api`, `Each API must cite at least one group/api/AR.` judges exactly as `Each API must cite at least one AR.` does, so a rule that failed only because the pin matched nothing now passes. A finding the corrected match alone produces — a `must not cite` prohibition, or an `at most N` or `exactly N` count it pushes out of bounds, outbound or inbound — is a warning in this release, carrying the code it will carry as an error and ending `; a target pinned at the rule's own nested alias path now matches, and this warning becomes an error in grund 0.18.0`; the exit code does not move until then. **Who this breaks, from 0.18.0:** a workspace with a member nested inside another whose prohibition or upper-bound rule pins that member's own alias path. The escapes are to remove the citation, loosen the rule's count, or pass `--ignore` on the finding's code. (agent-grounds/grund#627)
