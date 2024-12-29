use std::collections::HashMap;
use std::sync::Mutex;

use crate::tree::utils::{
    find_nearest_location, get_node_for_point, get_point_from_position, get_position_from_point,
    print_tree,
};
use dashmap::DashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use tracing::debug;
use tree_sitter::{Query, Tree};

use super::utils::get_variable_locations_for_query;

#[derive(Debug)]
pub struct Backend {
    pub client: Client,
    pub ast_map: DashMap<Url, Tree>,
    pub document_map: DashMap<Url, String>,
    pub root_path: Mutex<String>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        if let Some(root_uri) = params.root_uri {
            if let Ok(mut root_path) = self.root_path.lock() {
                *root_path = String::from(root_uri.path());
            }
        }

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL), // we probably want incremental?
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(true),
                        })),
                        will_save: Some(true),            //idk what this does
                        will_save_wait_until: Some(true), //idk what this does
                    },
                )),
                definition_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        debug!("Goto definition params: {:?}", params);

        let tree = self
            .ast_map
            .get(&params.text_document_position_params.text_document.uri)
            .expect("to get the tree");

        let document = self
            .document_map
            .get(&params.text_document_position_params.text_document.uri)
            .expect("to get the document");

        let current_point = get_point_from_position(&params.text_document_position_params.position);
        let current_node = get_node_for_point(&tree, current_point).expect("to get node");
        debug!("Current node: {:?}", current_node.kind());
        let parent = current_node
            .parent()
            .expect("to get parent of current node");
        debug!("Parent node: {:?}", parent.kind());

        match parent.kind() {
            "variable_name" => {
                let location = self
                    .find_variable_declaration(
                        &current_node,
                        &document,
                        &params.text_document_position_params.text_document.uri,
                        &tree,
                    )
                    .expect("to find variable declaration");

                return Ok(Some(GotoDefinitionResponse::Scalar(location)));
            }
            "named_type" => {
                let location = self
                    .find_class_definition()
                    .expect("to find class definition");

                return Ok(Some(GotoDefinitionResponse::Scalar(location)));
            }
            _ => return Ok(None),
        }
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        debug!("Got a textDocument/didOpen notification");
        if self.ast_map.contains_key(&params.text_document.uri) {
            return;
        }

        let lang = tree_sitter_php::LANGUAGE_PHP;
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&lang.into()).expect("to set lang");

        //should we panic?
        let tree = parser
            .parse(params.text_document.text.clone(), None)
            .expect("to parse file");
        print_tree(&tree);

        self.ast_map.insert(params.text_document.uri.clone(), tree);
        self.document_map
            .insert(params.text_document.uri, params.text_document.text);
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

impl Backend {
    fn find_class_definition(&self) -> Option<Location> {
        let _class_map = self.get_autoload_class_map();
        None
    }

    // returns Namespace\Class -> src/class.php
    fn get_autoload_class_map(&self) -> HashMap<String, String> {
        let root_path = self
            .root_path
            .lock()
            .ok()
            .map(|root_path| root_path.clone())
            .expect("to get root path");
        let autoload_classmap_path = format!("{}/vendor/composer/autoload_classmap.php", root_path);
        debug!("Path: {:?}", autoload_classmap_path);
        let contents = std::fs::read_to_string(autoload_classmap_path).expect("to read file");
        debug!("Contents: {:?}", contents);

        let lang = tree_sitter_php::LANGUAGE_PHP;
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&lang.into()).expect("to set lang");

        let tree = parser.parse(contents, None).expect("to parse file");
        print_tree(&tree);

        HashMap::default()
    }

    fn find_variable_declaration(
        &self,
        current_node: &tree_sitter::Node,
        document: &str,
        uri: &Url,
        tree: &tree_sitter::Tree,
    ) -> Option<Location> {
        // not sure if this is the correct approach
        // we try to find every occurence of the variable name
        // and then we pick the closest location to our current node
        // maybe we should just look at the scope where the current node
        // is located and based on that determine where the variable name is declared
        // but this kinda works so fuck it

        let var_declare_query = Query::new(
            &tree_sitter_php::LANGUAGE_PHP.into(),
            "(assignment_expression left: (variable_name) @declaration)",
        )
        .expect("to create variable declaration query");

        let var_param_query = Query::new(
            &tree_sitter_php::LANGUAGE_PHP.into(),
            "(simple_parameter (variable_name) @declaration)",
        )
        .expect("to create parameter query");

        let var_name = current_node
            .utf8_text(document.as_bytes())
            .expect("to get current variable name");
        let mut locations: Vec<Location> = vec![];

        locations.append(&mut get_variable_locations_for_query(
            var_name,
            &var_declare_query,
            tree,
            document,
            uri,
        ));
        locations.append(&mut get_variable_locations_for_query(
            var_name,
            &var_param_query,
            tree,
            document,
            uri,
        ));
        debug!("Locations: {:?}", locations);

        find_nearest_location(
            get_position_from_point(&current_node.start_position()),
            locations,
        )
    }
}
