/// `grund --help` / `grund help` — the top-level page: one usage line, the commands
/// grouped by intent, and the public address of the guides (§FS-cli.2.2).
/// `grund help <cmd>` defers to `print_subcommand_help`.
fn print_help() {
    println!("grund — ground your agents in the spec: §<ID>.<section> citations checked across docs and code.");
    println!("Usage:  grund <ID>[.<section>]  ·  grund <COMMAND> [ARGS]  ·  grund <COMMAND> -h  ·  grund -V");
    println!();
    println!("Query the catalog:");
    println!("  show          Print a coordinate's lead for agent context (default).  grund FS-login.3");
    println!("  list          Every declared ID, or the ones nothing cites.           grund list --unused");
    println!("  refs          Every citation site of a coordinate, as path:line.      grund refs FS-login");
    println!("  cover         Every scanned file, with the citations in it.           grund cover --format json");
    println!();
    println!("Check and author:");
    println!("  check         Scan a repo or subtree and report findings.             grund check .");
    println!("  fmt           Normalize citations; maintain links and section locks.  grund fmt --check");
    println!("  id            Next conflict-free ID for a new declaration.            grund id FS \"user login\"");
    println!("  fetch         Materialize one configured external snapshot.           grund fetch TICKET-1234");
    println!();
    println!("Set up:");
    println!("  guide         How to set up and use grund; lists its sections.        grund guide");
    println!("  init          Scaffold agent instructions + grund.toml.               grund init --scaffold");
    println!("  config        Validate or show grund.toml; explain a key or a code.   grund config show");
    println!("  schema        Kinds, places, shapes and citation rules at a glance.   grund schema");
    // The row names grund-lsp for LSP editors (§FS-integrations.2.3).
    println!("  integrations  Clickable-citation clients; LSP editors use grund-lsp.  grund integrations wezterm");
    println!("  completions   Print shell completion scripts.                         grund completions bash");
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

/// §FS-cli.2.3.2: the one line, worded once, that each page whose PATH narrows the run
/// (`check`, `fmt`, `cover`, `list`, `refs`) prints below its usage: what a run given no
/// path covers, in place of a default of `.` that is false below the root.
fn print_no_path_scope_help() {
    println!(
        "With no PATH, the run covers the whole project or workspace discovered from the current directory; a PATH narrows it."
    );
}

/// §FS-cli.3.4: the run-flag lines of every command that takes `--path-base`, naming
/// the flag that replaces each `[output]` key; the commands that honour
/// `[output] format` also name `--format json` as its replacement (§FS-cli.3).
/// `invocation` is the page's own command line the flag's example extends, so each
/// page shows the flag on the command it documents (§FS-cli.2.3.1).
fn print_run_flag_help(invocation: &str, honours_format: bool) {
    println!("  --path-base project|invocation");
    println!("                       project (default) spells report paths from the config root;");
    println!(
        "                       invocation: relative to the path you passed, or the current directory if you passed none."
    );
    println!("                       e.g. {invocation} --path-base=invocation");
    println!(
        "                       --path-base=invocation replaces a version-1 `[output] relative_paths = false`."
    );
    if honours_format {
        println!(
            "                       --format json replaces a version-1 `[output] format = \"json\"`; a flag outranks its key."
        );
    }
}

/// Per-subcommand `--help` / `help <subcommand>` page (§FS-cli.2.3, §FS-cli.3): what
/// it takes, every flag with a one-line example (§FS-cli.2.3.1), the exit codes, and the common
/// recovery path. Goes to stdout, exit 0 — help is never an error.
fn print_subcommand_help(cmd: &str) {
    match cmd {
        "check" => print_check_help(),
        "show" => print_show_help(),
        "fetch" => print_fetch_help(),
        "list" => print_list_help(),
        "refs" => print_refs_help(),
        "cover" => print_cover_help(),
        "fmt" => print_fmt_help(),
        "id" => {
            println!("grund id — emit the next conflict-free ID for a new declaration of a kind.");
            println!();
            println!(
                "Usage:  grund id <KIND> \"<title>\" [PATH] [--width N] [--explain] [--format text|json]"
            );
            println!();
            println!(
                "KIND is one of the configured citable kinds (v2 `[schema.kinds.<NAME>]`, v1 `[[kinds]]`) — defaults GRUND, GOAL, FS, AR, DF, DA, RM;"
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
            print_run_flag_help("grund id FS \"x\" docs/ --explain", false);
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
            println!(
                "  grund id FS \"User can log in\" --format json  # the ID with its kind, number, slug, folder"
            );
        }
        "init" => {
            println!(
                "grund init — scaffold agent instructions + a template's `grund.toml` (and, with --scaffold, its starter files)."
            );
            // §FS-init.2.4.3: the file holds only what differs from a default;
            // §FS-config.4.2.3.1: what sends a reader to `show` names `explain`.
            println!(
                "The `grund.toml` it writes is a minimal version-2 file; `grund config show` prints what applies, `grund config explain` every setting."
            );
            println!(
                "Idempotent: re-running updates the managed agent-instructions block in place and leaves your edits alone."
            );
            println!();
            println!(
                "Usage:  grund init [PATH] [--template NAME] [--scaffold] [--name NAME] [--description TEXT] [--force] [--dry-run] [--check] [--no-vcs] [--agents-md] [--claude] [--gemini] [--pi] [--copilot] [--cursor] [--windsurf] [--zed]"
            );
            println!();
            println!("Options:");
            println!(
                "  --template NAME  software-development (default: the canonical kinds) or none (no kinds, for a repo that declares its own)"
            );
            println!(
                "  --scaffold     also write the template's starter files: docs/ and tests/"
            );
            println!(
                "  --name NAME    project name (default: target project_name, then directory name)"
            );
            println!(
                "  --description TEXT  one-line project description written to grund.toml (shown next to this project in workspace member lists)"
            );
            println!(
                "  --force        rewrite the canonical AGENTS.md and the --scaffold starter files; grund.toml is never overwritten"
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
                "Exit:  0 written / updated / already current · 1 --check only: something is still pending · 2 missing or refused target, unknown flag or template, or unsupported newer block."
            );
            println!();
            println!("Examples:");
            println!("  grund init --scaffold              # full first-time scaffold");
            println!(
                "  grund init --template=none         # a grund.toml with no kinds, for a repo that declares its own"
            );
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
            println!(
                "  grund init --agents-md --cursor     # AGENTS.md beside the Cursor rules"
            );
            println!(
                "  grund init --pi --copilot --windsurf --zed  # those agents' entrypoints too"
            );
            println!(
                "  grund init --description \"Billing service\"  # the one-liner workspace lists show"
            );
            println!(
                "  grund init --scaffold --force      # rewrite AGENTS.md and the starter files"
            );
            println!(
                "  grund init --no-vcs ../scratch     # scaffold a directory no VCS tracks"
            );
            print_guide_links(
                &[
                    "https://github.com/agent-grounds/grund/blob/main/docs/user-facing/init.md",
                    "https://github.com/agent-grounds/grund/blob/main/docs/user-facing/init-repo-shapes.md",
                ],
                &["https://github.com/agent-grounds/grund/tree/main/examples/init-template-none"],
            );
        }
        "config" => print_config_help(),
        "schema" => print_schema_help(),
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
            // Editor navigation is grund-lsp's, not these clients' (§FS-integrations.2.3).
            println!(
                "Go-to-definition, hover and completion in an editor come from grund-lsp, not"
            );
            println!("from these integrations; IntelliJ-family editors get them through");
            println!("`grund-lsp integrations lsp4ij`.");
            println!();
            println!("Options:");
            print_run_flag_help("grund integrations", false);
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
            println!("  grund integrations wezterm --format json  # the same, as a JSON plan");
            println!();
            println!(
                "Exit:  0 printed or installed · 2 invalid options, missing write target, or a newer block."
            );
            // The setup guide, then the LSP guide, as detection names them (§FS-integrations.2.3).
            print_guide_links(&[SETUP_GUIDE_URL, LSP_GUIDE_URL], &[]);
        }
        // §FS-init.5, §FS-cli.2.3.1: the section names are read from the one table.
        "guide" => {
            println!("grund guide — how to set up and use grund, one section at a time.");
            println!();
            println!("Usage:  grund guide [SECTION | --full]");
            println!();
            println!("With no argument it prints the front page: the three commands a fresh repository");
            println!("runs, then every section with its size and what it holds. A fresh repository");
            println!("needs only that; adopting grund in any other one reads `setup` next.");
            println!();
            println!("Arguments:");
            println!("  SECTION  print one section, e.g. grund guide setup");
            println!();
            println!("Options:");
            println!("  --full   print every section, `setup` first: the whole guide at once");
            println!("           e.g. grund guide --full | less");
            println!();
            let names: Vec<&str> = guide_sections().iter().map(|section| section.name).collect();
            println!("Sections, in --full order:");
            println!("  {}", names.join(", "));
            println!();
            println!("The text is the Markdown shipped as `skills/grund-init/`: `SKILL.md` is the");
            println!("`setup` section and `references/<name>.md` each other one. It is embedded in the");
            println!("binary so installed agents can discover the setup workflow without the source tree.");
            println!();
            println!("Exit:  0 printed · 2 unknown section, unknown flag, or other CLI error.");
            println!();
            println!("Examples:");
            println!("  grund guide                # the front page: fresh path, then every section");
            println!("  grund guide setup          # adopting grund in a repository that is not fresh");
            println!("  grund guide rules          # one reference section");
            println!("  grund guide --full | less  # the whole guide, a page at a time");
        }
        _ => print_help(),
    }
}
