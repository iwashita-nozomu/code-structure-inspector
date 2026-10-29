//! Bounded syntactic evidence, never an executable-code or whole-program analyzer.
use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt};
use cap_std::fs::{Dir, OpenOptions};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    ops::ControlFlow,
    path::{Component, Path, PathBuf},
    time::{Duration, Instant},
};
use tree_sitter::{Node, ParseOptions, Parser};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const MAX_FILES: usize = 500;
pub const MAX_ENTRIES: usize = 10_000;
pub const MAX_FILE_BYTES: u64 = 1_048_576;
pub const MAX_TOTAL_BYTES: u64 = 16_777_216;
const MAX_AST_NODES: usize = 100_000;
const MAX_ITEMS: usize = 10_000;
const MAX_DEPTH: usize = 32;
const MAX_TIME: Duration = Duration::from_secs(5);

#[derive(Debug, Serialize)]
pub struct Failure {
    pub code: &'static str,
    pub message: &'static str,
}
impl Failure {
    pub fn new(code: &'static str, message: &'static str) -> Self {
        Self { code, message }
    }
}
#[derive(Debug, Serialize)]
pub struct Symbol {
    pub name: String,
    pub kind: String,
    pub start_line: usize,
    pub end_line: usize,
}
#[derive(Debug, Serialize)]
pub struct FileNode {
    pub path: String,
    pub language: String,
    pub lines: usize,
    pub parse_status: String,
    pub symbols: Vec<Symbol>,
}
#[derive(Debug, Serialize)]
pub struct Dependency {
    pub from: String,
    pub target: String,
    pub kind: String,
    pub line: usize,
    pub resolved: Option<String>,
    pub resolution: String,
}
#[derive(Debug, Serialize)]
pub struct Coverage {
    pub entries_visited: usize,
    pub files_read: usize,
    pub bytes_read: u64,
    pub excluded_entries: usize,
    pub unsupported_files: usize,
    pub skipped_files: usize,
    pub syntax_error_files: usize,
    pub unresolved_dependencies: usize,
    pub truncated: bool,
    pub reasons: BTreeSet<String>,
}
#[derive(Debug, Serialize)]
pub struct Limits {
    pub max_files: usize,
    pub max_entries: usize,
    pub max_file_bytes: u64,
    pub max_total_bytes: u64,
    pub max_depth: usize,
    pub time_budget_ms: u64,
    pub max_ast_nodes_per_file: usize,
    pub max_dependencies: usize,
}
#[derive(Debug, Serialize)]
pub struct Report {
    pub schema_version: u8,
    pub analyzer_version: &'static str,
    pub analysis: &'static str,
    pub files: Vec<FileNode>,
    pub dependencies: Vec<Dependency>,
    pub cycles: Vec<Vec<String>>,
    pub coverage: Coverage,
    pub limits: Limits,
    pub limitations: Vec<&'static str>,
    pub mermaid: String,
}
#[derive(Debug, Serialize)]
pub struct Related {
    pub schema_version: u8,
    pub path: String,
    pub outgoing: Vec<String>,
    pub incoming: Vec<String>,
    pub evidence: Vec<usize>,
    pub graph_truncated: bool,
}

