use code_structure_inspector::inspect;
use std::path::PathBuf;

fn assert_declarations(path: &str, expected: &[(&str, &str, usize, usize)]) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/declaration-fixtures");
    let report = inspect(&root).unwrap();
    assert!(!report.coverage.truncated);
    let file = report.files.iter().find(|file| file.path == path).unwrap();
    for &(name, kind, start, end) in expected {
        let symbol = file
            .symbols
            .iter()
            .find(|symbol| symbol.name == name && symbol.kind == kind)
            .unwrap_or_else(|| {
                panic!(
                    "Missing {kind} {name} in {path}; observed: {:?}",
                    file.symbols
                )
            });
        assert_eq!(
            (symbol.start_line, symbol.end_line),
            (start, end),
            "Wrong range for {kind} {name} in {path}"
        );
    }
    // All recognized declaration kinds in each fixture are covered by explicit witnesses.
    assert_eq!(
        file.symbols.len(),
        expected.len(),
        "Unexpected declaration set in {path}"
    );
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
