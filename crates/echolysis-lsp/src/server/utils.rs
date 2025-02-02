use std::path::{Path, PathBuf};

use tower_lsp::lsp_types;

use super::TSRange;

// Convert tree-sitter point to LSP position
pub fn to_lsp_position(point: &echolysis_core::tree_sitter::Point) -> lsp_types::Position {
    lsp_types::Position::new(point.row as u32, point.column as u32)
}

// Create LSP range from file positions
pub fn to_lsp_range(pos: &TSRange) -> lsp_types::Range {
    lsp_types::Range {
        start: to_lsp_position(&pos.start),
        end: to_lsp_position(&pos.end),
    }
}

pub fn get_all_files_under_folder(folder: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut dirs_to_scan = vec![folder.to_path_buf()];

    while let Some(current_dir) = dirs_to_scan.pop() {
        if let Ok(entries) = std::fs::read_dir(&current_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    dirs_to_scan.push(path);
                } else if path.is_file() {
                    files.push(path);
                }
            }
        }
    }

    files
}

pub fn get_git_root(path: &Path) -> Option<PathBuf> {
    if let Ok(repo) = git2::Repository::discover(path) {
        return Some(repo.workdir()?.to_path_buf());
    }
    None
}
