use code_structure_inspector::inspect;
use std::path::PathBuf;

fn assert_declarations(path: &str, expected: &[(&str, &str, usize, usize)]) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/declaration-fixtures");
    let report = inspect(&root).unwrap();
    assert!(!report.coverage.truncated);
    let file = report.files.iter().find(|file| file.path == path).unwrap();
    let observed: Vec<_> = file
        .symbols
        .iter()
        .map(|symbol| {
            (
                symbol.name.as_str(),
                symbol.kind.as_str(),
                symbol.start_line,
                symbol.end_line,
            )
        })
        .collect();
    // Exact ordered witnesses also distinguish specializations with a shared base name.
    assert_eq!(
        observed, expected,
        "Wrong declarations or source ranges in {path}"
    );
    for file in &report.files {
        assert!(
            root.join(&file.path).is_file(),
            "Source ID does not round-trip: {}",
            file.path
        );
    }
}

#[test]
fn rust_module_function_and_all_type_declarations_have_names_and_ranges() {
    assert_declarations(
        "boundaries.rs",
        &[
            ("boundary", "mod_item", 1, 14),
            ("Worker", "struct_item", 2, 4),
            ("State", "enum_item", 5, 7),
            ("Runner", "trait_item", 8, 10),
            ("Count", "type_item", 11, 11),
            ("execute", "function_item", 12, 13),
        ],
    );
}

#[test]
fn cpp_namespace_class_struct_enum_and_functions_have_names_and_ranges() {
    assert_declarations(
        "boundaries.hpp",
        &[
            ("analysis", "namespace_definition", 1, 17),
            ("Model", "class_specifier", 2, 7),
            ("run", "function_definition", 4, 6),
            ("Data", "struct_specifier", 8, 10),
            ("Mode", "enum_specifier", 11, 13),
            ("evaluate", "function_definition", 14, 16),
            ("Box", "class_specifier", 19, 20),
            ("Box", "class_specifier", 22, 23),
            ("Holder", "struct_specifier", 25, 26),
            ("Holder", "struct_specifier", 28, 29),
            ("outer::inner", "namespace_definition", 30, 34),
            ("Nested", "enum_specifier", 31, 33),
            ("extra", "function_definition", 35, 37),
            ("analysis", "namespace_definition", 38, 38),
            ("Forward", "struct_specifier", 38, 38),
            ("Forward", "struct_specifier", 39, 41),
        ],
    );
}

#[test]
fn python_class_and_functions_have_names_and_ranges() {
    assert_declarations(
        "boundaries.py",
        &[
            ("Experiment", "class_definition", 1, 3),
            ("run", "function_definition", 2, 3),
            ("helper", "function_definition", 5, 6),
        ],
    );
}

#[test]
fn cuda_cpp_fallback_retains_namespace_struct_and_kernel_witnesses() {
    assert_declarations(
        "boundaries.cu",
        &[
            ("device", "namespace_definition", 1, 8),
            ("Buffer", "struct_specifier", 2, 4),
            ("update", "function_definition", 5, 7),
        ],
    );
}

#[test]
fn cpp_fixture_exercises_all_pinned_declaration_name_field_variants() {
    use std::collections::BTreeSet;
    let source = include_str!("declaration-fixtures/boundaries.hpp");
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_cpp::LANGUAGE.into())
        .unwrap();
    let tree = parser.parse(source, None).unwrap();
    assert!(!tree.root_node().has_error());
    let mut stack = vec![tree.root_node()];
    let mut kinds = BTreeSet::new();
    while let Some(node) = stack.pop() {
        if matches!(
            node.kind(),
            "class_specifier" | "struct_specifier" | "enum_specifier" | "namespace_definition"
        ) && let Some(name) = node.child_by_field_name("name")
        {
            kinds.insert(name.kind());
        }
        let mut cursor = node.walk();
        stack.extend(node.named_children(&mut cursor));
    }
    assert_eq!(
        kinds,
        BTreeSet::from([
            "type_identifier",
            "template_type",
            "namespace_identifier",
            "nested_namespace_specifier",
            "qualified_identifier"
        ])
    );
}
