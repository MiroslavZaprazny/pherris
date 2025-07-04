use std::sync::RwLock;

use dashmap::DashMap;
use mago_ast::Program;
use tower_lsp::lsp_types::Url;
use tree_sitter::Tree;

pub struct State {
    pub document_program: DashMap<Url, Program>,
    pub document_map: DashMap<Url, String>,
    pub root_path: RwLock<String>,
    pub class_map: DashMap<String, String>,
    pub ast_map: DashMap<Url, Tree>,
}

impl Default for State {
    fn default() -> Self {
        State::new(
            DashMap::default(),
            DashMap::default(),
            RwLock::new(String::from("")),
            DashMap::default(),
            DashMap::default(),
        )
    }
}

impl State {
    pub fn new(
        document_program: DashMap<Url, Program>,
        document_map: DashMap<Url, String>,
        root_path: RwLock<String>,
        class_map: DashMap<String, String>,
        ast_map: DashMap<Url, Tree>,
    ) -> Self {
        Self {
            document_program,
            document_map,
            root_path,
            class_map,
            ast_map,
        }
    }
}
