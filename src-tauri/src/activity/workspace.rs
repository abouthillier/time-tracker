use std::{
    collections::hash_map::DefaultHasher,
    fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};

use serde::Deserialize;
use urlencoding::decode;

use super::types::{EditorKind, ResolvedWorkspace};

#[derive(Debug, Deserialize)]
struct MultiRootWorkspaceFile {
    folders: Option<Vec<MultiRootFolder>>,
}

#[derive(Debug, Deserialize)]
struct MultiRootFolder {
    path: String,
}

pub fn resolve_workspace_uri(uri: &str, editor: EditorKind) -> Result<Option<ResolvedWorkspace>, String> {
    if uri.starts_with("vscode-remote://") || uri.starts_with("vscode-remote:") {
        return Ok(Some(resolve_remote_workspace(uri, editor)));
    }

    let decoded = decode_uri_path(uri)?;
    let path = PathBuf::from(&decoded);
    let normalized = normalize_path_key(&decoded);

    if is_cursor_multi_root_stub(&normalized) {
        return resolve_multi_root_stub(&path, editor);
    }

    if decoded.ends_with(".code-workspace") {
        let label = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("workspace")
            .to_string();

        let key = format!(
            "{}:file:{}",
            editor_key(editor),
            normalize_path_key(&decoded)
        );

        return Ok(Some(ResolvedWorkspace { key, label }));
    }

    let label = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("workspace")
        .to_string();

    let key = format!(
        "{}:file:{}",
        editor_key(editor),
        normalize_path_key(&decoded)
    );

    Ok(Some(ResolvedWorkspace { key, label }))
}

fn resolve_remote_workspace(uri: &str, editor: EditorKind) -> ResolvedWorkspace {
    let label = uri
        .rsplit('/')
        .next()
        .unwrap_or("remote")
        .trim_end_matches(".code-workspace")
        .to_string();

    let key = format!("{}:remote:{}", editor_key(editor), uri);

    ResolvedWorkspace { key, label }
}

fn is_cursor_multi_root_stub(normalized_path: &str) -> bool {
    normalized_path.ends_with("workspace.json") && normalized_path.contains("/workspaces/")
}

fn resolve_multi_root_stub(
    stub_path: &Path,
    editor: EditorKind,
) -> Result<Option<ResolvedWorkspace>, String> {
    let contents = fs::read_to_string(stub_path).map_err(|error| error.to_string())?;
    let parsed: MultiRootWorkspaceFile =
        serde_json::from_str(&contents).map_err(|error| error.to_string())?;

    let folders = parsed.folders.unwrap_or_default();
    if folders.is_empty() {
        return Ok(None);
    }

    let mut normalized_paths: Vec<String> = folders
        .iter()
        .map(|folder| normalize_path_key(&folder.path))
        .collect();
    normalized_paths.sort();

    let label = folders
        .first()
        .and_then(|folder| {
            Path::new(&folder.path)
                .file_name()
                .and_then(|name| name.to_str())
        })
        .unwrap_or("workspace")
        .to_string();

    let mut hasher = DefaultHasher::new();
    normalized_paths.hash(&mut hasher);
    let hash = hasher.finish();

    let key = format!(
        "{}:multi:{}:{:x}",
        editor_key(editor),
        normalize_path_key(stub_path.to_string_lossy().as_ref()),
        hash
    );

    Ok(Some(ResolvedWorkspace { key, label }))
}

fn decode_uri_path(uri: &str) -> Result<String, String> {
    let without_scheme = uri
        .strip_prefix("file:///")
        .or_else(|| uri.strip_prefix("file://"))
        .unwrap_or(uri);

    let decoded = decode(without_scheme)
        .map_err(|error| error.to_string())?
        .into_owned();

    #[cfg(windows)]
    {
        if let Some(stripped) = decoded.strip_prefix('/') {
            return Ok(stripped.replace('/', "\\"));
        }
    }

    Ok(decoded.replace('/', std::path::MAIN_SEPARATOR_STR))
}

fn normalize_path_key(path: &str) -> String {
    PathBuf::from(path)
        .to_string_lossy()
        .replace('\\', "/")
        .to_lowercase()
}

fn editor_key(editor: EditorKind) -> &'static str {
    match editor {
        EditorKind::Cursor => "cursor",
        EditorKind::Code => "code",
        EditorKind::Vscodium => "vscodium",
    }
}
