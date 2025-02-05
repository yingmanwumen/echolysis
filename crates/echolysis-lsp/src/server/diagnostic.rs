use std::sync::Arc;

use ahash::AHashMap;
use echolysis_core::engine::indexed_node::IndexedNode;
use tower_lsp::lsp_types;

use super::{utils::get_node_location, Server};

impl Server {
    // Get all duplicate code fragments from engines
    async fn collect_duplicates(&self) -> Vec<Vec<Arc<IndexedNode>>> {
        self.router
            .engines()
            .iter()
            // TODO: make it configurable
            .flat_map(|engine| engine.detect_duplicates(Some(100)))
            .collect()
    }

    // Create diagnostic for a duplicate code fragment
    fn create_duplicate_diagnostic(
        location: &lsp_types::Location,
        other_locations: &[lsp_types::Location],
    ) -> lsp_types::Diagnostic {
        let message = "Duplicated code fragments found\n".to_string();
        lsp_types::Diagnostic {
            range: location.range,
            severity: Some(lsp_types::DiagnosticSeverity::INFORMATION),
            source: Some("echolysis".to_string()),
            message,
            related_information: Some(
                other_locations
                    .iter()
                    .map(|location| lsp_types::DiagnosticRelatedInformation {
                        location: location.clone(),
                        message: format!(
                            "Similar code fragment (line {}) {} lines long",
                            location.range.start.line + 1,
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
        group: &[Arc<IndexedNode>],
        diagnostics_map: &mut AHashMap<lsp_types::Url, Vec<lsp_types::Diagnostic>>,
    ) {
        let locations: Vec<_> = group
            .iter()
            .filter_map(|node| get_node_location(node))
            .collect();
        for node in group {
            if let Some(location) = get_node_location(node) {
                let diagnostic = Self::create_duplicate_diagnostic(&location, &locations);
                diagnostics_map
                    .entry(location.uri.clone())
                    .or_default()
                    .push(diagnostic);
                self.duplicate_locations.lock().push(locations.clone());
            }
        }
    }

    async fn publish_diagnostics(
        &self,
        diagnostics_map: AHashMap<lsp_types::Url, Vec<lsp_types::Diagnostic>>,
    ) {
        if diagnostics_map.is_empty() {
            // Clear existing diagnostics
            for item in self.diagnostics_uri_record.iter() {
                self.client
                    .publish_diagnostics(item.key().clone(), vec![], None)
                    .await;
            }
            self.diagnostics_uri_record.clear();
            return;
        }

        // Publish new diagnostics
        for (uri, diagnostics) in diagnostics_map {
            self.diagnostics_uri_record.insert(uri.clone());
            self.client
                .publish_diagnostics(uri, diagnostics, None)
                .await;
        }
    }

    // Main function to process and publish diagnostics
    pub async fn push_diagnostic(&self) {
        self.duplicate_locations.lock().clear();
        let duplicates = self.collect_duplicates().await;

        let mut diagnostics_map = AHashMap::new();
        for group in duplicates {
            self.process_duplicate_group(&group, &mut diagnostics_map);
        }
        self.publish_diagnostics(diagnostics_map).await;
    }

    pub async fn clear_diagnostic(&self, uris: &[lsp_types::Url], version: Option<i32>) {
        for uri in uris {
            self.diagnostics_uri_record.remove(uri);
            self.client
                .publish_diagnostics(uri.clone(), vec![], version)
                .await;
        }
    }
}
