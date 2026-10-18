/// The `grund refs` page (§FS-cli.2.3, §FS-refs.1): the citation sites of one coordinate.
fn print_refs_help() {
    // §FS-terms.terms.2: a coordinate's citation sites.
    println!("grund refs — list every citation site of a coordinate, as `path:line`, so you can");
    println!("see who depends on a declaration before you change it.");
    println!();
    println!("Usage:  grund refs <ID>[.<section>] [PATH] [--section S] [--descendants]");
    println!("                   [--summary] [--total] [--format text|json]");
    println!();
    println!("PATH defaults to `.`. With a `.<section>` (or --section), only citations of that");
    println!("exact section are listed; --descendants widens that to the section and every");
    println!("section beneath it. An ID with no citations prints nothing and exits 0, or the");
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
    print_run_flag_help("grund refs FS-login src/", true);
    println!();
    println!("The citation list is the result, so it goes to stdout (text and json alike) —");
    println!(
        "`grund refs <ID> | …` works like `grund list`. Only the typo `note:` goes to stderr."
    );
    println!();
    println!(
        "Exit:  0 scan succeeded (with or without hits) · 1 ID rejected after context · 2 run/scan error."
    );
    println!();
    println!("Examples:");
    println!("  grund refs FS-login             # every citation of FS-login");
    println!("  grund refs FS-login --summary   # one row per citing file");
    println!("  grund refs FS-login --total     # how many sites, in how many files");
    println!("  grund refs FS-login.3           # only citations of section 3");
    println!("  grund refs FS-login --format json # one NDJSON record per citation site");
}

/// The `grund cover` page (§FS-cli.2.3, §FS-cover.1): the citation graph by scanned
/// file, or one file's line owners (§FS-cover.6).
fn print_cover_help() {
    println!("grund cover — group the citation graph by scanned file.");
    println!();
    println!("Usage:  grund cover [PATH] [--format text|json]");
    println!("        grund cover FILE --lines N|N-M [--lines …] [--format text|json]");
    println!();
    println!("PATH defaults to `.`. The command runs the same scan as `check` and `refs`,");
    println!("then prints one file record with the citations found in that file.");
    println!("With --lines it prints instead, for each range of FILE, the declaration");
    println!("and section that own its lines.");
    println!();
    println!("Options:");
    println!(
        "  --format text|json   text (default) groups citations by file; json emits one record per file."
    );
    println!(
        "  --lines N|N-M        report the owners of these lines of FILE; repeatable, one record per range."
    );
    print_run_flag_help("grund cover src/", true);
    println!();
    println!("Exit:  0 scan succeeded · 2 unreadable tree, incomplete scan, or CLI error.");
    println!();
    println!("Examples:");
    println!("  grund cover src/                # source files and their spec citations");
    println!("  grund cover --format json       # machine-readable coverage index");
    println!("  grund cover docs/FS-x.md --lines 105-121   # which spec points a diff hunk edits");
}
