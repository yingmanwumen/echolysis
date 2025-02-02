use rayon::prelude::*;
use std::{collections::HashMap, path::PathBuf};
use tower_lsp::lsp_types;

use super::{FilePosition, LocationRange, Server};

impl Server {
    // Convert tree-sitter point to LSP position
    pub(super) fn to_lsp_position(
        point: &echolysis_core::tree_sitter::Point,
    ) -> lsp_types::Position {
        lsp_types::Position::new(point.row as u32, point.column as u32)
    }

    // Create LSP range from file positions
    pub(super) fn create_lsp_range(pos: &FilePosition) -> lsp_types::Range {
        lsp_types::Range {
            start: Self::to_lsp_position(&pos.start),
            end: Self::to_lsp_position(&pos.end),
        }
    }

    // Get all duplicate code fragments from engines
    async fn collect_duplicates(&self) -> Vec<(String, Vec<Vec<(String, FilePosition)>>)> {
        let duplicates = self
            .router
            .engines()
            .par_iter()
            .map(|engine| {
                let duplicates = engine.detect_duplicates();
                (
                    engine.key().clone(),
                    duplicates
                        .into_iter()
                        .map(|group| {
                            group
                                .into_iter()
                                .filter_map(|id| {
                                    let node = engine.get_node_by_id(id)?;
                                    let path = engine.get_path_by_id(id)?;
                                    Some((
                                        path.as_ref().to_string(),
                                        FilePosition {
                                            start: node.start_position(),
                                            end: node.end_position(),
                                        },
                                    ))
                                })
                                .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .collect();

        self.log_info(format!("publishing: {:?}", duplicates)).await;
        duplicates
    }

    // Store duplicate locations for goto definition feature
    fn store_duplicate_locations(
        &self,
        uri: &lsp_types::Url,
        current_pos: &FilePosition,
        group: &[(String, FilePosition)],
    ) {
        for (file_path, dup_pos) in group {
            if let Ok(target_uri) = lsp_types::Url::from_file_path(file_path.as_str()) {
                let target_range = Self::create_lsp_range(dup_pos);

                // Store the location for the current range
                self.duplicate_locations
                    .entry(LocationRange {
                        uri: uri.clone(),
                        range: Self::create_lsp_range(current_pos),
                    })
                    .or_insert_with(Vec::new)
                    .push(lsp_types::Location {
                        uri: target_uri,
                        range: target_range,
                    });
            }
        }
    }

    // Create diagnostic for a duplicate code fragment
    fn create_duplicate_diagnostic(
        current_pos: &FilePosition,
        other_locations: Vec<lsp_types::Location>,
    ) -> lsp_types::Diagnostic {
        lsp_types::Diagnostic {
            range: Self::create_lsp_range(current_pos),
            severity: Some(lsp_types::DiagnosticSeverity::WARNING),
            source: Some("echolysis".to_string()),
            message: format!(
                "Duplicated code fragment found, {} lines",
                current_pos.end.row - current_pos.start.row + 1
            ),
            related_information: Some(
                other_locations
                    .iter()
                    .map(|location| lsp_types::DiagnosticRelatedInformation {
                        location: location.clone(),
                        message: format!(
                            "Similar code fragment ({} lines)",
                            location.range.end.line - location.range.start.line + 1
                        ),
                    })
                    .collect(),
            ),
            ..Default::default()
        }
    }

    // Process a group of duplicates and update diagnostics map
    fn process_duplicate_group(
        &self,
        group: &[(String, FilePosition)],
        diagnostics_map: &mut HashMap<lsp_types::Url, Vec<lsp_types::Diagnostic>>,
    ) {
        for (file, pos) in group {
            if let Ok(uri) = lsp_types::Url::from_file_path(file.as_str()) {
                // Collect locations for definition jumping
                let other_locations: Vec<lsp_types::Location> = group
                    .iter()
                    .filter_map(|(other_file, other_pos)| {
                        if other_file == file
                            && other_pos.end == pos.end
                            && other_pos.start == pos.start
                        {
                            return None;
                        }
                        lsp_types::Url::from_file_path(other_file.as_str())
                            .ok()
                            .map(|other_uri| lsp_types::Location {
                                uri: other_uri,
                                range: Self::create_lsp_range(other_pos),
                            })
                    })
                    .collect();

                // Store locations for goto definition
                self.store_duplicate_locations(&uri, pos, group);

                // Create and store diagnostic
                let diagnostic = Self::create_duplicate_diagnostic(pos, other_locations);
                diagnostics_map.entry(uri).or_default().push(diagnostic);
            }
        }
    }

    // Publish diagnostics to the client
    async fn publish_diagnostics(
        &self,
        diagnostics_map: HashMap<lsp_types::Url, Vec<lsp_types::Diagnostic>>,
    ) {
        if diagnostics_map.is_empty() {
            // Clear existing diagnostics
            for item in self.diagnostics_record.iter() {
                self.client
                    .publish_diagnostics(item.key().clone(), vec![], None)
                    .await;
            }
            self.diagnostics_record.clear();
            return;
        }

        // Publish new diagnostics
        for (uri, diagnostics) in diagnostics_map {
            self.diagnostics_record.insert(uri.clone());
            self.client
                .publish_diagnostics(uri, diagnostics, None)
                .await;
        }
    }

    // Main function to process and publish diagnostics
    pub async fn push_diagnostic(&self) {
        self.duplicate_locations.clear();
        let duplicates = self.collect_duplicates().await;

        let mut diagnostics_map = HashMap::new();

        // Process all duplicate groups
        for (_, groups) in duplicates {
            for group in groups {
                self.process_duplicate_group(&group, &mut diagnostics_map);
            }
        }

        self.log_info(format!("diagnostics_map: {:#?}", diagnostics_map))
            .await;

        // Publish the diagnostics
        self.publish_diagnostics(diagnostics_map).await;
    }

    pub async fn clear_diagnostic(&self, paths: &[PathBuf]) {
        for path in paths {
            if let Ok(uri) = lsp_types::Url::from_file_path(path) {
                self.client
                    .publish_diagnostics(uri.clone(), vec![], None)
                    .await;
                self.diagnostics_record.remove(&uri);
            }
        }
    }
}
