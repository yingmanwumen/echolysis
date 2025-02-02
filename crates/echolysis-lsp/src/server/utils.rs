use std::path::PathBuf;

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

pub fn get_all_files_under_folder(folder: &PathBuf) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(folder).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            files.extend(get_all_files_under_folder(&path));
        } else if path.is_file() {
            files.push(path);
        }
    }
    files
}
