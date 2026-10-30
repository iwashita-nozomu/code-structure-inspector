use code_structure_inspector::inspect;
use serde_json::Value;
use std::{fs, path::Path, time::SystemTime};

#[test]
fn exact_read_bytes_pin_ranges_and_same_length_or_newline_changes_invalidate_revision() {
    let root = std::env::temp_dir().join(format!(
        "structure-revision-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    let path = root.join("source.py");
    fs::write(&path, "abc").unwrap();
    let report = inspect(&root).unwrap();
    assert_eq!(
        report.files[0].content_revision,
        "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        report.files[0].content_revision,
        inspect(&root).unwrap().files[0].content_revision
    );
    fs::write(&path, "abd").unwrap();
    let changed = inspect(&root).unwrap();
    assert_eq!(report.files[0].lines, changed.files[0].lines);
    assert_ne!(
        report.files[0].content_revision,
        changed.files[0].content_revision
    );
    fs::write(&path, "abc\n").unwrap();
    assert_eq!(
        inspect(&root).unwrap().files[0].content_revision,
        "sha256:edeaaff3f1774ad2888673770c6d64097e391bc362d7d6fb34982ddf0efd18cb"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn selected_fixture_registry_matches_real_report_and_all_ranges_stay_at_that_revision() {
    let fixture: Value = serde_json::from_str(include_str!("structure_review.json")).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/semantic-fixtures");
    let report = inspect(&root).unwrap();
    for file in fixture["view"]["files"].as_array().unwrap() {
        let observed = &report.files[file["file_index"].as_u64().unwrap() as usize];
        assert_eq!(observed.path, file["path"].as_str().unwrap());
        assert_eq!(
            observed.content_revision,
            file["content_revision"].as_str().unwrap()
        );
        assert_eq!(observed.lines as u64, file["lines"].as_u64().unwrap());
    }
    for target in fixture["view"]["targets"].as_array().unwrap() {
        for range in target["anchors"].as_array().unwrap() {
            let file = &report.files[range["file_index"].as_u64().unwrap() as usize];
            let first = range["start_line"].as_u64().unwrap() as usize;
            let last = range["end_line"].as_u64().unwrap() as usize;
            assert!(first > 0 && first <= last && last <= file.lines);
            assert!(
                file.symbols
                    .iter()
                    .any(|s| s.start_line == first && s.end_line == last)
                    || report
                        .dependencies
                        .iter()
                        .any(|d| d.from == file.path && d.line == first && first == last)
            );
        }
    }
    assert_eq!(fixture["cases"][0]["input"], "このレポを見て");
    assert_eq!(fixture["overview"]["automatic_followups"], 0);
    assert_eq!(fixture["live_host_GPT_or_click_proven"], false);
}

#[test]
fn include_only_evidence_is_a_literal_dependency_not_an_observed_runtime_call() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/selection-fixtures");
    let report = inspect(&root).unwrap();
    assert_eq!(report.dependencies.len(), 1);
    let witness = &report.dependencies[0];
    assert_eq!(witness.from, "include_only.cpp");
    assert_eq!(witness.line, 1);
    assert_eq!(witness.resolved.as_deref(), Some("declaration.hpp"));
    assert_eq!(witness.kind, "cpp_quoted_include");
    assert_eq!(witness.resolution, "literal_relative_path");
    let output = serde_json::to_value(report).unwrap();
    assert!(output.get("calls").is_none());
    assert!(output.get("interpretations").is_none());
}
