use std::path::PathBuf;
use std::sync::Mutex;

use crate::tree::utils::{
    find_nearest_location, get_node_for_point, get_point_from_position, get_position_from_point,
    print_tree,
};
use dashmap::DashMap;
use streaming_iterator::StreamingIterator;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::LanguageServer;
use tracing::debug;
use tree_sitter::{Query, QueryCursor, Tree};

use super::utils::get_variable_locations_for_query;

#[derive(Debug)]
pub struct Backend {
    //pub client: Client,
    pub ast_map: DashMap<Url, Tree>,
    pub document_map: DashMap<Url, String>,
    pub root_path: Mutex<String>,
    pub class_map: DashMap<String, String>,
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
        // self.client
        //     .log_message(MessageType::INFO, "server initialized!")
        //     .await;
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
                    .find_class_definition(
                        &current_node,
                        &document,
                        &tree,
                        &params.text_document_position_params.text_document.uri,
                    )
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

        self.load_autoload_class_map();
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

impl Backend {
    fn find_class_definition(
        &self,
        current_node: &tree_sitter::Node,
        document: &str,
        tree: &Tree,
        current_uri: &Url,
    ) -> Option<Location> {
        let class_name = current_node
            .utf8_text(document.as_bytes())
            .expect("to get class name");

        let query = Query::new(
            &tree_sitter_php::LANGUAGE_PHP.into(),
            "(namespace_use_clause
                (qualified_name) @namespace)
            ",
        )
        .expect("to create query");
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), document.as_bytes());

        let file_path = current_uri.to_file_path().unwrap();
        let current_dir = file_path.parent().unwrap();
        debug!("Current directory, {:?}", current_dir);

        while let Some(match_) = matches.next() {
            for capture in match_.captures {
                let fqn = capture
                    .node
                    .utf8_text(document.as_bytes())
                    .expect("to get use statement");

                debug!("FQN: {}", fqn);
                debug!("class_name: {}", class_name);

                let path = self.class_map.get(fqn);

                if path.is_none() {
                    continue;
                }

                if fqn.ends_with(format!("\\{}", class_name).as_str()) {
                    debug!("found: {}", fqn);
                    let path = path.unwrap();
                    let location = self.get_class_declaration_location(
                        &PathBuf::from(&path.to_owned()),
                        class_name,
                    );

                    if location.is_some() {
                        return Some(location.unwrap());
                    }
                }
            }
        }

        //if there is no use statement try searching the current directory for the class
        // TODO: maybe instead of searching all of the files we could assume that
        // it would live at current_dir/class_name?
        let files = std::fs::read_dir(current_dir).expect("to read files");
        for entry in files {
            if entry.is_err() {
                continue;
            }
            let entry = entry.unwrap();
            let path = entry.path();

            // we dont care about nested directories since the class would
            // have to been defined by a use statement which is handled above
            if path.is_dir() {
                continue;
            }

            if let Some(location) = self.get_class_declaration_location(&path, class_name) {
                return Some(location);
            }
        }

        None
    }

    fn get_class_declaration_location(&self, path: &PathBuf, class_name: &str) -> Option<Location> {
        if path.is_dir() {
            return None;
        }

        let content = std::fs::read_to_string(path).expect("to read destination file");

        let lang = tree_sitter_php::LANGUAGE_PHP;
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&lang.into()).expect("to set lang");

        let tree = parser.parse(content.clone(), None).expect("to parse file");
        print_tree(&tree);

        let query = Query::new(
            &tree_sitter_php::LANGUAGE_PHP.into(),
            "(class_declaration
                (name) @class_name)
            ",
        )
        .expect("to create query");

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), content.as_bytes());

        while let Some(match_) = matches.next() {
            for capture in match_.captures {
                let node = capture.node;
                let node_text = node
                    .utf8_text(content.as_bytes())
                    .expect("to get class name");
                if node_text == class_name {
                    return Some(Location::new(
                        Url::from_file_path(&path).unwrap(),
                        Range::new(
                            get_position_from_point(&node.start_position()),
                            get_position_from_point(&node.end_position()),
                        ),
                    ));
                }
            }
        }

        None
    }

    // returns Namespace\Class -> src/class.php
    fn load_autoload_class_map(&self) {
        let root_path = self
            .root_path
            .lock()
            .ok()
            .map(|root_path| root_path.clone())
            .expect("to get root path");
        let autoload_classmap_path = format!("{}/vendor/composer/autoload_classmap.php", root_path);
        let vendor_path = format!("{}/vendor", root_path);

        debug!("Path: {:?}", autoload_classmap_path);
        let contents = std::fs::read(autoload_classmap_path).expect("to read file");

        let lang = tree_sitter_php::LANGUAGE_PHP;
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&lang.into()).expect("to set lang");

        let tree = parser.parse(contents.clone(), None).expect("to parse file");
        let query = Query::new(
            &tree_sitter_php::LANGUAGE_PHP.into(),
            r#"
            (array_creation_expression 
                (array_element_initializer
                    (string) @namespace
                    (binary_expression
                        (variable_name) @dir 
                        "." 
            (string) @path)))
        "#,
        )
        .expect("to create query");
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), contents.as_slice());

        while let Some(match_) = matches.next() {
            let mut namespace = None;
            let mut path = None;
            let mut dir = None;

            for capture in match_.captures {
                let node = capture.node;
                let text = &contents[node.byte_range()];

                match query.capture_names()[capture.index as usize] {
                    "namespace" => namespace = Some(text),
                    "dir" => dir = Some(text),
                    "path" => path = Some(text),
                    _ => {}
                }
            }

            if let (Some(key_bytes), Some(base_dir_bytes), Some(path_bytes)) =
                (namespace, dir, path)
            {
                if let (Ok(key_string), Ok(base_dir_string), Ok(path_string)) = (
                    String::from_utf8(key_bytes.to_vec()),
                    String::from_utf8(base_dir_bytes.to_vec()),
                    String::from_utf8(path_bytes.to_vec()),
                ) {
                    let path_string = path_string.trim_matches('\'').to_string();
                    let full_path = match base_dir_string.as_str() {
                        "$vendorDir" => format!("{}{}", vendor_path, path_string),
                        "$baseDir" => format!("{}{}", root_path, path_string),
                        _ => path_string,
                    };

                    let namespace = key_string
                        .trim_matches('\'')
                        .replace("\\\\", "\\")
                        .to_string();
                    let full_path = full_path.to_string();
                    // debug!("Namespace: {} => path: {}", namespace, full_path);
                    self.class_map.insert(namespace, full_path);
                }
            }
        }
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

