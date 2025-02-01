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
    let path_str = format!("{}/{}", temp_dir.path().to_str().unwrap(), "test.php");
    let temp_dir_path = Path::new(&path_str);
    prepare_php_file(temp_dir_path, file_contents);

    let uri = Url::from_file_path(temp_dir_path).unwrap();
    let mut parser = Parser::new().expect("to create a parser");
    let tree = parser.parse(file_contents).expect("to parse file");

    let ast_map = DashMap::new();
    ast_map.insert(uri.clone(), tree);
    let document_map = DashMap::new();
    document_map.insert(uri.clone(), String::from(file_contents));

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
        assert_eq!(response.uri.as_str(), uri.as_str());
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
    let path_str = format!("{}/{}", temp_dir.path().to_str().unwrap(), "test.php");
    let temp_dir_path = Path::new(&path_str);
    prepare_php_file(temp_dir_path, file_contents);

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
        assert_eq!(response.uri.as_str(), uri.as_str());
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
    let path_str = format!("{}/{}", temp_dir.path().to_str().unwrap(), "test.php");
    let temp_dir_path = Path::new(&path_str);
    prepare_php_file(temp_dir_path, file_contents);

    let uri = Url::from_file_path(temp_dir_path).unwrap();
    let mut parser = Parser::new().expect("to create a parser");
    let tree = parser.parse(file_contents).expect("to parse file");

    let ast_map = DashMap::new();
    ast_map.insert(uri.clone(), tree);
    let document_map = DashMap::new();
    document_map.insert(uri.clone(), String::from(file_contents));

    let state = State::new(
        ast_map,
        document_map,
        RwLock::new(String::from(temp_dir_path.to_str().unwrap())),
        DashMap::default(),
    );

    let response =
        handle_go_to_definition(&uri, &Position::new(1, 35), &state, &RwLock::new(parser));
    assert!(response.is_some());
    if let GotoDefinitionResponse::Scalar(response) = response.unwrap() {
        assert_eq!(response.uri.as_str(), uri.as_str());
        assert_eq!(response.range.start.line, 1);
        assert_eq!(response.range.start.character, 22);
    } else {
        panic!("response is not a location");
    }
}

#[test]
fn test_find_variable_definition_in_function() {
    let file_contents = r#"<?php
        function ($test) {
            echo $test;
        }
    "#;
    let temp_dir = TempDir::new().expect("to initialize temp dir");
    let path_str = format!("{}/{}", temp_dir.path().to_str().unwrap(), "test.php");
    let temp_dir_path = Path::new(&path_str);
    prepare_php_file(temp_dir_path, file_contents);

    let uri = Url::from_file_path(temp_dir_path).unwrap();
    let mut parser = Parser::new().expect("to create a parser");
    let tree = parser.parse(file_contents).expect("to parse file");

    let ast_map = DashMap::new();
    ast_map.insert(uri.clone(), tree);
    let document_map = DashMap::new();
    document_map.insert(uri.clone(), String::from(file_contents));

    let state = State::new(
        ast_map,
        document_map,
        RwLock::new(String::from(temp_dir_path.to_str().unwrap())),
        DashMap::default(),
    );

    let response =
        handle_go_to_definition(&uri, &Position::new(2, 20), &state, &RwLock::new(parser));
    assert!(response.is_some());
    if let GotoDefinitionResponse::Scalar(response) = response.unwrap() {
        assert_eq!(response.uri.as_str(), uri.as_str());
        assert_eq!(response.range.start.line, 1);
        assert_eq!(response.range.start.character, 18);
    } else {
        panic!("response is not a location");
    }
}

#[test]
fn test_find_class_definition_in_same_folder() {
    let file_contents = r#"<?php
        function (MyClass $test) {
            echo $test;
        }
    "#;
    let temp_dir = TempDir::new().expect("to initialize temp dir");
    let path_str = format!("{}/{}", temp_dir.path().to_str().unwrap(), "test.php");
    let temp_dir_path = Path::new(&path_str);
    let target_uri = Url::from_file_path(temp_dir_path).unwrap();

    let mut parser = Parser::new().expect("to create a parser");
    let tree = parser.parse(file_contents).expect("to parse file");

    let ast_map = DashMap::new();
    ast_map.insert(target_uri.clone(), tree);
    let document_map = DashMap::new();
    document_map.insert(target_uri.clone(), String::from(file_contents));

    let state = State::new(
        ast_map,
        document_map,
        RwLock::new(String::from(temp_dir_path.to_str().unwrap())),
        DashMap::default(),
    );

    prepare_php_file(temp_dir_path, file_contents);

    let file_contents = r#"<?php
        namespace MyNamespace;

        class MyClass {}
    "#;

    let path_str = format!("{}/{}", temp_dir.path().to_str().unwrap(), "class.php");
    let temp_dir_path = Path::new(&path_str);
    let class_uri = Url::from_file_path(temp_dir_path).unwrap();

    prepare_php_file(temp_dir_path, file_contents);

    let response = handle_go_to_definition(
        &target_uri,
        &Position::new(1, 19),
        &state,
        &RwLock::new(parser),
    );
    assert!(response.is_some());
    if let GotoDefinitionResponse::Scalar(response) = response.unwrap() {
        assert_eq!(response.uri.as_str(), class_uri.as_str());
        assert_eq!(response.range.start.line, 3);
        assert_eq!(response.range.start.character, 14);
    } else {
        panic!("response is not a location");
    }
}

fn prepare_php_file(file_path: &Path, file_contents: &str) {
    let mut file = std::fs::File::create(file_path).expect("to create file");
    file.write_all(file_contents.as_bytes())
        .expect("to write to file");
}
