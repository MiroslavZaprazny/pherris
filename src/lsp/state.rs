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

impl Default for State {
    fn default() -> Self {
        State::new(
            DashMap::default(),
            DashMap::default(),
            RwLock::new(String::from("")),
            DashMap::default(),
        )
    }
}

impl State {
    pub fn new(
        ast_map: DashMap<Url, Tree>,
        document_map: DashMap<Url, String>,
        root_path: RwLock<String>,
        class_map: DashMap<String, String>,
    ) -> Self {
        Self {
            ast_map,
            document_map,
            root_path,
            class_map,
        }
    }
}
