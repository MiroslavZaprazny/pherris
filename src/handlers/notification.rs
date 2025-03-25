use std::sync::RwLock;

use tower_lsp::lsp_types::TextDocumentItem;

use crate::{
    analyzer::parser::Parser,
    lsp::state::State,
};

pub async fn handle_did_open(
    document: &TextDocumentItem,
    state: &State,
    parser: &RwLock<Parser>,
) {
    let tree = parser.write().unwrap().parse(document.text.clone()).expect("to parse file");
    let uri = document.uri.clone();
    state.ast_map.insert(uri.clone(), tree);
    state
        .document_map
        .insert(uri.clone(), document.text.clone());
}
