/// The `grund schema --help` page (§FS-cli.2.3, §FS-schema.1): the formats, the run
/// flag, discovery and the workspace footer, and one example per flag
/// (§FS-cli.2.3.1).
fn print_schema_help() {
    println!("grund schema — the kinds, places, shapes and citation rules a repository is held to.");
    println!();
    println!("Usage:  grund schema [PATH] [--format table|mermaid|dot]");
    println!();
    println!(
        "Reads only the effective grund.toml discovered from PATH: no scan, so a broken tree still shows."
    );
    println!();
    println!("Options:");
    println!("  --format table|mermaid|dot");
    println!("                       table (default): the terminal view, with a legend.");
    println!(
        "                       mermaid, dot: the same facts as Mermaid or Graphviz source; grund runs neither."
    );
    println!("                       e.g. grund schema --format mermaid");
    println!("  --path-base project|invocation");
    println!("                       project (default) spells report paths from the config root;");
    println!(
        "                       invocation: relative to the path you passed, or the current directory if you passed none."
    );
    println!("                       e.g. grund schema docs/ --path-base=invocation");
    println!();
    println!("PATH defaults to `.`; config is discovered by walking up from it.");
    println!("At a workspace root the root project is shown, with one line naming its members.");
    println!();
    println!("Exit:  0 printed · 2 the config couldn't be read, or a usage error.");
    println!();
    println!("Examples:");
    println!("  grund schema                         # the quick terminal view");
    println!("  grund schema --format mermaid        # Mermaid source, e.g. for a GitHub comment");
    println!("  grund schema --format dot | dot -Tsvg > schema.svg");
    println!("  grund schema docs/ --path-base=invocation");
    print_guide_links(
        &["https://github.com/agent-grounds/grund/blob/main/docs/user-facing/schema.md"],
        &["https://github.com/agent-grounds/grund/tree/main/examples/schema"],
    );
}
