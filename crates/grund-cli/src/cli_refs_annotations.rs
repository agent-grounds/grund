/// §FS-refs.3.1.1: every text mode appends the same semantic annotation after
/// the existing row, leaving ordinary uses untouched.
fn render_refs_text(hits: &[RefHit], index_entries: &BTreeSet<(String, usize, usize)>) {
    for hit in hits {
        let suffix = if ref_is_index_entry(hit, index_entries) {
            " (index entry)"
        } else {
            ""
        };
        println!("{}:{}: {}{suffix}", hit.path, hit.line, hit.text);
    }
}

/// §FS-refs.3.1.1: look up the exact site classified by core; the frontend
/// never guesses membership from a filename or a citation's spelling.
fn ref_is_index_entry(hit: &RefHit, entries: &BTreeSet<(String, usize, usize)>) -> bool {
    entries.contains(&(hit.path.clone(), hit.line, hit.column))
}

/// §FS-refs.3.3.1: identify only excluded lines in each file. If an ordinary
/// use shares a line, identify each excluded column on that line too.
fn refs_summary_annotations<'a>(
    hits: &'a [RefHit],
    entries: &BTreeSet<(String, usize, usize)>,
) -> BTreeMap<&'a str, String> {
    let mut excluded: BTreeMap<&str, BTreeMap<usize, BTreeSet<usize>>> = BTreeMap::new();
    let mut ordinary = BTreeSet::new();
    for hit in hits {
        if ref_is_index_entry(hit, entries) {
            excluded
                .entry(&hit.path)
                .or_default()
                .entry(hit.line)
                .or_default()
                .insert(hit.column);
        } else {
            ordinary.insert((hit.path.as_str(), hit.line));
        }
    }
    excluded
        .into_iter()
        .map(|(path, lines)| {
            let locations = lines
                .into_iter()
                .map(|(line, columns)| {
                    if ordinary.contains(&(path, line)) {
                        let columns = columns
                            .into_iter()
                            .map(|column| column.to_string())
                            .collect::<Vec<_>>()
                            .join(", ");
                        format!("line {line}, column {columns}")
                    } else {
                        format!("line {line}")
                    }
                })
                .collect::<Vec<_>>()
                .join("; ");
            (path, format!(" (index entries at {locations})"))
        })
        .collect()
}
