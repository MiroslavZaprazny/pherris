use std::path::PathBuf;
use dashmap::DashMap;
use tracing::debug;
use crate::{lsp::locator::traits::Locator, tree::utils::get_position_from_point};
use tower_lsp::lsp_types::{Location, Url, Range};
use tree_sitter::{Node, Tree, Query, QueryCursor};
use streaming_iterator::StreamingIterator;

pub struct ClassLocator {
    class_map: DashMap<String, String>,
}

impl Locator for ClassLocator {
    fn find(
        &self,
        current_node: &Node,
        document: &str,
        tree: &Tree,
        current_uri: &Url,
    ) -> Option<Location>
    {
        let class_name = current_node
            .utf8_text(document.as_bytes())
            .expect("to get class name");

        if let Some(location) = self.locate_class_by_use_statement(class_name, document, tree) {
            return Some(location);
        }

        //if there is no use statement search the current directory for the class
        //TODO: maybe instead of searching all of the files we could assume that
        //it would live at current_dir/class_name.php?
        
        let file_path = current_uri.to_file_path().unwrap();
        let current_dir = file_path.parent().unwrap();
        debug!("Current directory, {:?}", current_dir);
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

            if let Some(location) = self.locate_class_from_path(&path, class_name) {
                return Some(location);
            }
        }

        None
    }
}

impl ClassLocator {
    pub fn new(class_map: DashMap<String, String>) -> ClassLocator {
        ClassLocator { class_map }
    }

    fn locate_class_by_use_statement(
        &self, 
        class_name: &str,
        document: &str,
        tree: &Tree,
    ) -> Option<Location> {
        let query = Query::new(
            &tree_sitter_php::LANGUAGE_PHP.into(),
            "(namespace_use_clause
                (qualified_name) @namespace)
            ",
        )
        .expect("to create query");
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), document.as_bytes());

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
                    let path = path.unwrap();
                    let location = self.locate_class_from_path(
                        &PathBuf::from(&path.to_owned()),
                        class_name,
                    );

                    if location.is_some() {
                        return Some(location.unwrap());
                    }
                }
            }
        }

        None
    }

    fn locate_class_from_path(&self, path: &PathBuf, class_name: &str) -> Option<Location> {
        if path.is_dir() {
            return None;
        }

        let content = std::fs::read_to_string(path).expect("to read destination file");

        let lang = tree_sitter_php::LANGUAGE_PHP;
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&lang.into()).expect("to set lang");

        let tree = parser.parse(content.clone(), None).expect("to parse file");

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
}


