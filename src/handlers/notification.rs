use std::sync::RwLock;

use tower_lsp::{lsp_types::TextDocumentItem, Client};
use tracing::debug;

use crate::{
    analyzer::{
        diagnostics::DiagnosticCollector, diagnostics::DiagnosticCollectorFactory, parser::Parser,
    },
    lsp::state::State,
};

pub async fn handle_did_open(
    document: &TextDocumentItem,
    state: &State,
    parser: &RwLock<Parser>,
    client: &Client,
) {
    let uri = document.uri.clone();
    let diagnostic_collector = DiagnosticCollectorFactory::create();

    let diagnostics = {
        let tree = parser
            .write()
            .unwrap()
            .parse(&document.text)
            .expect("to parse file");

        let diags = match diagnostic_collector.collect(document) {
            Ok(d) => Some(d),
            Err(err) => {
                debug!("Failed to collect diagnostics: {}", err.message);
                None
            }
        };

        state.ast_map.insert(uri.clone(), tree);
        state
            .document_map
            .insert(uri.clone(), document.text.clone());

        diags
    };

    if let Some(diags) = diagnostics {
        client.publish_diagnostics(uri, diags, None).await;
    }
}
