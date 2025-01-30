use dashmap::DashMap;
use pherris::analyzer::parser::Parser;
use pherris::handlers::request::handle_go_to_definition;
use pherris::lsp::state::State;
use std::io::Write;
use std::path::Path;
use std::sync::RwLock;
use tempfile::TempDir;
use tower_lsp::lsp_types::{GotoDefinitionResponse, Position, Url};

#[test]
fn test_find_variable_decleration() {
    let file_contents = r#"<?php
        $variable = 0;
        echo $variable;
    "#;
    let temp_dir = TempDir::new().expect("to initialize temp dir");
    let temp_dir_path = temp_dir.path();

    let uri = Url::from_file_path(temp_dir_path).unwrap();
    let mut parser = Parser::new().expect("to create a parser");
    let tree = parser.parse(file_contents).expect("to parse file");

    let ast_map = DashMap::new();
    ast_map.insert(uri.clone(), tree);
    let document_map = DashMap::new();
    document_map.insert(uri.clone(), String::from(file_contents));

    prepare_php_file(temp_dir_path, file_contents);
    let state = State::new(
        ast_map,
        document_map,
        RwLock::new(String::from(temp_dir_path.to_str().unwrap())),
        DashMap::default(),
    );

    let response =
        handle_go_to_definition(&uri, &Position::new(2, 15), &state, &RwLock::new(parser));
    assert!(response.is_some());
    if let GotoDefinitionResponse::Scalar(response) = response.unwrap() {
        assert_eq!(response.range.start.line, 1);
        assert_eq!(response.range.start.character, 8);
    } else {
        panic!("response is not a location");
    }
}

#[test]
fn test_find_shadowed_variable_decleration() {
    let file_contents = r#"<?php
        $variable = 0;
        $variable = 1;
        echo $variable;
    "#;
    let temp_dir = TempDir::new().expect("to initialize temp dir");
    let temp_dir_path = temp_dir.path();

    let uri = Url::from_file_path(temp_dir_path).unwrap();
    let mut parser = Parser::new().expect("to create a parser");
    let tree = parser.parse(file_contents).expect("to parse file");

    let ast_map = DashMap::new();
    ast_map.insert(uri.clone(), tree);
    let document_map = DashMap::new();
    document_map.insert(uri.clone(), String::from(file_contents));

    prepare_php_file(temp_dir_path, file_contents);
    let state = State::new(
        ast_map,
        document_map,
        RwLock::new(String::from(temp_dir_path.to_str().unwrap())),
        DashMap::default(),
    );

    let response =
        handle_go_to_definition(&uri, &Position::new(2, 15), &state, &RwLock::new(parser));
    assert!(response.is_some());
    if let GotoDefinitionResponse::Scalar(response) = response.unwrap() {
        assert_eq!(response.range.start.line, 2);
        assert_eq!(response.range.start.character, 8);
    } else {
        panic!("response is not a location");
    }
}

#[test]
fn test_find_variable_definition_in_array_function() {
    let file_contents = r#"<?php
        array_map(fn ($test) => $test, []);
    "#;
    let temp_dir = TempDir::new().expect("to initialize temp dir");
    let temp_dir_path = temp_dir.path();

    let uri = Url::from_file_path(temp_dir_path).unwrap();
    let mut parser = Parser::new().expect("to create a parser");
    let tree = parser.parse(file_contents).expect("to parse file");

    let ast_map = DashMap::new();
    ast_map.insert(uri.clone(), tree);
    let document_map = DashMap::new();
    document_map.insert(uri.clone(), String::from(file_contents));

    prepare_php_file(temp_dir_path, file_contents);
    let state = State::new(
        ast_map,
        document_map,
        RwLock::new(String::from(temp_dir_path.to_str().unwrap())),
        DashMap::default(),
    );

    let response =
        handle_go_to_definition(&uri, &Position::new(2, 15), &state, &RwLock::new(parser));
    assert!(response.is_some());
    if let GotoDefinitionResponse::Scalar(response) = response.unwrap() {
        assert_eq!(response.range.start.line, 2);
        assert_eq!(response.range.start.character, 22);
    } else {
        panic!("response is not a location");
    }
}

fn prepare_php_file(root: &Path, file_contents: &str) {
    let file_path = format!("{}/test_file.php", root.to_str().unwrap());

    let mut file = std::fs::File::create(file_path).expect("to create file");
    file.write_all(file_contents.as_bytes())
        .expect("to write to file");
}
