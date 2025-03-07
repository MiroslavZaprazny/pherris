use std::sync::RwLock;

use tower_lsp::{
    lsp_types::{TextDocumentItem, Url},
    Client,
};
use tracing::debug;

use crate::{
    analyzer::{
        diagnostics::{collect_diagnostics, Document},
        parser::Parser,
    },
    lsp::{config::InitializeOptions, state::State},
};

pub async fn handle_did_open(
    document: &TextDocumentItem,
    state: &State,
    parser: &RwLock<Parser>,
    client: &Client,
    options: &RwLock<InitializeOptions>,
) {
    let uri = document.uri.clone();

    let diagnostics = {
        let tree = parser
            .write()
            .unwrap()
            .parse(&document.text)
            .expect("to parse file");

        let diags = match collect_diagnostics(&Document::new(document.uri.clone()), options) {
            Ok(d) => Some(d),
            Err(err) => {
                debug!("Failed to collect diagnostics: {}", err.message);
                None
            }
        };
        debug!("diags {:?}", diags);

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

pub async fn handle_did_change(
    uri: &Url,
    text: &str,
    state: &State,
    parser: &RwLock<Parser>,
    client: &Client,
    options: &RwLock<InitializeOptions>,
) {
    let uri = uri.clone();

    let diagnostics = {
        let tree = parser.write().unwrap().parse(text).expect("to parse file");

        let diags = match collect_diagnostics(&Document::new(uri.clone()), options) {
            Ok(d) => Some(d),
            Err(err) => {
                debug!("Failed to collect diagnostics: {}", err.message);
                None
            }
        };
        debug!("diags {:?}", diags);

        state.ast_map.insert(uri.clone(), tree);
        state.document_map.insert(uri.clone(), text.to_string());

        diags
    };

    if let Some(diags) = diagnostics {
        client.publish_diagnostics(uri, diags, None).await;
    }
}
