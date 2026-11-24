# DF-fenced-citations: A Markdown fence is read for citations only when `[schema.sources] fenced` names its tag

**Date:** 2026-11-16

Settles how a repository opts a fence language into the citation check, as [§FS-config-v2.schema.sources.fenced](../../functional-spec/FS-config-v2.md#schemasourcesfenced-fenced-names-the-fences-whose-citations-are-read) and [§FS-languages.fence.language](../../functional-spec/FS-languages.md#fencelanguage-a-fences-tag-and-the-fences-fenced-selects) specify. Serves [§GOAL-no-dangling-refs](../../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration), which calls a false negative a bug, and the blind-spot rule of [§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded), which asks that every region the scan does not read be chosen by the repository. Reported as [grund#1364](https://github.com/agent-grounds/grund/issues/1364).

## status: Accepted

## 1. Context

Every Markdown fence was skipped for citations ([§FS-citations.recognized.contexts](../../functional-spec/FS-citations.md#recognizedcontexts-contexts-read-as-neither-prose-nor-code)), which is what keeps an illustration of a citation from becoming a dangling reference. A diagram's node labels are where a component's name goes stale first, and a Mermaid diagram renders only from a `mermaid` fence. A project that moved its diagrams from `.mmd` files into fences, so that they render, took their citations out of the check without a warning: a dangling label passed, and the declaration the diagram cites was reported as never cited. The only way to keep the check was a `<pre>` block, which renders as text, or a separate file, which the document only links.

## 2. Decision

### 2.1 An explicit key, one tag at a time

`[schema.sources] fenced` lists the fence tags whose citations are read. It defaults to `[]`, so nothing changes for a project that does not write it. Selection is not inferred from `languages` or `[schema.sources.definitions]`: a definition admits nothing by itself ([§FS-config-v2.schema.sources.definitions](../../functional-spec/FS-config-v2.md#schemasourcesdefinitions-schemasourcesdefinitionsname-defines-a-language)), a file's reader is never read from its contents ([§FS-config-v2.schema.sources](../../functional-spec/FS-config-v2.md#schemasources-schemasources)), and a project that kept a `mermaid` definition from its `.mmd` days would start failing on upgrade without changing a line of its configuration.

### 2.2 The tag is the literal first word

A fence's tag is the first whitespace-delimited word of its info string, compared literally and case-sensitively. `mermaid title` is `mermaid`, while `Mermaid`, `{.mermaid}` and an empty info string are not. Attribute syntax gets no reading, because each renderer spells it differently and a project names exactly the tag it writes.

### 2.3 Citations, and nothing else

A selected fence is read for citations the way a `<pre>` block is, and it stays a fence for everything else: no heading, declaration, section, value, link definition or `grund:fmt` directive, and no rewrite. A Markdown link inside a Mermaid label breaks the diagram, so `fmt` keeps its hands off as it does in front matter.

### 2.4 No fifth rewrite exemption

Under a `canonical` shorthand policy, a shorthand in a selected fence draws the error, and the author writes the full ID by hand, as in front matter or a suppressed scope ([§FS-citations.rewrite-exemption](../../functional-spec/FS-citations.md#rewrite-exemption-where-the-text-forbids-the-rewrite)). A fifth exempt zone would let a shorthand persist in exactly the text the project asked to have checked.

### 2.5 No new block pair

The managed block's marker bullet names only the unselected fences when `fenced` lists a tag ([§FS-init.2.3.4](../../functional-spec/FS-init.md#234-managed-block-content-sections)). That changes no block a release can write, because every released grund refuses `fenced` at load, so the block keeps its versions ([§FS-init.2.3.7](../../functional-spec/FS-init.md#237-the-block-version)).

## 3. Consequences

- A diagram can stay inline, where it renders, and keep its citations checked.
- A citation written in a selected fence as an illustration needs the `<§>` escape, as one in a `<pre>` block or in front matter already does.
- Every consumer of citations reads a selected fence the same way: `check`, `refs`, `cover`, unused accounting, grounding, citation directions and the editor.

## 4. Alternatives considered

- **Read a fence whose language has a definition**: the cheapest form, but it reads a file's contents to choose a reading and moves verdicts on upgrade, as 2.1 says.
- **Write the diagram in `<pre>`**: keeps the citations live but renders as text, which is enough for an ASCII diagram and not for Mermaid.
- **Read every fence**: defeats the skip, since every illustration of citation syntax would become a dangling reference.

## release-note: Release note

- [§FS-config-v2.schema.sources.fenced](../../functional-spec/FS-config-v2.md#schemasourcesfenced-fenced-names-the-fences-whose-citations-are-read): **A repository can opt a fence language into the citation check.** `[schema.sources] fenced = ["mermaid"]` makes grund read the citations in every Markdown fence tagged `mermaid`: each one resolves, counts as a use and is reported where it dangles, while the fence still holds no heading and `grund fmt` still rewrites nothing in it. The key is additive and defaults to `[]`, so no verdict moves on upgrade. A grund older than this release refuses the key at its line. **What an upgrader does:** nothing; a project that wants its inline diagrams checked lists their tag. (agent-grounds/grund#1364)
