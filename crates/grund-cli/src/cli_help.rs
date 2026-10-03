/// `grund --help` / `grund help` — the top-level page: one usage line, the commands
/// grouped by intent, and the public address of the guides (§FS-cli.2.2).
/// `grund help <cmd>` defers to `print_subcommand_help`.
fn print_help() {
    println!("grund — ground your agents in the spec: §<ID>.<section> citations checked across docs and code.");
    println!();
    println!("Usage:  grund <ID>[.<section>]  ·  grund <COMMAND> [ARGS]  ·  grund <COMMAND> -h  ·  grund -V");
    println!();
    println!("Query the catalog:");
    println!("  show          Print a coordinate's lead for agent context (default).  grund FS-login.3");
    println!("  list          Every declared ID, with path:line and title.            grund list --kind FS");
    println!("  refs          Every citation site of a coordinate, as path:line.      grund refs FS-login");
    println!("  cover         Every scanned file, with the citations in it.           grund cover --format json");
    println!();
    println!("Check and author:");
    println!("  check         Scan a repo or subtree and report findings.             grund check .");
    println!("  fmt           Rewrite `$$` to `§`; --marker cites bare IDs.           grund fmt --check");
    println!("  id            Next conflict-free ID for a new declaration.            grund id FS \"user login\"");
    println!("  fetch         Materialize one configured external snapshot.           grund fetch TICKET-1234");
    println!();
    println!("Set up:");
    println!("  init          Scaffold agent instructions + grund.toml.               grund init --docs");
    println!("  config        Validate or show the effective grund.toml.              grund config show");
    println!("  integrations  Print/install clickable-citation integrations.          grund integrations wezterm");
    println!("  completions   Print shell completion scripts.                         grund completions bash");
    println!("  agent-setup-instructions  Print the setup guide for AI agents.");
    println!();
    println!("Guides and examples:  https://github.com/agent-grounds/grund/tree/main/docs/user-facing");
}

/// The closing block of a command page: the public `blob/main` or `tree/main` address
/// of each guide, then each runnable example, that covers the command (§FS-cli.2.3).
fn print_guide_links(guides: &[&str], examples: &[&str]) {
    println!();
    for url in guides {
        println!("Guide:    {url}");
    }
    for url in examples {
        println!("Example:  {url}");
    }
}

