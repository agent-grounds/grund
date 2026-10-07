# REQ-remote-equivalence: a fetched remote answers every operation as a local project does

Every operation works on a fetched project exactly as on a local one, and that holds by
construction rather than by teaching each command to read a cache: the next command added
must not be able to break it. A project that cites another repository's declarations must
get the same answers an agent or an editor gets for a local member, or cross-repository
grounding is a second-class citation ([§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work), [§GOAL-small-and-large](../goals.md#goal-small-and-large-start-small-configure-for-big)) and a
check that passes on a remote is not the check that passes on the source
([§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration)).

## 1. The contract

For any project P and any project O, every operation `grund` offers — `check`'s findings
about P's sites, `grund <ID>` in every form, `list`, `refs`, `cover`, value bindings, rule
subjects and objects, completion, hover, definition, references, cross-ref rendering, and
every write into P's own files computed from O's content — produces the same result when O
is a fetched remote of P as when O's bytes are a locally mounted member of P under the same
alias, and the same graph O's standalone run produces, except as [§REQ-remote-equivalence.2](REQ-remote-equivalence.md#2-the-closed-list-of-differences)
lists.

## 2. The closed list of differences

The differences are exactly five, specified in [§FS-remote-projects.differences](../functional-spec/FS-remote-projects.md#differences-the-closed-list-of-differences), and no
operation may add a sixth without amending this requirement:

1. writes whose target lies in a projection are refused, the one exception to "every
   operation", accepted because editing a locked snapshot in place would make its committed
   bytes disagree with the commit its lock names;
2. content findings the remote owns are suppressed in an automatic consumer run and
   reported by an explicitly scoped one, and no data is removed;
3. a mounted root's ignore inputs are its own;
4. paths are relocated under `.grund/remotes/<alias>/`;
5. integrity is verified offline and recursively.

Suppression may never let a consumer citation pass that a local twin would fail
([§FS-remote-projects.differences.consumer-site](../functional-spec/FS-remote-projects.md#differencesconsumer-site-suppression-never-lets-a-consumer-citation-pass)).

## 3. Writes are inside the contract

A write into P's own files computed from O's content is an operation like any read:
re-deriving a link after an upstream rename, normalizing a citation and inserting a
completion produce the same diff against a remote as against a locally mounted member, and
a dry run reports the same exclusions and refusals as write mode.

## 4. The proof is two comparisons over an audited inventory

The requirement is met only while both comparisons pass over an inventory of every CLI,
library and editor operation, writer diffs, preview and write parity, and refusals
included:

1. **source against projection** — O's standalone run against O as a remote of a consumer,
   which catches lost inputs and namespace edges that two copies of the same loss cannot;
2. **projection against local twin** — O as a remote against O's bytes as a locally mounted
   member, which catches any special reading path.

Only declared relocation fields are normalized between the two sides, never arbitrary text,
and the source run uses the committed portable scan context of [§FS-remote-projects.ignore](../functional-spec/FS-remote-projects.md#ignore-ignore-isolation).
An operation added later is covered by entering the inventory, and an operation with no
inventory row fails the proof rather than passing it.