#[cfg(test)]
mod tests {
    use std::{path::Path, io::Write, sync::Mutex, collections::HashMap};
    use dashmap::DashMap;
    use tempfile::TempDir;
    use crate::lsp::lsp::Backend;

    #[tokio::test]
    async fn load_class_map() {
        let temp_dir = TempDir::new().expect("to initialize temp dir");
        let temp_dir_path = temp_dir.path();
        let mut expected_map = HashMap::new();

        let vendor_dir = format!("{}/vendor", temp_dir_path.to_str().unwrap());

        expected_map.insert(
            String::from("Symfony\\Component\\DependencyInjection\\Argument\\RewindableGenerator"),
            format!("{}/symfony/dependency-injection/Argument/RewindableGenerator.php", vendor_dir), 
        );

        expected_map.insert(
            String::from("Symfony\\Component\\DependencyInjection\\Argument\\ServiceClosureArgument"),
            format!("{}/symfony/dependency-injection/Argument/ServiceClosureArgument.php", vendor_dir), 
        );

        expected_map.insert(
            String::from("MyApplicationNamespace\\Testing\\Class"),
            String::from(format!("{}/src/testing/Class.php", temp_dir_path.to_str().unwrap())),
        );

        let lang_server = Backend {
            ast_map: DashMap::default(),
            document_map: DashMap::default(),
            root_path: Mutex::new(String::from(temp_dir_path.to_str().unwrap())),
            class_map: DashMap::default(),
        };
        prepare_autload_file(temp_dir_path);

        lang_server.load_autoload_class_map();

        assert_eq!(lang_server.class_map.is_empty(), false);
        assert_eq!(lang_server.class_map.iter().count(), 3);

        for (key, val) in lang_server.class_map.into_iter() {
            let expected = expected_map.get(&key).expect("to find namespace");
            assert_eq!(*expected, val);
        }
    }

    fn prepare_autload_file(root: &Path) {
        let file_contents = r#"
            <?php

            // autoload_classmap.php @generated by Composer

            $vendorDir = dirname(__DIR__);
            $baseDir = dirname($vendorDir);

            return array(
                'Symfony\\Component\\DependencyInjection\\Argument\\RewindableGenerator' => $vendorDir . '/symfony/dependency-injection/Argument/RewindableGenerator.php',
                'Symfony\\Component\\DependencyInjection\\Argument\\ServiceClosureArgument' => $vendorDir . '/symfony/dependency-injection/Argument/ServiceClosureArgument.php',
                'MyApplicationNamespace\\Testing\\Class' => $baseDir . '/src/testing/Class.php',
            );
        "#;

        let composer_dir = format!("{}/vendor/composer", root.to_str().unwrap());
        let file_path = format!("{}/vendor/composer/autoload_classmap.php", root.to_str().unwrap());
        let vendor_composer_path = Path::new(&composer_dir);
        std::fs::create_dir_all(vendor_composer_path).expect("to create /vendor/composer directory");

        let mut file = std::fs::File::create(file_path).expect("to create file");
        file.write_all(file_contents.as_bytes()).expect("to write to file");
    }
}
