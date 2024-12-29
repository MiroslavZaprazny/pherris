use crate::tree::class::ClassDefinition;
use crate::tree::utils::*;
use dashmap::DashMap;
use std::path::PathBuf;
use streaming_iterator::StreamingIterator;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use tracing::debug;
use tree_sitter::{Query, QueryCursor, Tree};
use walkdir::WalkDir;

use super::utils::get_variable_locations_for_query;

pub struct Backend {
    pub client: Client,
    pub ast_map: DashMap<Url, Tree>,
    pub document_map: DashMap<Url, String>,
    pub class_index: DashMap<String, ClassDefinition>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        // maybe we should find it if its not in the params?
        if let Some(root_uri) = params.root_uri {
            let _ = self.index_workspace(PathBuf::from(root_uri.path())).await;
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

        match current_node.kind() {
            "name" => {
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
            _ => return Ok(None),
        }
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        debug!("Got a textDocument/didOpen notification");

        if self.ast_map.contains_key(&params.text_document.uri) == false {
            let lang = tree_sitter_php::LANGUAGE_PHP;
            let mut parser = tree_sitter::Parser::new();
            parser.set_language(&lang.into()).expect("to set lang");

            let tree = parser
                .parse(params.text_document.text.clone(), None)
                .expect("to parse file");

            print_tree(&tree);
            self.ast_map.insert(params.text_document.uri.clone(), tree);
        }

        if self.document_map.contains_key(&params.text_document.uri) == false {
            self.document_map
                .insert(params.text_document.uri, params.text_document.text);
        }
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

impl Backend {
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

    async fn index_workspace(&self, dir: PathBuf) -> Result<()> {
        debug!("Starting to index workspace, root_dir: {:?}", dir);

        let lang = tree_sitter_php::LANGUAGE_PHP;
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&lang.into()).expect("to set lang");

        for entry in WalkDir::new(&dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "php"))
        {
            let path = entry.path();
            debug!("Indexing: {:?}", path);
            let content = tokio::fs::read_to_string(path).await.expect("to read file");
            let tree = parser.parse(&content, None).expect("to parse tree");
            let uri = Url::from_file_path(path).unwrap();
            self.update_index(&uri, &tree, content.as_str());
        }

        debug!("Indexing finished");

        Ok(())
    }

    fn get_namespace(&self, tree: &Tree, document: &str) -> Option<String> {
        let query = Query::new(
            &tree_sitter_php::LANGUAGE_PHP.into(),
            "(namespace_definition
              name: (namespace_name) @namespace_name
            )",
        )
        .expect("to create namespace definition query");

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), document.as_bytes());

        while let Some(match_) = matches.next() {
            for capture in match_.captures {
                return Some(
                    capture
                        .node
                        .utf8_text(document.as_bytes())
                        .unwrap()
                        .to_string(),
                );
            }
        }

        None
    }

    fn update_index(&self, uri: &Url, tree: &Tree, document: &str) {
        let class_def_query = Query::new(
            &tree_sitter_php::LANGUAGE_PHP.into(),
            "(class_declaration name: (name) @class_name)",
        )
        .expect("to create class definition query");

        let mut class_cursor = QueryCursor::new();
        let mut class_def_matches =
            class_cursor.matches(&class_def_query, tree.root_node(), document.as_bytes());
        let namespace = self.get_namespace(tree, document);

        if namespace.is_none() {
            return;
        }
        let namespace = namespace.unwrap();

        while let Some(match_) = class_def_matches.next() {
            for capture in match_.captures {
                let class_name = capture
                    .node
                    .utf8_text(document.as_bytes())
                    .ok()
                    .map(String::from)
                    .unwrap();

                self.class_index.insert(
                    format!("{}\\{}", namespace, class_name),
                    ClassDefinition {
                        uri: uri.clone(),
                        range: Range::new(
                            get_position_from_point(&capture.node.start_position()),
                            get_position_from_point(&capture.node.end_position()),
                        ),
                    },
                );
            }
        }
    }
}
