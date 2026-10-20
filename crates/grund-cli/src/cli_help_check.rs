/// The complete `grund check --help` discovery surface: all inputs, selected
/// report semantics, the public code catalog, and recovery examples (§FS-cli.3.3).
/// Scope advice stays independent of config discovery (§FS-cli.2.3).
fn print_check_help() {
    println!("grund check — validate every ID citation across the repo.");
    println!();
    println!(
        "Usage:  grund check [PATH] [--watch] [--rule SENTENCE] [--full] [--require-grounding] [--suggestions] [--only CODE] [--ignore CODE] [--only-rule] [--format text|json]"
    );
    println!();
    print_no_path_scope_help();
    println!(
        "Config is discovered by walking up from PATH, or from the current directory with none — the root `grund.toml` is the home, `.agents/grund.toml` a deprecated fallback."
    );
    println!(
        "With no config, grund scans `docs/`, `e2e/`, and `src/`; configure source scope to widen it."
    );
    println!(
        "Pointing grund at an explicit PATH reports exactly that file or directory; citations still resolve against the whole project."
    );
    println!("Path validation is explicit; `grund PATH` is parsed as an ID query.");
    println!();
    println!("Options:");
    // §FS-cli.3.3: the resident flag's discovery and lifecycle contract.
    println!("  --watch              check immediately, then re-check on native filesystem notifications.");
    println!("                       Ctrl-C returns the last completed status (2 before completion).");
    println!("                       Text may use an alternate screen; clean JSON runs emit no record.");
    // §FS-check.2.1: describe the explicit channels and fixed text grouping.
    println!(
        "  --format text|json   text (default) prints `success` or `path:line: <channel>: message`,"
    );
    println!(
        "                       grouped errors, warnings, then enabled suggestions; json emits NDJSON."
    );
    // §FS-declarations.checks.section-outside-declaration.3: this is the one non-reference hard
    // finding the widened scan carries with its ordinary code and message.
    println!(
        "  --full               also walk past configured scope and report unresolved references or section headings outside declaration bodies there; source exclusions still apply."
    );
    println!(
        "  --require-grounding  also require every source file to cite a declared ID (v1 [reference] require_grounding, v2 [rules.citations.grounding] must; a kind that sets it false stays exempt)."
    );
    println!(
        "  --suggestions        also surface should/should-not citation-direction findings (v1 [citations], v2 [rules.citations])."
    );
    println!(
        "  --rule <sentence>    add one controlled-English chapter/citation rule to configured rules."
    );
    println!(
        "  --only <code>        retain one exact finding code; repeat for a union (`--only=<code>` also works)."
    );
    println!(
        "  --ignore <code>      remove one exact finding code; repeat for a union; ignore wins over only (`--ignore=<code>` also works)."
    );
    // §FS-cli.3.3: the third selector on the same footing as the code axis —
    // what it narrows by, and that it refuses to stand alone (§FS-rules.8).
    println!(
        "  --only-rule          retain only what the --rule sentence found; requires --rule."
    );
    print_run_flag_help("grund check docs/", true);
    println!();
    println!(
        "Findings go to stdout (the linter convention) — `grund check | …` and `grund check"
    );
    println!("--format json | jq` need no redirect. Only run-level `error:` / `warning:` lines");
    println!("(unreadable path, empty scan) go to stderr; a clean text run prints `success`.");
    println!();
    // §FS-check.2: state both the post-scan boundary and selected exit meaning.
    println!("Selection happens after the complete check. It filters errors, warnings, and enabled");
    println!("suggestions by exact code; operational failures remain visible and exit 2. Exit 0 means");
    println!("the selected report has no errors, not that the unselected repository is clean.");
    // §FS-check.1.4, §FS-rules.7.6: a narrowed run carries the rules it skipped.
    println!("Selecting a code a rule produces also selects invalid-rule and empty-citation-obligation,");
    println!("so a rule that could not run, or an obligation with nothing to judge, is never a pass");
    println!("unless --ignore removes its line.");
    // §FS-check.1.4.1: a selected check the run leaves off says so in place of `success`.
    println!("A selected code the run leaves off earns one selected-check-off warning naming what");
    println!("turns it on; only --ignore selected-check-off removes it, and the exit is unchanged.");
    println!();
    // §FS-cli.3.3, §FS-rules.8: a trial sentence emits the codes the declared
    // rules emit, so only the authority axis answers what it found — and its
    // `should`-level empty report is the one most likely to be misread.
    println!("--only-rule narrows that same report by rule authority instead of by code: only what");
    println!("the --rule sentence authored, whether it authored a finding alone or jointly with a");
    println!("declared rule that means the same. The two axes intersect and --ignore wins over both.");
    println!("Without --rule it is `error: --only-rule requires --rule`, exit 2. A `should`-level");
    println!("sentence lands in the suggestions channel, so it needs --suggestions too or the scoped");
    println!("run prints `success`.");
    println!();
    println!("Supported check finding codes:");
    for code in Code::ALL {
        println!("  {}", code.as_str());
    }
    // §FS-cli.3.3: where to ask what a code means.
    println!("grund config explain CODE says what a code means, its severity and what turns it on.");
    println!();
    println!(
        "Exit:  0 clean · 1 dangling / duplicate / unknown-section / ungrounded findings · 2 unreadable tree or CLI error."
    );
    println!();
    println!("Examples:");
    println!("  grund check              # check the whole repo");
    println!("  grund check --watch      # live terminal feedback; Ctrl-C to stop");
    println!("  grund check docs/        # check one subtree");
    println!("  grund check --full       # plus unresolved references and outside sections beyond configured scope");
    println!(
        "  grund check --rule \"Each FS should have exactly one requirements chapter.\" --suggestions"
    );
    println!(
        "  grund check --ignore agents-init # ask whether the selected content report has errors"
    );
    println!(
        "  grund check --rule \"Each FS must have exactly one security chapter.\" --only-rule # what does this sentence find?"
    );
    println!("  grund check --format json | jq # machine-readable diagnostics for CI");
    println!("  grund check --only dangling --only duplicate # only those two codes");
    println!("  grund check --require-grounding src/ # every source file must cite a declared ID");
    print_guide_links(
        &[
            "https://github.com/agent-grounds/grund/blob/main/docs/user-facing/watch.md",
            "https://github.com/agent-grounds/grund/blob/main/docs/user-facing/rules.md",
            "https://github.com/agent-grounds/grund/blob/main/docs/user-facing/citation-directions.md",
        ],
        &["https://github.com/agent-grounds/grund/tree/main/examples/rules"],
    );
}