fn excluded(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n.starts_with('.')
        || matches!(
            n.as_str(),
            "target"
                | "build"
                | "dist"
                | "node_modules"
                | "vendor"
                | "venv"
                | "__pycache__"
                | "credentials"
                | "credential"
                | "secrets"
                | "secret"
                | "env"
                | "environment"
                | "site-packages"
        )
        || n.starts_with("credentials.")
        || n.starts_with("secrets.")
        || n.starts_with("secret.")
        || n.ends_with(".pem")
        || n.ends_with(".key")
}
fn path_string(path: &Path) -> Option<String> {
    let s = path.to_str()?.replace('\\', "/");
    if s.len() > 512 || s.chars().any(char::is_control) {
        None
    } else {
        Some(s)
    }
}
fn language(path: &str) -> Option<&'static str> {
    if matches!(
        Path::new(path).file_name().and_then(|s| s.to_str()),
        Some("Cargo.toml" | "pyproject.toml" | "requirements.txt")
    ) {
        return Some("manifest");
    }
    match Path::new(path).extension()?.to_str()? {
        "rs" => Some("rust"),
        "py" => Some("python"),
        "c" | "cc" | "cpp" | "cxx" | "h" | "hh" | "hpp" | "hxx" => Some("cpp"),
        "cu" | "cuh" => Some("cuda"),
        _ => None,
    }
}
fn clean_name(s: &str) -> bool {
    !s.is_empty() && s.len() <= 256 && !s.chars().any(char::is_control)
}
fn dep(out: &mut Vec<Dependency>, path: &str, target: &str, kind: &str, line: usize) {
    if out.len() < MAX_ITEMS && clean_name(target) {
        out.push(Dependency {
            from: path.into(),
            target: target.into(),
            kind: kind.into(),
            line,
            resolved: None,
            resolution: "unresolved".into(),
        });
    }
}
fn node_text<'a>(node: Node<'_>, source: &'a str) -> &'a str {
    source.get(node.byte_range()).unwrap_or("")
}
fn declaration_name<'a>(mut node: Node<'_>, source: &'a str) -> Option<&'a str> {
    for _ in 0..32 {
        if let Some(name) = node.child_by_field_name("name") {
            return Some(node_text(name, source));
        }
        if let Some(next) = node.child_by_field_name("declarator") {
            node = next;
        } else if matches!(
            node.kind(),
            "identifier"
                | "field_identifier"
                | "qualified_identifier"
                | "operator_name"
                | "destructor_name"
        ) {
            return Some(node_text(node, source));
        } else {
            return None;
        }
    }
    None
}

fn unsupported_rust_scope(node: Node<'_>, source: &str) -> bool {
    let mut ancestor = node.parent();
    while let Some(parent) = ancestor {
        if parent.kind() == "mod_item" && parent.child_by_field_name("body").is_some() {
            return true;
        }
        ancestor = parent.parent();
    }
    let mut previous = node.prev_named_sibling();
    while let Some(sibling) = previous {
        if sibling.kind() == "attribute_item" {
            if node_text(sibling, source).contains("path") {
                return true;
            }
        } else if sibling.kind() != "line_comment" && sibling.kind() != "block_comment" {
            break;
        }
        previous = sibling.prev_named_sibling();
    }
    false
}

