use std::sync::RwLock;

use dashmap::DashMap;
use tower_lsp::lsp_types::Url;
use tree_sitter::Tree;

pub struct State {
    pub ast_map: DashMap<Url, Tree>,
    pub document_map: DashMap<Url, String>,
    pub root_path: RwLock<String>,
    pub class_map: DashMap<String, String>,
}

impl State {
    pub fn new() -> Self {
        Self {
            ast_map: DashMap::default(),
            document_map: DashMap::default(),
            root_path: RwLock::new(String::new()),
            class_map: DashMap::default(),
        }
    }
}