/// Per-subcommand `--help` / `help <subcommand>` page (§FS-cli.2.3, §FS-cli.3): what
/// it takes, every flag with a one-line example, the exit codes, and the common
/// recovery path. Goes to stdout, exit 0 — help is never an error.
fn print_subcommand_help(cmd: &str) {
    match cmd {
        "check" => print_check_help(),
        "show" => print_show_help(),
        "fetch" => print_fetch_help(),
        "list" => {
            println!("grund list — the ID catalog: every declared ID in the repo, with where it's");
            println!(
                "declared and its one-line title. The complement of `grund refs` (which lists"
            );
            println!(
                "the citations of one ID) — `list` is the index of what you can read with `grund <ID>`."
            );
            println!();
            println!(
                "Usage:  grund list [PATH] [--kind KIND[,KIND]...] [--selector SUBJECT] [--unused] [--summary]"
            );
            println!(
                "                   [--size[=lines,words,bytes]] [--top N] [--format text|json]"
            );
            println!();
            println!(
                "Output is one line per declared ID, `<ID>  <path>:<line>  <title>`, sorted by ID."
            );
            println!(
                "Stub-and-inline pairs collapse to one line; a duplicate-declared ID gets a line per home."
            );
            println!(
                "Size mode adds every citable section and reports selected lead/full measurement pairs."
            );
            println!();
            println!("Options:");
            println!(
                "  --kind KIND[,KIND]  only selected kinds; repeatable       e.g. grund list --kind FS,AR"
            );
            println!(
                "  --selector SUBJECT  only one declaration/chapter selector, e.g. FS.requirements"
            );
            println!(
                "  --unused            only declarations nothing cites yet (skips E2E unless E2E is selected)"
            );
            println!(
                "  --summary           one row per kind with count and home  e.g. grund list --summary"
            );
            println!(
                "  --size[=lines,words,bytes]  coordinate lead/full sizes; bare selects all three units"
            );
            println!(
                "  --top N             largest N leads by the first selected unit; requires --size"
            );
            println!(
                "  --format text|json   text (default) is the table on stdout; json emits NDJSON (adds `refs` count)."
            );
            println!();
            println!(
                "Exit:  0 scan succeeded (an empty catalog prints nothing) · 2 unreadable tree, or an unknown --kind."
            );
            println!();
            println!("Examples:");
            println!("  grund list                      # the whole catalog");
            println!("  grund list --kind FS,AR docs/   # specs and architecture IDs under docs/");
            println!("  grund list --summary            # counts by kind");
            println!("  grund list --selector FS.requirements # every FS requirements chapter");
            println!("  grund list --size=words --top 10 # largest coordinate leads");
            println!(
                "  grund list --unused             # uncited declarations (specs, decisions, …) — E2E cases excluded"
            );
            println!("  grund list --unused --kind E2E  # uncited e2e cases only, for inventory");
            print_guide_links(
                &["https://github.com/agent-grounds/grund/blob/main/docs/user-facing/coordinate-sizes.md"],
                &[],
            );
        }
        "refs" => {
            // §FS-terms.terms.2: a coordinate's citation sites.
            println!(
                "grund refs — list every citation site of a coordinate, as `path:line`, so you can"
            );
            println!("see who depends on a declaration before you change it.");
            println!();
            println!(
                "Usage:  grund refs <ID>[.<section>] [PATH] [--section S] [--descendants]"
            );
            println!("                   [--summary] [--total] [--format text|json]");
            println!();
            println!(
                "PATH defaults to `.`. With a `.<section>` (or --section), only citations of that"
            );
            println!(
                "exact section are listed; --descendants widens that to the section and every"
            );
            println!(
                "section beneath it. An ID with no citations prints nothing and exits 0, or the"
            );
            println!("line `not cited` under --total.");
            println!();
            println!("Options:");
            println!(
                "  --section S          list only citations of that section path   e.g. grund refs FS-login --section 3"
            );
            println!(
                "  --descendants        also list citations of sections beneath it e.g. grund refs FS-login.3 --descendants"
            );
            println!(
                "  --summary            group citations by citing file             e.g. grund refs FS-login --summary"
            );
            println!(
                "  --total              the set's size instead of its members      e.g. grund refs FS-login --total"
            );
            println!(
                "  --format text|json   text (default) prints `path:line: <citation>`; json emits NDJSON."
            );
            println!();
            println!(
                "The citation list is the result, so it goes to stdout (text and json alike) —"
            );
            println!(
                "`grund refs <ID> | …` works like `grund list`. Only the typo `note:` goes to stderr."
            );
            println!();
            println!(
                "Exit:  0 scan succeeded (with or without hits) · 1 ID rejected after context · 2 run/scan error."
            );
            println!(
                "       Until 0.16.0 rejected IDs still exit 2 with a warning; they move to 1 in 0.16.0."
            );
            println!();
            println!("Examples:");
            println!("  grund refs FS-login             # every citation of FS-login");
            println!("  grund refs FS-login --summary   # one row per citing file");
            println!("  grund refs FS-login --total     # how many sites, in how many files");
            println!("  grund refs FS-login.3           # only citations of section 3");
        }
        "cover" => {
            println!("grund cover — group the citation graph by scanned file.");
            println!();
            println!("Usage:  grund cover [PATH] [--format text|json]");
            println!();
            println!("PATH defaults to `.`. The command runs the same scan as `check` and `refs`,");
            println!("then prints one file record with the citations found in that file.");
            println!();
            println!("Options:");
            println!(
                "  --format text|json   text (default) groups citations by file; json emits one record per file."
            );
            println!();
            println!("Exit:  0 scan succeeded · 2 unreadable tree, incomplete scan, or CLI error.");
            println!();
            println!("Examples:");
            println!("  grund cover src/                # source files and their spec citations");
            println!("  grund cover --format json       # machine-readable coverage index");
        }
        "fmt" => print_fmt_help(),
        "id" => {
            println!("grund id — emit the next conflict-free ID for a new declaration of a kind.");
            println!();
            println!(
                "Usage:  grund id <KIND> \"<title>\" [PATH] [--width N] [--explain] [--format text|json]"
            );
            println!();
            println!(
                "KIND is one of the configured citable `[[kinds]]` — defaults GRUND, GOAL, FS, AR, DF, DA, RM;"
            );
            println!(
                "`grund config show` lists this repo's. The title is slugified deterministically; the number"
            );
            println!("is `max(existing) + 1` (holes are never filled).");
            println!();
            println!("Options:");
            println!(
                "  --width N      minimum digit width for the number (default 3)   e.g. grund id FS \"x\" --width 4"
            );
            println!(
                "  --explain      also print where to put the declaration file     e.g. grund id FS \"x\" --explain"
            );
            println!(
                "  --format text|json   text (default) is the bare ID on stdout; json adds kind/number/slug/folder."
            );
            println!();
            println!(
                "Exit:  0 ID emitted · 1 empty slug / collision · 2 unknown kind, scan, or CLI error."
            );
            println!();
            println!("Examples:");
            println!(
                "  grund id FS \"User can log in\"          # -> FS-007-user-can-log-in (or FS-user-can-log-in)"
            );
            println!(
                "  ID=$(grund id FS \"User can log in\"); $EDITOR \"docs/functional-spec/$ID.md\""
            );
        }
        "init" => {
            println!(
                "grund init — scaffold agent instructions + `grund.toml` (and, with --docs, the docs/ and e2e/ layout)."
            );
            println!(
                "Idempotent: re-running updates the managed agent-instructions block in place and leaves your edits alone."
            );
            println!();
            println!(
                "Usage:  grund init [PATH] [--docs] [--name NAME] [--description TEXT] [--force] [--dry-run] [--check] [--no-vcs] [--agents-md] [--claude] [--gemini] [--pi] [--copilot] [--cursor] [--windsurf] [--zed]"
            );
            println!();
            println!("Options:");
            println!(
                "  --docs         also write docs/ (grund, goals, roadmap, changelog, spec READMEs) and e2e/"
            );
            println!(
                "  --name NAME    project name (default: target project_name, then directory name)"
            );
            println!(
                "  --description TEXT  one-line project description written to grund.toml (shown next to this project in workspace member lists)"
            );
            println!(
                "  --force        rewrite the canonical AGENTS.md and --docs scaffolds; grund.toml is never overwritten"
            );
            println!(
                "  --dry-run      report what would be written/appended/updated without touching any file"
            );
            println!(
                "  --check        the same report, as a gate: writes nothing, exits 1 if anything is pending"
            );
            println!(
                "  --no-vcs       scaffold into a target with no .git/.hg/.jj/.svn above it (refused by default)"
            );
            println!(
                "  --agents-md    create/update canonical AGENTS.md even when another entrypoint exists"
            );
            println!(
                "  --claude       create/update CLAUDE.md and .claude/CLAUDE.md; creates only the first"
            );
            println!("  --gemini       create/update GEMINI.md");
            println!("  --pi           create/update .pi/AGENTS.md");
            println!("  --copilot      create/update .github/copilot-instructions.md");
            println!(
                "  --cursor       create/update .cursor/rules/grund.mdc and legacy .cursorrules; creates only the first"
            );
            println!("  --windsurf     create/update .windsurfrules");
            println!("  --zed          create/update .rules");
            println!();
            println!(
                "Exit:  0 written / updated / already current · 1 --check only: something is still pending · 2 missing or refused target, unknown flag, or unsupported newer block."
            );
            println!();
            println!("Examples:");
            println!("  grund init --docs                  # full first-time scaffold");
            println!("  grund init --dry-run               # preview without writing");
            println!(
                "  grund init --check                 # the same preview as a gate: exit 1 if pending"
            );
            println!(
                "  grund init --name \"My Service\"      # auto-detect entrypoint, else AGENTS.md"
            );
            println!(
                "  grund init --claude --gemini        # create/update both agent entrypoints"
            );
            print_guide_links(
                &["https://github.com/agent-grounds/grund/blob/main/docs/user-facing/init-repo-shapes.md"],
                &["https://github.com/agent-grounds/grund/tree/main/examples"],
            );
        }
        "config" => print_config_help(),
        "completions" => {
            println!("grund completions — print a shell completion script for grund.");
            println!();
            println!("Usage:  grund completions <bash|zsh|fish>");
            println!();
            println!("The generated scripts complete subcommands and complete declared IDs for");
            println!(
                "`grund <ID>`, explicit `show`, and `grund refs <ID>` by calling the hidden helper:"
            );
            println!("`grund complete ids --prefix <word>`.");
            println!();
            println!("Install examples:");
            println!("  source <(grund completions bash)");
            println!("  grund completions zsh > ~/.zfunc/_grund");
            println!("  grund completions fish > ~/.config/fish/completions/grund.fish");
            println!();
            println!("Exit:  0 script printed · 2 unsupported shell.");
        }
        "integrations" => {
            println!(
                "grund integrations — print or install the clickable-citation terminal/editor integrations."
            );
            println!();
            println!("Usage:  grund integrations [<client>] [--write] [--conversation plain|link]");
            println!(
                "                          [--conversation-target file|path|web|vscode|vscodium|cursor]"
            );
            println!(
                "                          [--agent codex|claude|gemini|copilot|zed|pi] [--format text|json]"
            );
            println!();
            println!("With no client, detects the terminal/editor from the environment and prints");
            println!("what applies. With a client — codium, iterm2, kitty, tmux, vscode, or");
            println!("wezterm — prints that integration's snippet and the grund-open resolver;");
            println!("--write installs it as a managed, idempotent block instead of printing. A");
            println!(
                "write also records the local conversation preference and updates global agent"
            );
            println!("instructions. With no client, --write with either conversation flag changes");
            println!("only that preference. --conversation-target picks how a linked citation");
            println!("addresses its declaration; --agent scopes that choice to one agent instead");
            println!("of the machine. --format json emits a plan.");
            println!();
            println!("Preview and install examples:");
            println!("  grund integrations                    # detect and list what applies");
            println!("  grund integrations wezterm            # print the snippet + resolver");
            println!("  grund integrations wezterm --write    # install it, idempotently");
            println!("  grund integrations --write --conversation link  # preference only");
            println!(
                "  grund integrations --write --conversation-target vscodium  # open in the editor"
            );
            println!(
                "  grund integrations --write --agent codex --conversation-target web  # that agent only"
            );
            println!();
            println!(
                "Exit:  0 printed or installed · 2 invalid options, missing write target, or a newer block."
            );
            print_guide_links(
                &["https://github.com/agent-grounds/grund/blob/main/docs/user-facing/clickable-citations.md"],
                &[],
            );
        }
        "agent-setup-instructions" => {
            println!(
                "grund agent-setup-instructions — print the guided setup instructions for AI agents."
            );
            println!();
            println!("Usage:  grund agent-setup-instructions");
            println!();
            println!(
                "The output is the same Markdown source shipped as `skills/grund-init/SKILL.md`,"
            );
            println!("embedded in the binary so installed agents can discover the setup workflow");
            println!("without access to the source tree.");
            println!();
            println!("Exit:  0 instructions printed · 2 unexpected arguments.");
            println!();
            println!("Examples:");
            println!("  grund agent-setup-instructions          # print the setup guide");
            println!("  grund agent-setup-instructions | less   # read it a page at a time");
        }
        _ => print_help(),
    }
}
