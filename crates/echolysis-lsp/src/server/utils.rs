use std::path::{Path, PathBuf};

use echolysis_core::engine::indexed_node::IndexedNode;
use tower_lsp::lsp_types;

// Convert tree-sitter point to LSP position
pub fn point_to_position(point: &echolysis_core::Point) -> lsp_types::Position {
    lsp_types::Position::new(point.row as u32, point.column as u32)
}

pub fn get_node_location(node: &IndexedNode) -> Option<lsp_types::Location> {
    let uri = lsp_types::Url::from_file_path(node.path()).ok()?;
    let (start, end) = node.position_range();
    Some(lsp_types::Location {
        uri,
        range: lsp_types::Range {
            start: point_to_position(&start),
            end: point_to_position(&end),
        },
    })
}

const MAX_FILE_COUNT: usize = 10000; // TODO: configurable file count

pub fn get_all_files_under_folder(folder: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut dirs_to_scan = vec![folder.to_path_buf()];

    while let Some(current_dir) = dirs_to_scan.pop() {
        if let Ok(entries) = std::fs::read_dir(&current_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.components().any(|c| {
                    matches!(
                        c.as_os_str().to_str(),
                        Some("venv" | "node_modules" | "target")
                    )
                }) {
                    continue;
                }
                if path.is_dir() {
                    dirs_to_scan.push(path);
                    continue;
                }
                if !path.is_file() {
                    continue;
                }
                files.push(path);
                if files.len() == MAX_FILE_COUNT {
                    return files;
                }
            }
        }
    }

    files
}

pub fn get_git_top_root(path: &Path) -> Option<PathBuf> {
    if let Ok(repo) = git2::Repository::discover(path) {
        return Some(repo.workdir()?.to_path_buf());
    }
    None
}
