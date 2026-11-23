use std::path::{Path, PathBuf};
use std::sync::RwLock;

static READ_ONLY_ROOTS: RwLock<Vec<PathBuf>> = RwLock::new(Vec::new());

pub fn set_read_only_roots(roots: Vec<PathBuf>) {
    let normalized: Vec<PathBuf> = roots
        .into_iter()
        .filter_map(|path| normalize_root(&path))
        .collect();
    if let Ok(mut guard) = READ_ONLY_ROOTS.write() {
        *guard = normalized;
    }
}

pub fn load_read_only_roots_from_json(json: &str) -> Result<(), String> {
    #[derive(serde::Deserialize)]
    struct File {
        #[serde(default, rename = "readOnlyRoots")]
        read_only_roots: Vec<String>,
    }

    let parsed: File =
        serde_json::from_str(json).map_err(|e| format!("Invalid reference-libraries JSON: {e}"))?;

    set_read_only_roots(
        parsed
            .read_only_roots
            .iter()
            .map(PathBuf::from)
            .collect(),
    );
    Ok(())
}

/// Dev-only file at repo `config/reference-libraries.local.json` (see example).
pub fn load_dev_reference_libraries() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config = manifest_dir.join("../config/reference-libraries.local.json");
    if !config.is_file() {
        return;
    }
    if let Ok(json) = std::fs::read_to_string(&config) {
        let _ = load_read_only_roots_from_json(&json);
    }
}

pub fn read_only_roots() -> Vec<PathBuf> {
    READ_ONLY_ROOTS
        .read()
        .map(|guard| guard.clone())
        .unwrap_or_default()
}

pub fn is_under_read_only_root(path: &Path) -> bool {
    let normalized = normalize_path(path);
    read_only_roots()
        .iter()
        .any(|root| path_starts_with_root(&normalized, root))
}

/// Blocks delete, rename, or other mutations on reference libraries.
pub fn assert_mutation_allowed(path: &Path) -> Result<(), String> {
    if is_under_read_only_root(path) {
        return Err(format!(
            "Path is read-only (reference library): {}. Use a copied test folder for write/delete tests.",
            path.display()
        ));
    }
    Ok(())
}

fn normalize_root(path: &Path) -> Option<PathBuf> {
    if !path.exists() {
        return Some(normalize_path(path));
    }
    std::fs::canonicalize(path)
        .map(|p| normalize_path(p.as_path()))
        .ok()
        .or_else(|| Some(normalize_path(path)))
}

fn normalize_path(path: &Path) -> PathBuf {
    let lossy = path.to_string_lossy().replace('/', "\\");
    PathBuf::from(lossy.trim_end_matches('\\').to_ascii_lowercase())
}

fn path_starts_with_root(path: &PathBuf, root: &PathBuf) -> bool {
    let path = path.to_string_lossy();
    let root = root.to_string_lossy();
    path == root || path.starts_with(&(root.to_string() + "\\"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_nested_read_only_path() {
        set_read_only_roots(vec![PathBuf::from(r"e:\movies")]);
        assert!(is_under_read_only_root(Path::new(r"E:\Movies\Inception\movie.nfo")));
        assert!(!is_under_read_only_root(Path::new(r"G:\Movies\a.nfo")));
    }

    #[test]
    fn mutation_blocked_under_root() {
        set_read_only_roots(vec![PathBuf::from(r"d:\library")]);
        let err = assert_mutation_allowed(Path::new(r"D:\library\dup\folder")).unwrap_err();
        assert!(err.contains("read-only"));
    }
}