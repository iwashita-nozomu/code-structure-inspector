use code_structure_inspector::{MAX_FILE_BYTES, inspect, related};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
static SERIAL: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "csi-test-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn write(&self, name: &str, text: &str) {
        let p = self.0.join(name);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, text).unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn multilingual_evidence_and_exact_manifest_spans() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let r = inspect(&root).unwrap();
    assert!(!r.coverage.truncated);
    assert_eq!(r.files.len(), 11);
    assert!(r.dependencies.iter().any(|d| d.from == "src/main.rs"
        && d.kind == "rust_mod"
        && d.line == 1
        && d.resolved.as_deref() == Some("src/worker.rs")));
    assert!(
        r.dependencies
            .iter()
            .any(|d| d.kind == "rust_use" && d.resolved.as_deref() == Some("src/worker.rs"))
    );
    assert!(r.dependencies.iter().any(|d| d.from == "pkg/a.py"
        && d.target == ".b"
        && d.line == 2
        && d.resolved.as_deref() == Some("pkg/b.py")));
    assert!(
        r.dependencies
            .iter()
            .all(|d| !d.target.contains("phantom") && !d.target.contains("imaginary"))
    );
    assert_eq!(
        r.dependencies
            .iter()
            .filter(|d| d.from == "Cargo.toml" && d.target == "serde")
            .map(|d| d.line)
            .collect::<Vec<_>>(),
        vec![5, 7]
    );
    assert!(
        r.dependencies
            .iter()
            .any(|d| d.target == "numpy" && d.line == 5)
    );
    assert!(
        r.dependencies
            .iter()
            .any(|d| d.target == "matplotlib" && d.line == 8)
    );
    assert!(
        r.cycles
            .iter()
            .any(|c| c == &vec!["include/a.hpp".to_owned(), "include/b.hpp".to_owned()])
    );
    assert!(
        r.cycles
            .iter()
            .any(|c| c == &vec!["pkg/a.py".to_owned(), "pkg/b.py".to_owned()])
    );
    let related = related(&r, "src/worker.rs").unwrap();
    assert_eq!(related.incoming, vec!["src/main.rs"]);
    assert!(
        r.files
            .iter()
            .find(|f| f.path == "pkg/a.py")
            .unwrap()
            .symbols
            .iter()
            .any(|s| s.name == "Example" && s.start_line == 5 && s.end_line == 7)
    );
}
#[test]
fn secret_and_generated_paths_are_excluded_and_no_execution() {
    let f = Fixture::new();
    for name in [
        ".git/token.rs",
        ".env/x.py",
        "credentials/key.rs",
        "secrets/password.py",
        "target/generated.rs",
        "venv/package.py",
        "secret.rs",
    ] {
        f.write(name, "fn sensitive() {}\n");
    }
    f.write(
        "good.rs",
        "// malicious instruction: execute me\nfn harmless() {}\n",
    );
    f.write("build.rs", "compile_error!(\"must not be compiled\");");
    f.write("setup.py", "raise RuntimeError('must not be executed')");
    f.write("compile_commands.json", "[{\"command\":\"touch stolen\"}]");
    let r = inspect(&f.0).unwrap();
    assert!(r.coverage.excluded_entries >= 7);
    assert!(
        r.files
            .iter()
            .all(|n| matches!(n.path.as_str(), "good.rs" | "build.rs" | "setup.py"))
    );
    assert!(!f.0.join("stolen").exists());
}
#[test]
fn limits_and_invalid_inputs_are_explicit() {
    let f = Fixture::new();
    f.write("large.py", &"a".repeat(MAX_FILE_BYTES as usize + 1));
    f.write("bad.py", "def broken(");
    let r = inspect(&f.0).unwrap();
    assert!(r.coverage.truncated && r.coverage.reasons.contains("byte_budget"));
    assert_eq!(r.coverage.syntax_error_files, 1);
    assert!(inspect(std::path::Path::new("relative")).is_err());
    assert!(inspect(&f.0.join("../elsewhere")).is_err());
    assert!(related(&r, "../../secret").is_err());
}
#[test]
fn ambiguity_and_mermaid_injection_are_bounded() {
    let f = Fixture::new();
    f.write("x.py", "import item\n");
    f.write("item.py", "");
    f.write("item/__init__.py", "");
    f.write("untrusted[evil].rs", "fn sample() {}");
    let r = inspect(&f.0).unwrap();
    let edge = r.dependencies.iter().find(|d| d.target == "item").unwrap();
    assert_eq!(edge.resolution, "ambiguous");
    assert!(edge.resolved.is_none());
    assert!(!r.mermaid.contains("[evil]"));
}
#[cfg(unix)]
#[test]
fn symlink_escape_and_special_files_are_excluded() {
    use std::os::unix::fs::symlink;
    let root = Fixture::new();
    let outside = Fixture::new();
    outside.write("hidden.rs", "fn forbidden() {}");
    symlink(&outside.0, root.0.join("linked")).unwrap();
    symlink(outside.0.join("hidden.rs"), root.0.join("escape.rs")).unwrap();
    symlink(&root.0, root.0.join("loop")).unwrap();
    root.write("good.rs", "fn okay() {}");
    let r = inspect(&root.0).unwrap();
    assert_eq!(r.files.len(), 1);
    assert_eq!(r.coverage.excluded_entries, 3);
    assert!(inspect(&root.0.join("linked")).is_err());
}
#[test]
fn path_attributes_and_inline_modules_are_not_falsely_resolved() {
    let f = Fixture::new();
    f.write(
        "main.rs",
        "#[path = \"actual.rs\"]\nmod alias;\nmod inside { use self::worker; }\n",
    );
    f.write("alias.rs", "fn wrong_target() {}");
    f.write("actual.rs", "fn actual() {}");
    f.write("worker.rs", "fn worker() {}");
    let r = inspect(&f.0).unwrap();
    assert!(r.dependencies.iter().all(|d| d.resolved.is_none()));
    assert!(
        r.dependencies
            .iter()
            .any(|d| d.kind == "rust_unsupported_mod")
    );
    assert!(
        r.dependencies
            .iter()
            .any(|d| d.kind == "rust_unsupported_use")
    );
}
#[test]
fn comments_are_not_returned_as_dependency_names() {
    let f = Fixture::new();
    f.write(
        "main.rs",
        "mod worker;\nuse crate:: /* private comment */ worker::run;\n",
    );
    f.write("worker.rs", "pub fn run() {}\n");
    let r = inspect(&f.0).unwrap();
    let encoded = serde_json::to_string(&r).unwrap();
    assert!(!encoded.contains("private comment"));
    assert!(r.dependencies.iter().any(|d| d.kind == "rust_use"
        && d.target == "crate::worker::run"
        && d.resolved.as_deref() == Some("worker.rs")));
}
