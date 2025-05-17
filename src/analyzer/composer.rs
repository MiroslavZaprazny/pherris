use std::sync::RwLock;

use streaming_iterator::StreamingIterator;
use tree_sitter::{Query, QueryCursor};

use crate::lsp::state::State;

use super::parser::Parser;

//TODO: if the autoload map is not present we should probably
//index the project ourselfs
//TODO: Use mago parser instead of tree sitter
pub fn load_autoload_class_map(parser: &RwLock<Parser>, state: &State) {
    let root_path = state.root_path.read().unwrap();
    let autoload_classmap_path = format!("{}/vendor/composer/autoload_classmap.php", *root_path);
    let vendor_path = format!("{}/vendor", *root_path);

    let contents = std::fs::read(autoload_classmap_path);

    if contents.is_err() {
        return;
    }
    let contents = contents.unwrap();

    let tree = parser
        .write()
        .unwrap()
        .parse(contents.clone())
        .expect("to parse file");
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

        if let (Some(key_bytes), Some(base_dir_bytes), Some(path_bytes)) = (namespace, dir, path) {
            if let (Ok(key_string), Ok(base_dir_string), Ok(path_string)) = (
                String::from_utf8(key_bytes.to_vec()),
                String::from_utf8(base_dir_bytes.to_vec()),
                String::from_utf8(path_bytes.to_vec()),
            ) {
                let path_string = path_string.trim_matches('\'').to_string();
                let full_path = match base_dir_string.as_str() {
                    "$vendorDir" => format!("{}{}", vendor_path, path_string),
                    "$baseDir" => format!("{}{}", *root_path, path_string),
                    _ => path_string,
                };

                let namespace = key_string
                    .trim_matches('\'')
                    .replace("\\\\", "\\")
                    .to_string();
                let full_path = full_path.to_string();
                state.class_map.insert(namespace, full_path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        analyzer::{composer::load_autoload_class_map, parser::Parser},
        lsp::state::State,
    };
    use dashmap::DashMap;
    use std::{collections::HashMap, io::Write, path::Path, sync::RwLock};
    use tempfile::TempDir;

    #[tokio::test]
    async fn load_class_map() {
        let temp_dir = TempDir::new().expect("to initialize temp dir");
        let temp_dir_path = temp_dir.path();
        let mut expected_map = HashMap::new();

        let vendor_dir = format!("{}/vendor", temp_dir_path.to_str().unwrap());

        expected_map.insert(
            String::from("Symfony\\Component\\DependencyInjection\\Argument\\RewindableGenerator"),
            format!(
                "{}/symfony/dependency-injection/Argument/RewindableGenerator.php",
                vendor_dir
            ),
        );

        expected_map.insert(
            String::from(
                "Symfony\\Component\\DependencyInjection\\Argument\\ServiceClosureArgument",
            ),
            format!(
                "{}/symfony/dependency-injection/Argument/ServiceClosureArgument.php",
                vendor_dir
            ),
        );

        expected_map.insert(
            String::from("MyApplicationNamespace\\Testing\\Class"),
            format!("{}/src/testing/Class.php", temp_dir_path.to_str().unwrap()),
        );

        prepare_autload_file(temp_dir_path);
        let state = State::new(
            DashMap::default(),
            DashMap::default(),
            RwLock::new(String::from(temp_dir_path.to_str().unwrap())),
            DashMap::default(),
            DashMap::default(),
        );

        load_autoload_class_map(&RwLock::new(Parser::new().unwrap()), &state);

        assert!(!state.class_map.is_empty());
        assert_eq!(state.class_map.iter().count(), 3);

        for (key, val) in state.class_map.into_iter() {
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
        let file_path = format!(
            "{}/vendor/composer/autoload_classmap.php",
            root.to_str().unwrap()
        );
        let vendor_composer_path = Path::new(&composer_dir);
        std::fs::create_dir_all(vendor_composer_path)
            .expect("to create /vendor/composer directory");

        let mut file = std::fs::File::create(file_path).expect("to create file");
        file.write_all(file_contents.as_bytes())
            .expect("to write to file");
    }
}