fn parse_source(
    path: &str,
    lang: &str,
    source: &str,
    end: Instant,
    deps: &mut Vec<Dependency>,
) -> (Vec<Symbol>, String, bool) {
    let mut parser = Parser::new();
    let grammar = match lang {
        "rust" => tree_sitter_rust::LANGUAGE,
        "python" => tree_sitter_python::LANGUAGE,
        _ => tree_sitter_cpp::LANGUAGE,
    };
    if parser.set_language(&grammar.into()).is_err() {
        return (vec![], "parser_unavailable".into(), true);
    }
    let deadline = (Instant::now() + Duration::from_millis(500)).min(end);
    let mut cancel = |_: &tree_sitter::ParseState| {
        if Instant::now() > deadline {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    };
    let mut input = |offset, _| source.as_bytes().get(offset..).unwrap_or(&[]);
    let Some(tree) = parser.parse_with_options(
        &mut input,
        None,
        Some(ParseOptions::new().progress_callback(&mut cancel)),
    ) else {
        return (vec![], "parse_timeout".into(), true);
    };
    let has_error = tree.root_node().has_error();
    let mut stack = vec![tree.root_node()];
    let mut symbols = vec![];
    let mut visited = 0;
    let mut truncated = false;
    while let Some(node) = stack.pop() {
        visited += 1;
        if visited > MAX_AST_NODES
            || symbols.len() >= 1000
            || deps.len() >= MAX_ITEMS
            || Instant::now() > end
        {
            truncated = true;
            break;
        }
        let kind = node.kind();
        if matches!(
            kind,
            "function_item"
                | "struct_item"
                | "enum_item"
                | "trait_item"
                | "mod_item"
                | "type_item"
                | "function_definition"
                | "class_definition"
                | "class_specifier"
                | "struct_specifier"
                | "enum_specifier"
                | "namespace_definition"
        ) && let Some(name) = declaration_name(node, source).filter(|s| clean_name(s))
        {
            symbols.push(Symbol {
                name: name.into(),
                kind: kind.into(),
                start_line: node.start_position().row + 1,
                end_line: node.end_position().row + 1,
            });
        }
        let line = node.start_position().row + 1;
        match (lang, kind) {
            ("rust", "mod_item") if node.child_by_field_name("body").is_none() => {
                if let Some(name) = node.child_by_field_name("name") {
                    dep(
                        deps,
                        path,
                        node_text(name, source),
                        if unsupported_rust_scope(node, source) {
                            "rust_unsupported_mod"
                        } else {
                            "rust_mod"
                        },
                        line,
                    );
                }
            }
            ("rust", "use_declaration") => {
                if let Some(arg) = node.child_by_field_name("argument") {
                    dep(
                        deps,
                        path,
                        node_text(arg, source),
                        if unsupported_rust_scope(node, source) {
                            "rust_unsupported_use"
                        } else {
                            "rust_use"
                        },
                        line,
                    );
                }
            }
            ("python", "import_statement") => {
                let mut cursor = node.walk();
                for child in node.named_children(&mut cursor) {
                    let name = child.child_by_field_name("name").unwrap_or(child);
                    if matches!(child.kind(), "dotted_name" | "aliased_import") {
                        dep(deps, path, node_text(name, source), "python_import", line);
                    }
                }
            }
            ("python", "import_from_statement") => {
                if let Some(module) = node.child_by_field_name("module_name") {
                    dep(deps, path, node_text(module, source), "python_from", line);
                }
            }
            ("cpp" | "cuda", "preproc_include") => {
                if let Some(p) = node.child_by_field_name("path") {
                    let raw = node_text(p, source);
                    if raw.starts_with('"') && raw.ends_with('"') {
                        dep(
                            deps,
                            path,
                            &raw[1..raw.len() - 1],
                            "cpp_quoted_include",
                            line,
                        );
                    } else {
                        dep(deps, path, raw, "cpp_system_or_macro_include", line);
                    }
                }
            }
            _ => {}
        }
        // Walk iteratively: no recursion on adversarial source nesting.
        let mut cursor = node.walk();
        let children: Vec<_> = node
            .named_children(&mut cursor)
            .take(MAX_AST_NODES - visited.min(MAX_AST_NODES))
            .collect();
        stack.extend(children.into_iter().rev());
    }
    let status = if truncated {
        "ast_limit"
    } else if has_error {
        "syntax_errors"
    } else if lang == "cuda" {
        "cpp_syntax_only"
    } else {
        "parsed"
    };
    (symbols, status.into(), truncated)
}

#[derive(serde::Deserialize, Default)]
struct Manifest {
    #[serde(default)]
    dependencies: BTreeMap<toml::Spanned<String>, serde::de::IgnoredAny>,
    #[serde(default, rename = "dev-dependencies")]
    dev: BTreeMap<toml::Spanned<String>, serde::de::IgnoredAny>,
    #[serde(default, rename = "build-dependencies")]
    build: BTreeMap<toml::Spanned<String>, serde::de::IgnoredAny>,
    #[serde(default)]
    target: BTreeMap<String, Manifest>,
    workspace: Option<Box<Manifest>>,
    project: Option<PythonProject>,
}
#[derive(serde::Deserialize)]
struct PythonProject {
    #[serde(default)]
    dependencies: Vec<toml::Spanned<String>>,
    #[serde(default, rename = "optional-dependencies")]
    optional: BTreeMap<String, Vec<toml::Spanned<String>>>,
}
fn parse_manifest(path: &str, source: &str, deps: &mut Vec<Dependency>) -> bool {
    let filename = Path::new(path).file_name().and_then(|s| s.to_str());
    let newlines: Vec<_> = source
        .bytes()
        .enumerate()
        .filter_map(|(i, b)| (b == b'\n').then_some(i))
        .collect();
    if filename == Some("requirements.txt") {
        for (i, line) in source.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let name: String = line
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
                .collect();
            let rest = &line[name.len()..];
            let named = rest.chars().next().is_none_or(|c| {
                c.is_whitespace()
                    || matches!(c, '[' | '<' | '>' | '=' | '~' | '!' | ';' | '@' | '#')
            });
            if !name.is_empty() && named {
                dep(deps, path, &name, "python_requirement", i + 1);
            }
        }
        return true;
    }
    let Ok(manifest) = toml::from_str::<Manifest>(source) else {
        return false;
    };
    if filename == Some("Cargo.toml") {
        let mut stack = vec![&manifest];
        while let Some(m) = stack.pop() {
            for (kind, table) in [
                ("cargo_dependencies", &m.dependencies),
                ("cargo_dev-dependencies", &m.dev),
                ("cargo_build-dependencies", &m.build),
            ] {
                for key in table.keys() {
                    let line = newlines.partition_point(|&i| i < key.span().start) + 1;
                    dep(deps, path, key.get_ref(), kind, line);
                }
            }
            stack.extend(m.target.values());
            if let Some(workspace) = &m.workspace {
                stack.push(workspace);
            }
        }
    } else if let Some(project) = manifest.project {
        for requirement in project
            .dependencies
            .iter()
            .chain(project.optional.values().flatten())
        {
            let name: String = requirement
                .get_ref()
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
                .collect();
            let line = newlines.partition_point(|&i| i < requirement.span().start) + 1;
            dep(deps, path, &name, "pyproject_requirement", line);
        }
    }
    true
}

