/// The `grund fetch --help` page (§FS-cli.2.3, §FS-fetch): the one integration
/// call, its exit codes, and the guide and example that cover external facts.
fn print_fetch_help() {
    println!("grund fetch — materialize one configured external fact snapshot.");
    println!("\nUsage:  grund fetch <ID>\n");
    println!(
        "The selected kind's [[kinds]].fetch executable receives the local ID and returns one declaration, validated before an atomic write."
    );
    println!(
        "No check, query, formatter, completion, or LSP operation runs the integration implicitly."
    );
    println!(
        "\nExit:  0 stored · 1 invalid ID · 2 missing integration, rejected output, or operational error."
    );
    println!();
    println!("Examples:");
    println!("  grund fetch TICKET-1234         # store that ticket's snapshot in its kind's home");
    print_guide_links(
        &["https://github.com/agent-grounds/grund/blob/main/docs/user-facing/external-facts.md"],
        &["https://github.com/agent-grounds/grund/tree/main/examples/external-tickets"],
    );
}
