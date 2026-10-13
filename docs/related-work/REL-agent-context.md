# REL-agent-context: repository context for coding agents, what an agent is given to read

Research and tools on how a coding agent comes to know a repository, through the commands
it can call, the instruction files it is handed and the amount it reads, are the setting
of grund's cheap reads, which give an agent the smallest slice of a specification that
answers its question ([§GOAL-token-economy](../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file)). The same work bears on the agent entrypoints
`grund init` writes, the first of grund's three grounding layers ([§GOAL-agent-grounding.1](../goals.md#1-the-three-layers)),
and its evidence does not all point one way.

## work: What the work is

**Interfaces and structured reads.** John Yang, Carlos E. Jimenez, Alexander Wettig,
Kilian Lieret, Shunyu Yao, Karthik Narasimhan and Ofir Press, *SWE-agent: Agent-Computer
Interfaces Enable Automated Software Engineering*, NeurIPS 2024
([arXiv:2405.15793](https://arxiv.org/abs/2405.15793)), argue that agents need an
interface built for them: a file viewer that shows 100 lines at a time, a search command,
and an edit command gated by a linter. On SWE-bench Lite with one model, the 100-line
window resolved 18.0% of tasks and a view of the whole file 12.7%; the study reports its
cost rather than its tokens. Myeongsoo Kim, Joe Hsu, Dingmin Wang, Shweta Garg, Varun
Kumar and Murali Krishna Ramanathan, *CODESTRUCT: Code Agents over Structured Action
Spaces*, ACL 2026, DOI 10.18653/v1/2026.acl-long.607
([arXiv:2604.05407](https://arxiv.org/abs/2604.05407)), let agents "operate on named AST
entities rather than text spans"; on SWE-bench Verified across six models this "improves
Pass@1 accuracy by 1.2-5.0% while reducing token consumption by 12-38% for most models".
Pengcheng Xu, *Does a Language Server Save Tokens for Coding Agents? A Measurement
Methodology and Preliminary Study*, 2026
([arXiv:2608.13568](https://arxiv.org/abs/2608.13568)), not peer-reviewed, compares tools
at equal task success, by tokens-to-success. Its preliminary study finds that, to locate
a named symbol, a language server costs 6% to 118% more tokens and agents call it 0–6% of
the time; the author's answer is "conditional and usually negative". None of the three
links or checks anything. Each changes the unit an agent reads, a 100-line window, a
named AST entity or a language-server symbol, and each is judged empirically, by agent
task success and tokens.

Retrieval is the older approach. Fengji Zhang et al., *RepoCoder: Repository-Level Code
Completion Through Iterative Retrieval and Generation*, EMNLP 2023, DOI
10.18653/v1/2023.emnlp-main.151 ([arXiv:2303.12570](https://arxiv.org/abs/2303.12570)),
retrieve similar code from the repository in rounds and improve on in-file completion "by
over 10% in all settings". Shuyan Zhou et al., *DocPrompting: Generating Code by
Retrieving the Docs*, ICLR 2023 ([arXiv:2207.05987](https://arxiv.org/abs/2207.05987)),
retrieve documentation pieces for a natural-language intent and raise CodeT5's pass@1 on
CoNaLa by 2.85%, a 52% relative gain. Siru Ouyang et al., *RepoGraph: Enhancing AI
Software Engineering with Repository-level Code Graph*, ICLR 2025
([arXiv:2410.14684](https://arxiv.org/abs/2410.14684)), is a plug-in module that keeps a
repository-level code graph; plugged into four methods on SWE-bench, it raised all of
them. Several of these studies take their tasks from SWE-bench (Carlos E. Jimenez et al.,
ICLR 2024, [arXiv:2310.06770](https://arxiv.org/abs/2310.06770)), while SWE-Bench Pro
(Xiang Deng et al., 2025, [arXiv:2509.16941](https://arxiv.org/abs/2509.16941), not
peer-reviewed) and Doc2Feat-Bench (Zhonghao Jiang et al., ASE 2026,
[arXiv:2507.18130](https://arxiv.org/abs/2507.18130)) pair each task with a requirements
list or a documentation change.

**A trace graph as an agent's context.** Mingyu Chen et al., *TraceDev: A
Traceability-Driven Multi-agent Framework for Requirement-to-Code Development*,
Proceedings of the ACM on Software Engineering (ISSTA 2026), DOI 10.1145/3832171
([arXiv:2607.18886](https://arxiv.org/abs/2607.18886)), has five role agents build code
from use cases. Its Validator agent keeps a traceability graph that links requirements,
design models and code, and that "serves as a structured context for efficient memory
management". The Validator recovers the links itself, by LLM matching from requirement
entities (actors, entry and exit conditions, flow steps) to design entities (classes,
methods, relations, messages) and by AST matching from design entities to code files. It
reports missing links, such as a requirement with no design element, and inconsistencies
between the artifacts. Whether a link survives a moved file or a renamed artifact is not
addressed. The framework reaches success rates of 53.6% on eTOUR and 56.8% on SMOS.

**Context files.** A context file, such as `AGENTS.md` or `CLAUDE.md`, is a Markdown file
of instructions committed to a repository and handed whole to the agent; nothing checks
it. Two controlled studies, both not peer-reviewed, measure what it does. Thibaud
Gloaguen, Niels Mündler-Sasahara, Mark Niklas Müller, Veselin Raychev and Martin Vechev,
*Evaluating AGENTS.md: Are Repository-Level Context Files Helpful for Coding Agents?*,
2026 ([arXiv:2602.11988](https://arxiv.org/abs/2602.11988)), build CTXbench, 138 tasks
from 12 Python repositories with developer-committed context files, beside SWE-bench Lite
tasks given LLM-generated ones. Context files "do not generally improve task success
rates, while increasing inference cost by over 20% on average"; agents follow their
instructions, but repository overviews are "not helpful". Jai Lal Lulla, Seyedmoein
Mohsenimofidi, Matthias Galster, Jie M. Zhang, Sebastian Baltes and Christoph Treude, *On
the Impact of AGENTS.md Files on the Efficiency of AI Coding Agents*, 2026
([arXiv:2601.20404](https://arxiv.org/abs/2601.20404)), run an agent on 124 pull requests
from 10 repositories with and without the file: median wall-clock time falls by 28.64%
and output tokens by 16.58%, but the measures "do not capture whether agent-produced
changes are correct".

Mining studies describe what such files hold. Worawalan Chatlatanagulchai et al., *On the
Use of Agentic Coding Manifests: An Empirical Study of Claude Code*, PROFES 2025, DOI
10.1007/978-3-032-12089-2_40 ([arXiv:2509.14744](https://arxiv.org/abs/2509.14744)), read
253 `CLAUDE.md` files from 242 repositories and find shallow hierarchies, mostly
operational commands, implementation notes and high-level architecture. Their *Agent
READMEs: An Empirical Study of Context Files for Agentic Coding*, 2025
([arXiv:2511.12884](https://arxiv.org/abs/2511.12884)), not peer-reviewed, codes 2,303
files from 1,925 repositories: test procedures appear in 75.9%, implementation details in
70.8% and architecture in 68.1%, and the files "evolve like configuration code".
Seyedmoein Mohsenimofidi, Matthias Galster, Christoph Treude and Sebastian Baltes,
*Context Engineering for AI Agents in Open-Source Software*, MSR 2026, DOI
10.1145/3793302.3793350 ([arXiv:2510.21413](https://arxiv.org/abs/2510.21413)), find
across 466 projects "no established content structure yet". Shaokang Jiang and Daye Nam,
*Beyond the Prompt: An Empirical Study of Cursor Rules*, MSR 2026, DOI
10.1145/3793302.3793367 ([arXiv:2512.18925](https://arxiv.org/abs/2512.18925)), sort the
rules of 401 repositories into five themes: conventions, guidelines, project information,
directives to the model, and examples. Zhijun Gao and Jing Chen, *From Agent Behaviour to
Agent-Friendly Documentation: An Empirical Study of How Coding Agents Discover, Read, and
Write Technical Documentation*, 2026
([arXiv:2608.20195](https://arxiv.org/abs/2608.20195)), not peer-reviewed, trace 557 agent
sessions and 33,097 agentic pull requests: instruction files and working notes are 60.5%
of agents' documentation interactions and API references 1.3%, and where a pull request
changes both, code is touched first 4.7 times more often than documentation.

**Long-context degradation.** Nelson F. Liu, Kevin Lin, John Hewitt, Ashwin Paranjape,
Michele Bevilacqua, Fabio Petroni and Percy Liang, *Lost in the Middle: How Language
Models Use Long Contexts*, TACL 12, 2024, DOI 10.1162/tacl_a_00638
([arXiv:2307.03172](https://arxiv.org/abs/2307.03172)), find performance often highest
when the relevant information is at the start or end of the input, and significantly
lower in the middle, "even for explicitly long-context models". Ali Modarressi et al.,
*NoLiMa: Long-Context Evaluation Beyond Literal Matching*, ICML 2025
([arXiv:2502.05167](https://arxiv.org/abs/2502.05167)), keep the literal overlap between
a question and the passage that answers it minimal: at 32K tokens, 11 of 13 models that
claim at least 128K fall below half their short-context baseline. Yufeng Du et al.,
*Context Length Alone Hurts LLM Performance Despite Perfect Retrieval*, Findings of EMNLP
2025, DOI 10.18653/v1/2025.findings-emnlp.1264
([arXiv:2510.05381](https://arxiv.org/abs/2510.05381)), find performance falling by 13.9%
to 85% as input grows, even when every relevant piece is retrieved and the input stays
within the claimed length. Kelly Hong, Anton Troynikov and Jeff Huber, *Context Rot: How
Increasing Input Tokens Impacts LLM Performance*, a Chroma technical report of 2025
([trychroma.com](https://www.trychroma.com/research/context-rot)), not peer-reviewed,
report the same for 18 models; on LongMemEval, focused prompts of about 300 tokens score
"significantly higher" than the full input of about 113k tokens.

### work.tools: Tools, checked 2026-10-13

Each claim below is what the tool's own documentation states at the version named, read
on 2026-10-13, unless it says that it rests on source code. None of the three stores or
resolves a reference written in a project's files, so whether such a reference survives a
moved file or a reworded title does not apply to any of them.

**Serena** ([oraios/serena](https://github.com/oraios/serena)) is an MCP server that gives
an agent retrieval and editing at the level of code symbols. Its latest release is v1.7.0
(2026-08-09); some pages cited here document its main branch, version 2.0.0.dev0, which
is unreleased. The v1.7.0 [README](https://github.com/oraios/serena/blob/v1.7.0/README.md)
documents "support for over 40 programming languages" through language servers, Markdown
among them, and a paid JetBrains plugin as an alternative analysis backend; Serena is a
client of language servers, not one itself. The main-branch
[language page](https://github.com/oraios/serena/blob/main/docs/01-about/020_programming-languages.md)
says Markdown "must explicitly enable language `markdown`", that several language servers
run "in parallel to support polyglot projects", and that some are downloaded
automatically; it describes no resolution across languages or from prose to code. The
unit Serena addresses is a symbol's name path "within a source file", such as
`MyClass/my_method`. At v1.7.0, `find_symbol` with `include_body` returns that symbol with
its source code
([symbol_tools.py](https://github.com/oraios/serena/blob/v1.7.0/src/serena/tools/symbol_tools.py));
this rests on its source code, not its documentation, whose
[tools page](https://oraios.github.io/serena/01-about/035_tools.html) does not list the
parameter. Its queries run locally against language servers, and its
[privacy page](https://github.com/oraios/serena/blob/main/docs/01-about/070_privacy.md)
says that no data about the project is sent to external servers, but that a usage report
(its version, the operating system and the backends in use) is sent whenever Serena
starts, unless turned off. Its documentation describes no rule about
which kind may link to which, no coverage obligation, no value check and no CI gate:
`serena project health-check` checks the setup
([running](https://github.com/oraios/serena/blob/main/docs/02-usage/020_running.md)). A
Python REPL over its operations is documented for v2 only
([configuration](https://github.com/oraios/serena/blob/main/docs/02-usage/050_configuration.md)).
Its [licence file](https://github.com/oraios/serena/blob/main/LICENSE) names v1.7.0 the
"Last MIT-licensed release"; on main the application is GPL-3.0-or-later and its SolidLSP
component MIT.

**Context7** ([upstash/context7](https://github.com/upstash/context7)), checked at
`@upstash/context7-mcp` 4.3.0 (2026-10-09,
[releases](https://github.com/upstash/context7/releases)), serves current,
version-specific documentation of third-party libraries to an agent over MCP or its
`ctx7` CLI. A hosted indexer parses each library's documentation files, `.md`, `.mdx`,
`.markdown`, `.rst`, `.txt` and `.ipynb`
([adding libraries](https://github.com/upstash/context7/blob/master/docs/adding-libraries.mdx)).
The address is a library ID, `/owner/repo`, which may be pinned to a version; what comes
back from inside the library is chosen by a query, since Context7 "selects relevant
libraries before searching and reranking their snippets"
([API guide](https://github.com/upstash/context7/blob/master/docs/api-guide.mdx)), and the
CLI takes "a library ID and a natural-language question" and returns "relevant code
snippets and explanations"
([CLI](https://github.com/upstash/context7/blob/master/docs/clients/cli.mdx)). It serves
no project's own references, so it keeps no namespace of a project's targets. The MCP
server and the CLI call a hosted API, and the README says that the "API backend, parsing
engine, and crawling engine — are private". A
[VS Code extension](https://github.com/upstash/context7/blob/master/docs/clients/vscode.mdx)
registers its MCP server; there is no language server. Its documentation describes no link
rules, coverage obligation, value check or user rule language (a `context7.json` lets a
library's owner control parsing), and no CI gate: its GitHub Action triggers a
documentation refresh. The repository is MIT-licensed.

**Aider's repository map** ([repository map](https://aider.chat/docs/repomap.html),
[languages](https://aider.chat/docs/languages.html)) was read on the live pages. The
latest GitHub release of [Aider](https://github.com/Aider-AI/aider), which is Apache-2.0,
is v0.86.0 (2025-08-09), and PyPI's `aider-chat` is at 0.86.2 (2026-02-12) with no
matching GitHub release ([releases](https://github.com/Aider-AI/aider/releases),
[PyPI](https://pypi.org/project/aider-chat/)). Aider sends the model, "along with each
change request", a map with "a list of the files in the repo, along with the key symbols
which are defined in each file". The map needs a language's tree-sitter `tags.scm`; the
languages page lists 39 entries with map support, and Markdown is linted but not
mapped. Users write no addresses. The map covers the whole git repository and is chosen
by "a graph ranking algorithm" over files and their dependencies, keeping what is "most
relevant to the current state of the chat" within a token budget, 1k tokens by default:
it is built locally, but what it holds depends on the chat and the budget. Its
documentation describes no command that returns one unit's text, since `--show-repo-map`
prints the whole map
([options](https://github.com/Aider-AI/aider/blob/main/aider/website/docs/config/options.md))
and files enter the chat whole. It describes no editor extension or language server
(`--watch-files` answers comments written in any editor,
[watch](https://github.com/Aider-AI/aider/blob/main/aider/website/docs/usage/watch.md)),
and no link rules, coverage, value check, rule language (a new language needs its
`tags.scm` shared upstream) or CI gate: the map produces context, not findings.

## agrees: Where grund agrees

- What an agent is shown matters, not only which model reads it. SWE-agent's 100-line
  window beat a whole-file view on task success, and CODESTRUCT's named-entity reads beat
  text-based ones on success and, for most models, on tokens. grund makes the same move
  for a repository's prose: `grund <ID>` prints a declaration's lead and
  `grund <ID>.<section>` one section, and a whole body is an explicit escalation
  ([§GOAL-token-economy.1](../goals.md#1-what-this-requires), [§FS-show.2.2](../functional-spec/FS-show.md#22-section)).
- Long inputs are used less well than short ones, even when the relevant text is present,
  as Liu et al., NoLiMa and Du et al. find. That is a reason beyond cost to hand an agent
  the one fact it needs rather than the file around it ([§GOAL-token-economy](../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file)).
- Context files are a channel agents read, and the mining studies at PROFES 2025 and MSR
  2026 find them carrying commands, conventions and architecture. grund's first grounding
  layer uses that channel: `grund init` writes a managed block into `AGENTS.md` and each
  supported agent's own instruction file ([§GOAL-agent-grounding.1](../goals.md#1-the-three-layers), [§FS-init.2.3.12](../functional-spec/FS-init.md#2312-supported-agents-and-their-entrypoints)). The
  block teaches only what an agent needs before work begins and leaves the detail to
  cited declarations read on demand ([§FS-init.2.3](../functional-spec/FS-init.md#23-generated-agent-entrypoints)).
- TraceDev's graph links requirements, design and code and serves agents as context.
  grund's declarations and citations form such a graph, written by authors and held by the
  check ([§GRUND-links](../grund.md#grund-links-the-cross-linking-rules-are-hard-to-define-and-to-hold)).

Agreement is not evidence that grund helps. Two studies, both not peer-reviewed, cut the
other way: Gloaguen et al. find that context files raise cost without a general gain, and
grund's managed block is a context file; Xu finds that a precise read can cost more tokens
than a text search and go unused, which bears on grund's cheap reads too. No study here
measures grund: its goal meters are fixtures, checks and read sizes, not agent task
success or tokens ([§AR-goal-measurement.2](../architecture/AR-goal-measurement.md#2-goal-meters)).

## departs: Where grund departs

- Context7 picks snippets by ranking them against a natural-language question on a
  private backend, Aider's map is picked by graph ranking against the state of the chat
  and a token budget, RepoCoder retrieves by similarity, and DocPrompting retrieves
  documentation for a natural-language intent. A grund read is a verbatim slice of the
  heading tree, chosen by the address the reader names, never a generated summary, and
  byte-identical for the same tree, configuration and invocation ([§FS-non-goals.14](../functional-spec/FS-non-goals.md#14-generated-summaries-token-saving-inside-check),
  [§FS-non-goals.13](../functional-spec/FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)).
- SWE-agent and CODESTRUCT are interfaces through which an agent edits as well as reads,
  and TraceDev coordinates five role agents around its graph. grund is a checker and a
  retriever: it coordinates no agents and drives no workflow ([§FS-non-goals.7](../functional-spec/FS-non-goals.md#7-inter-agent-messaging-or-workflow)).
- TraceDev's links are recovered by an agent, by LLM and AST matching. grund guesses no
  citation; a link exists only where an author wrote one ([§GOAL-agent-grounding.3](../goals.md#3-what-this-rules-out)).
- Serena, CODESTRUCT and Aider address code through its structure, by language servers,
  an AST or tree-sitter, and RepoGraph keeps a graph of the repository's code. grund
  parses no code and addresses only declarations and their sections ([§FS-non-goals.3](../functional-spec/FS-non-goals.md#3-code-ast-parsing)).
- Context7 answers from a hosted service, and Serena downloads some language servers and
  sends a usage report unless turned off. A grund check or query does no network I/O and
  sends no telemetry; fetching is a separate, explicit command ([§FS-non-goals.11](../functional-spec/FS-non-goals.md#11-network-access-during-a-check)).
