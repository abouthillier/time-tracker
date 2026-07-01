use std::path::PathBuf;

use super::types::EditorKind;

pub fn storage_json_path(editor: EditorKind) -> Option<PathBuf> {
    let base = editor_data_dir(editor)?;
    Some(base.join("User").join("globalStorage").join("storage.json"))
}

pub fn workspace_storage_dir(editor: EditorKind) -> Option<PathBuf> {
    let base = editor_data_dir(editor)?;
    Some(base.join("User").join("workspaceStorage"))
}

fn editor_data_dir(editor: EditorKind) -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var_os("APPDATA")?;
        let folder = match editor {
            EditorKind::Cursor => "Cursor",
            EditorKind::Code => "Code",
            EditorKind::Vscodium => "VSCodium",
        };
        return Some(PathBuf::from(appdata).join(folder));
    }

    #[cfg(target_os = "macos")]
    {
        let home = std::env::var_os("HOME")?;
        let folder = match editor {
            EditorKind::Cursor => "Cursor",
            EditorKind::Code => "Code",
            EditorKind::Vscodium => "VSCodium",
        };
        return Some(
            PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join(folder),
        );
    }

    #[cfg(target_os = "linux")]
    {
        let config = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config"))
            })?;
        let folder = match editor {
            EditorKind::Cursor => "Cursor",
            EditorKind::Code => "Code",
            EditorKind::Vscodium => "VSCodium",
        };
        return Some(config.join(folder));
    }

    #[allow(unreachable_code)]
    None
}
