use code_structure_inspector::inspect;
use serde_json::Value;
use std::{
    collections::BTreeSet,
    path::{Component, Path, PathBuf},
};

fn contract() -> Value {
    serde_json::from_str(include_str!("semantic_contract.json")).unwrap()
}

#[test]
fn semantic_fixture_returns_source_indexed_facts_without_backend_meaning_guesses() {
    let c = contract();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(c["fixture_root"].as_str().unwrap());
    let report = inspect(&root).unwrap();
    assert!(!report.coverage.truncated);
    assert_eq!(report.coverage.syntax_error_files, 0);
    assert_eq!(report.files.len(), 3);
    for selected in c["selections"].as_array().unwrap() {
        let file = report
            .files
            .iter()
            .find(|f| f.path == selected["path"].as_str().unwrap())
            .unwrap();
        assert!(
            file.symbols
                .iter()
                .any(|symbol| symbol.name == selected["symbol"].as_str().unwrap()
                    && symbol.start_line == selected["start_line"].as_u64().unwrap() as usize
                    && symbol.end_line == selected["end_line"].as_u64().unwrap() as usize)
        );
    }
    let output = serde_json::to_value(&report).unwrap();
    assert_eq!(output["dependencies"], c["import_facts"]);
    assert!(output.get("responsibilities").is_none());
    assert!(output.get("interpretations").is_none());
    assert_eq!(output["analysis"], "syntactic_evidence");
}

#[test]
fn selected_semantic_evidence_and_diagram_contract_are_bounded_and_traceable() {
    let c = contract();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(c["fixture_root"].as_str().unwrap());
    let mut files = BTreeSet::new();
    let mut ids = BTreeSet::new();
    let mut total_lines = 0;
    for selected in c["selections"].as_array().unwrap() {
        let path = selected["path"].as_str().unwrap();
        assert!(
            Path::new(path)
                .components()
                .all(|part| matches!(part, Component::Normal(_)))
        );
        assert!(ids.insert(selected["id"].as_str().unwrap()));
        files.insert(path);
        let source = std::fs::read_to_string(root.join(path)).unwrap();
        let lines: Vec<_> = source.lines().collect();
        let first = selected["start_line"].as_u64().unwrap() as usize;
        let last = selected["end_line"].as_u64().unwrap() as usize;
        assert!(first >= 1 && last >= first && last <= lines.len());
        let count = last - first + 1;
        assert!(count <= c["source_budget"]["max_lines_per_range"].as_u64().unwrap() as usize);
        total_lines += count;
        let evidence = lines[first - 1..last].join("\n");
        for operation in selected["observed_operations"].as_array().unwrap() {
            assert!(evidence.contains(operation.as_str().unwrap()));
        }
    }
    assert!(files.len() <= c["source_budget"]["max_files"].as_u64().unwrap() as usize);
    assert!(total_lines <= c["source_budget"]["max_lines_total"].as_u64().unwrap() as usize);
    for role in c["interpretations"].as_array().unwrap() {
        assert!(files.contains(role["component"].as_str().unwrap()));
        assert!(!role["evidence"].as_array().unwrap().is_empty());
        for id in role["evidence"].as_array().unwrap() {
            assert!(ids.contains(id.as_str().unwrap()));
        }
    }
    for edge in c["diagram"]["meaning_edges"].as_array().unwrap() {
        assert!(
            c["import_facts"]
                .as_array()
                .unwrap()
                .iter()
                .any(|fact| fact["from"] == edge["from"] && fact["resolved"] == edge["to"])
        );
        let selected = c["selections"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["id"] == edge["source"])
            .unwrap();
        assert!(edge["line"].as_u64().unwrap() >= selected["start_line"].as_u64().unwrap());
        assert!(edge["line"].as_u64().unwrap() <= selected["end_line"].as_u64().unwrap());
    }
    assert_eq!(
        c["diagram"]["candidate_impact"]["guaranteed_breakage"],
        false
    );
    assert_eq!(c["diagram"]["source_snippets_in_payload"], false);
    assert_eq!(c["diagram"]["private_values_in_payload"], false);
    assert_eq!(c["automated_test_is_a_gpt_quality_proof"], false);
}