pub fn inspect(root: &Path) -> Result<Report, Failure> {
    if !root.is_absolute() || root.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(Failure::new(
            "invalid_root",
            "Use an absolute repository directory without parent traversal.",
        ));
    }
    if root
        .components()
        .any(|c| matches!(c,Component::Normal(n) if n.to_str().is_none_or(excluded)))
    {
        return Err(Failure::new(
            "excluded_root",
            "Hidden, secret, environment and generated directories are excluded.",
        ));
    }
    let meta = std::fs::symlink_metadata(root)
        .map_err(|_| Failure::new("root_unavailable", "Cannot access the requested root."))?;
    if meta.is_symlink() || !meta.is_dir() {
        return Err(Failure::new(
            "invalid_root",
            "The root must be a real directory, not a symlink.",
        ));
    }
    let canonical = std::fs::canonicalize(root)
        .map_err(|_| Failure::new("root_unavailable", "Cannot resolve the requested root."))?;
    if canonical.parent().is_none()
        || canonical
            .components()
            .any(|c| matches!(c,Component::Normal(n) if n.to_str().is_none_or(excluded)))
    {
        return Err(Failure::new(
            "excluded_root",
            "Root is excluded or too broad.",
        ));
    }
    let dir = Dir::open_ambient_dir(canonical, cap_std::ambient_authority())
        .map_err(|_| Failure::new("root_unavailable", "Cannot open the requested root."))?;
    let end = Instant::now() + MAX_TIME;
    let mut r = Report {
        schema_version: 1,
        analyzer_version: VERSION,
        analysis: "syntactic_evidence",
        files: vec![],
        dependencies: vec![],
        cycles: vec![],
        coverage: Coverage {
            entries_visited: 0,
            files_read: 0,
            bytes_read: 0,
            excluded_entries: 0,
            unsupported_files: 0,
            skipped_files: 0,
            syntax_error_files: 0,
            unresolved_dependencies: 0,
            truncated: false,
            reasons: BTreeSet::new(),
        },
        limits: Limits {
            max_files: MAX_FILES,
            max_entries: MAX_ENTRIES,
            max_file_bytes: MAX_FILE_BYTES,
            max_total_bytes: MAX_TOTAL_BYTES,
            max_depth: MAX_DEPTH,
            time_budget_ms: 5000,
            max_ast_nodes_per_file: MAX_AST_NODES,
            max_dependencies: MAX_ITEMS,
        },
        limitations: vec![
            "Syntactic file dependencies and declaration ranges only; not a call graph or semantic responsibility proof.",
            "No code, imports, cargo metadata, build scripts, compile_commands, compiler or network is executed.",
            "Rust cfg/macros, path attributes, inline-module imports and grouped uses are not resolved; local resolution is a layout candidate, not compiler verification.",
            "Python dynamic imports, sys.path changes, src layouts and from-imported submodules are not resolved; module paths are layout candidates.",
            "C/C++ quoted includes resolve only relative to their containing file; include paths, macros and conditionals are not evaluated. CUDA uses the C++ grammar, not a CUDA semantic analyzer.",
            "Manifest dependencies are declarations, not lockfile/feature/target resolved package graphs. Witnesses use exact declaration key/string spans.",
            "All hidden entries, symlinks, generated/vendor/environment/secret directories and unsupported files are excluded. No snippets or docstrings are returned.",
            "Output names are untrusted source data. Responsibility explanations must be labeled as interpretations and cite symbol/file/line evidence.",
            "The scan is not an atomic filesystem snapshot. Capability-relative opens prevent symlink escapes; do not mutate the repository concurrently.",
            "Local processing has no network client. MCP results are supplied to Codex and may be processed by its configured model service.",
        ],
        mermaid: String::new(),
    };
    let mut directories = vec![(PathBuf::new(), 0)];
    'scan: while let Some((relative, depth)) = directories.pop() {
        if Instant::now() > end {
            truncate(&mut r, "time_budget");
            break;
        }
        let opened = if relative.as_os_str().is_empty() {
            dir.try_clone()
        } else {
            dir.open_dir(&relative)
        };
        let Ok(opened) = opened else {
            r.coverage.skipped_files += 1;
            r.coverage.reasons.insert("directory_unreadable".into());
            continue;
        };
        let Ok(entries) = opened.entries() else {
            r.coverage.skipped_files += 1;
            continue;
        };
        // Bound directory enumeration before sorting, including excluded/unsupported entries.
        let mut names = vec![];
        for entry in entries {
            r.coverage.entries_visited += 1;
            if r.coverage.entries_visited > MAX_ENTRIES || Instant::now() > end {
                truncate(&mut r, "entry_or_time_budget");
                break 'scan;
            }
            match entry {
                Ok(e) => names.push(e.file_name()),
                Err(_) => r.coverage.skipped_files += 1,
            }
        }
        names.sort();
        for name in names {
            let Some(name_str) = name.to_str() else {
                r.coverage.skipped_files += 1;
                continue;
            };
            if excluded(name_str) {
                r.coverage.excluded_entries += 1;
                continue;
            }
            let rel = relative.join(&name);
            let Some(path) = path_string(&rel) else {
                r.coverage.skipped_files += 1;
                continue;
            };
            let Ok(metadata) = dir.symlink_metadata(&rel) else {
                r.coverage.skipped_files += 1;
                continue;
            };
            if metadata.is_symlink() {
                r.coverage.excluded_entries += 1;
                continue;
            }
            if metadata.is_dir() {
                if depth >= MAX_DEPTH {
                    truncate(&mut r, "depth_budget");
                } else {
                    directories.push((rel, depth + 1));
                }
                continue;
            }
            if !metadata.is_file() {
                r.coverage.excluded_entries += 1;
                continue;
            }
            let Some(lang) = language(&path) else {
                r.coverage.unsupported_files += 1;
                continue;
            };
            if r.coverage.files_read >= MAX_FILES || Instant::now() > end {
                truncate(&mut r, "file_or_time_budget");
                break 'scan;
            }
            if metadata.len() > MAX_FILE_BYTES
                || r.coverage.bytes_read + metadata.len() > MAX_TOTAL_BYTES
            {
                r.coverage.skipped_files += 1;
                truncate(&mut r, "byte_budget");
                continue;
            }
            let mut options = OpenOptions::new();
            options.read(true).follow(FollowSymlinks::No);
            #[cfg(unix)]
            {
                use cap_std::fs::OpenOptionsExt;
                options.custom_flags(libc::O_NONBLOCK);
            }
            let Ok(file) = dir.open_with(&rel, &options) else {
                r.coverage.skipped_files += 1;
                continue;
            };
            if !file.metadata().is_ok_and(|m| m.is_file()) {
                r.coverage.skipped_files += 1;
                continue;
            }
            let mut bytes = Vec::new();
            let remaining = (MAX_TOTAL_BYTES - r.coverage.bytes_read).min(MAX_FILE_BYTES);
            if file.take(remaining + 1).read_to_end(&mut bytes).is_err()
                || bytes.len() as u64 > remaining
            {
                r.coverage.skipped_files += 1;
                truncate(&mut r, "read_or_byte_budget");
                continue;
            }
            r.coverage.bytes_read += bytes.len() as u64;
            r.coverage.files_read += 1;
            let Ok(source) = std::str::from_utf8(&bytes) else {
                r.coverage.skipped_files += 1;
                continue;
            };
            let (symbols, status, truncated) = if lang == "manifest" {
                (
                    vec![],
                    if parse_manifest(&path, source, &mut r.dependencies) {
                        "declared_only".into()
                    } else {
                        "syntax_errors".into()
                    },
                    false,
                )
            } else {
                parse_source(&path, lang, source, end, &mut r.dependencies)
            };
            if truncated {
                truncate(&mut r, "parser_budget");
            }
            if status == "syntax_errors" {
                r.coverage.syntax_error_files += 1;
            }
            r.files.push(FileNode {
                path,
                language: lang.into(),
                lines: source.lines().count(),
                parse_status: status,
                symbols,
            });
        }
    }
    if r.dependencies.len() >= MAX_ITEMS {
        truncate(&mut r, "dependency_budget");
    }
    r.files.sort_by(|a, b| a.path.cmp(&b.path));
    r.dependencies.sort_by(|a, b| {
        (&a.from, a.line, &a.kind, &a.target).cmp(&(&b.from, b.line, &b.kind, &b.target))
    });
    r.dependencies.dedup_by(|a, b| {
        a.from == b.from && a.line == b.line && a.kind == b.kind && a.target == b.target
    });
    resolve(&mut r);
    r.coverage.unresolved_dependencies = r
        .dependencies
        .iter()
        .filter(|d| d.resolved.is_none())
        .count();
    r.cycles = cycles(&r);
    r.mermaid = mermaid(&r);
    Ok(r)
}
fn truncate(r: &mut Report, reason: &str) {
    r.coverage.truncated = true;
    r.coverage.reasons.insert(reason.into());
}
fn normalize(path: &Path) -> Option<String> {
    let mut parts = vec![];
    for part in path.components() {
        match part {
            Component::Normal(s) => parts.push(s.to_str()?),
            Component::CurDir => {}
            Component::ParentDir => {
                parts.pop()?;
            }
            _ => return None,
        }
    }
    Some(parts.join("/"))
}
fn rust_base(from: &str) -> PathBuf {
    let path = Path::new(from);
    let parent = path.parent().unwrap_or(Path::new(""));
    if matches!(
        path.file_name().and_then(|s| s.to_str()),
        Some("main.rs" | "lib.rs" | "mod.rs")
    ) {
        parent.into()
    } else {
        parent.join(path.file_stem().unwrap_or_default())
    }
}
fn resolve(r: &mut Report) {
    let files: BTreeSet<_> = r.files.iter().map(|f| f.path.clone()).collect();
    for d in &mut r.dependencies {
        let parent = Path::new(&d.from).parent().unwrap_or(Path::new(""));
        let candidates: Vec<String> = match d.kind.as_str() {
            "cpp_quoted_include" => normalize(&parent.join(&d.target)).into_iter().collect(),
            "python_import" | "python_from" => {
                let dots = d.target.chars().take_while(|c| *c == '.').count();
                let base = if dots > 0 {
                    let mut p = parent.to_path_buf();
                    let mut valid = true;
                    for _ in 1..dots {
                        if !p.pop() {
                            valid = false;
                            break;
                        }
                    }
                    if !valid {
                        continue;
                    }
                    p
                } else {
                    PathBuf::new()
                };
                let p = base.join(d.target[dots..].replace('.', "/"));
                vec![
                    format!("{}.py", p.display()).replace('\\', "/"),
                    p.join("__init__.py").to_string_lossy().replace('\\', "/"),
                ]
            }
            "rust_mod" => {
                let p = rust_base(&d.from).join(&d.target);
                vec![
                    format!("{}.rs", p.display()).replace('\\', "/"),
                    p.join("mod.rs").to_string_lossy().replace('\\', "/"),
                ]
            }
            "rust_use" => {
                let target = d.target.split(" as ").next().unwrap_or("");
                if target.contains(['{', '}', '*']) {
                    continue;
                }
                let parts: Vec<_> = target.split("::").collect();
                let mut base = rust_base(&d.from);
                let mut start = 1;
                match parts.first().copied() {
                    Some("crate") => {
                        // Nearest scanned crate root (src/lib.rs or src/main.rs), not workspace src.
                        let mut ancestor = Some(parent);
                        let mut found = None;
                        while let Some(a) = ancestor {
                            if [a.join("lib.rs"), a.join("main.rs")]
                                .iter()
                                .any(|p| normalize(p).is_some_and(|s| files.contains(&s)))
                            {
                                found = Some(a.to_path_buf());
                                break;
                            }
                            ancestor = a.parent();
                        }
                        let Some(root) = found else {
                            continue;
                        };
                        base = root;
                    }
                    Some("self") => {}
                    Some("super") => {
                        if !base.pop() {
                            continue;
                        }
                        while parts.get(start) == Some(&"super") {
                            if !base.pop() {
                                break;
                            }
                            start += 1;
                        }
                    }
                    _ => continue,
                }
                let mut result = vec![];
                // A symbol suffix can be removed, so these remain layout candidates only.
                for part in parts.iter().skip(start) {
                    base.push(part);
                    result.push(format!("{}.rs", base.display()).replace('\\', "/"));
                    result.push(base.join("mod.rs").to_string_lossy().replace('\\', "/"));
                }
                result.reverse();
                result
            }
            _ => vec![],
        };
        let found: Vec<_> = candidates
            .into_iter()
            .filter(|c| files.contains(c))
            .collect();
        if d.kind == "rust_use" {
            if let Some(first) = found.first() {
                d.resolved = Some(first.clone());
                d.resolution = "layout_candidate".into();
            }
        } else if found.len() == 1 {
            d.resolved = Some(found[0].clone());
            d.resolution = if d.kind == "cpp_quoted_include" {
                "literal_relative_path"
            } else {
                "layout_candidate"
            }
            .into();
        } else if found.len() > 1 {
            d.resolution = "ambiguous".into();
        }
    }
}
fn graph(r: &Report) -> Vec<BTreeSet<usize>> {
    let index: BTreeMap<_, _> = r
        .files
        .iter()
        .enumerate()
        .map(|(i, f)| (f.path.as_str(), i))
        .collect();
    let mut graph = vec![BTreeSet::new(); r.files.len()];
    for d in &r.dependencies {
        if let (Some(&a), Some(&b)) = (
            index.get(d.from.as_str()),
            d.resolved.as_ref().and_then(|p| index.get(p.as_str())),
        ) {
            graph[a].insert(b);
        }
    }
    graph
}
fn cycles(r: &Report) -> Vec<Vec<String>> {
    // Bounded transitive closure: <= 500 vertices. SCCs, not exhaustive simple cycles.
    let g = graph(r);
    let n = g.len();
    let mut reachable = vec![vec![false; n]; n];
    for start in 0..n {
        let mut stack: Vec<_> = g[start].iter().copied().collect();
        while let Some(v) = stack.pop() {
            if !reachable[start][v] {
                reachable[start][v] = true;
                stack.extend(g[v].iter().copied());
            }
        }
    }
    let mut used = BTreeSet::new();
    let mut result = vec![];
    for (a, row) in reachable.iter().enumerate() {
        if used.contains(&a) {
            continue;
        }
        let group: Vec<_> = (a..n)
            .filter(|&b| (a == b && row[a]) || (a != b && row[b] && reachable[b][a]))
            .collect();
        if !group.is_empty() {
            used.extend(group.iter().copied());
            result.push(group.into_iter().map(|i| r.files[i].path.clone()).collect());
        }
    }
    result
}
fn mermaid(r: &Report) -> String {
    let mut s = String::from("flowchart LR\n");
    for (i, file) in r.files.iter().enumerate() {
        let label: String = file
            .path
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || matches!(c, '/' | '.' | '_' | '-' | ' ') {
                    c
                } else {
                    '?'
                }
            })
            .collect();
        s.push_str(&format!("  n{i}[\"{label}\"]\n"));
    }
    for (a, edges) in graph(r).iter().enumerate() {
        for b in edges {
            s.push_str(&format!("  n{a} --> n{b}\n"));
        }
    }
    s
}
pub fn related(r: &Report, path: &str) -> Result<Related, Failure> {
    if !r.files.iter().any(|f| f.path == path) {
        return Err(Failure::new(
            "file_not_scanned",
            "Requested relative file is absent from the bounded scan.",
        ));
    }
    let mut outgoing = BTreeSet::new();
    let mut incoming = BTreeSet::new();
    let mut evidence = vec![];
    for (i, d) in r.dependencies.iter().enumerate() {
        if d.from == path {
            if let Some(p) = &d.resolved {
                outgoing.insert(p.clone());
            }
            evidence.push(i);
        } else if d.resolved.as_deref() == Some(path) {
            incoming.insert(d.from.clone());
            evidence.push(i);
        }
    }
    Ok(Related {
        schema_version: 1,
        path: path.into(),
        outgoing: outgoing.into_iter().collect(),
        incoming: incoming.into_iter().collect(),
        evidence,
        graph_truncated: r.coverage.truncated,
    })
}
