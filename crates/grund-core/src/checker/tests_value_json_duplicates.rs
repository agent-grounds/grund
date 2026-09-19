//! Duplicate JSON value authority wins over binding comparison (§FS-values.2.3).

use super::check_findings;
use crate::config::load_config;
use crate::scanner::scan_tree;
use crate::testing::{test_root, write};

#[test]
fn every_json_duplicate_shape_reports_all_sites_without_comparing_a_winner() {
    let cases = [
        (
            "repeated",
            false,
            vec![(
                "values/a.json",
                "{\n  \"CONST-price\": [1],\n  \"CONST-price\": [2]\n}\n",
            )],
            vec![("values/a.json", 2), ("values/a.json", 3)],
        ),
        (
            "cross_file",
            false,
            vec![
                ("values/a.json", "{\"CONST-price\":[1]}\n"),
                ("values/b.json", "{\"CONST-price\":[2]}\n"),
            ],
            vec![("values/a.json", 1), ("values/b.json", 1)],
        ),
        (
            "markdown_json",
            false,
            vec![
                ("values/a.json", "{\"CONST-price\":[1]}\n"),
                ("values/b.md", "# CONST-price: Price\n\n## 1. 2\n"),
            ],
            vec![("values/a.json", 1), ("values/b.md", 1)],
        ),
        (
            "shared_home",
            true,
            vec![("values/a.json", "{\"CONST-price\":[1]}\n")],
            vec![("values/a.json", 1), ("values/a.json", 1)],
        ),
    ];
    for (name, shared, files, expected_sites) in cases {
        let root = test_root(&format!("json_duplicate_precedence_{name}"))
            .canonicalize()
            .expect("canonical fixture root");
        let second_home = if shared {
            "[[kinds]]\nkind = \"RATE\"\nfolder = \"values\"\nindex = false\nvalues = true\n"
        } else {
            ""
        };
        write(
            &root.join("grund.toml"),
            &format!(
                "grund_config_version = 1\n\n\
             [reference]\nstrict = true\n\n\
             [id]\nformat = \"{{kind}}-{{slug}}\"\nslug_pattern = \"[a-z][a-z-]*\"\n\n\
             [[kinds]]\nkind = \"CONST\"\nfolder = \"values\"\nindex = false\nvalues = true\n\n\
             {second_home}\n[scan]\ninclude = [\"docs\"]\n"
            ),
        );
        for (path, contents) in files {
            write(&root.join(path), contents);
        }
        // §FS-values.2.3: 999 disagrees with every candidate, so picking any winner is observable.
        write(
            &root.join("docs/use.md"),
            "Price: `999` (§CONST-price.1).\n",
        );
        let config = load_config(&root).expect("load duplicate fixture");
        let (findings, errors) = scan_tree(&config, Some(&root), true).expect("scan duplicates");
        assert!(errors.is_empty(), "{name}: {errors:?}");
        assert_eq!(
            findings.value_bindings.len(),
            1,
            "{name}: binding must be exercised"
        );
        let report = check_findings(&findings, &config);
        assert_eq!(
            report
                .errors
                .iter()
                .map(|diagnostic| diagnostic.code)
                .collect::<Vec<_>>(),
            ["duplicate"],
            "{name}: duplicate must suppress comparison, including value-mismatch"
        );
        let duplicate = &report.errors[0];
        assert_eq!(
            duplicate
                .sites
                .iter()
                .map(|site| (site.path.clone(), site.line))
                .collect::<Vec<_>>(),
            expected_sites
                .iter()
                .map(|(path, line)| (root.join(path), *line))
                .collect::<Vec<_>>(),
            "{name}: retain every declaration site, even coincident home claims"
        );
        assert_eq!(duplicate.path, Some(root.join(expected_sites[0].0)));
        assert_eq!(duplicate.line, Some(expected_sites[0].1));
        assert_eq!(duplicate.column, None);
        let others = expected_sites[1..]
            .iter()
            .map(|(path, line)| format!("{path}:{line}"))
            .collect::<Vec<_>>()
            .join(", ");
        assert_eq!(
            duplicate.message,
            format!("duplicate declaration of CONST-price (also declared at {others})")
        );
    }
}
