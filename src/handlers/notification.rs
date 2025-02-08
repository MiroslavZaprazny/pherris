use std::sync::RwLock;

use tower_lsp::lsp_types::TextDocumentItem;

use crate::{
    analyzer::parser::Parser,
    lsp::state::State,
};

pub fn handle_did_open(document: TextDocumentItem, state: &State, parser: &RwLock<Parser>) {
    if state.ast_map.contains_key(&document.uri) {
        return;
    }

    let tree = parser
        .write()
        .unwrap()
        .parse(&document.text)
        .expect("to parse file");

    state.ast_map.insert(document.uri.clone(), tree);
    state.document_map.insert(document.uri, document.text);
}
