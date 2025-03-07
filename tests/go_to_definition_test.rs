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
fn test_find_variable_definition_in_foreach() {
    let file_contents = r#"<?php
        $list = [];
        foreach ($list as $element) {
            echo $element;
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
        handle_go_to_definition(&uri, &Position::new(3, 18), &state, &RwLock::new(parser));
    assert!(response.is_some());
    if let GotoDefinitionResponse::Scalar(response) = response.unwrap() {
        assert_eq!(response.uri.as_str(), uri.as_str());
        assert_eq!(response.range.start.line, 2);
        assert_eq!(response.range.start.character, 26);
    } else {
        panic!("response is not a location");
    }
}

#[test]
fn test_find_variable_definition_in_foreach_pair() {
    let file_contents = r#"<?php
        $list = [];
        foreach ($list as $key => $val) {
            echo $val;
            echo $key;
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
        handle_go_to_definition(&uri, &Position::new(3, 18), &state, &RwLock::new(parser));
    assert!(response.is_some());
    if let GotoDefinitionResponse::Scalar(response) = response.unwrap() {
        assert_eq!(response.uri.as_str(), uri.as_str());
        assert_eq!(response.range.start.line, 2);
        assert_eq!(response.range.start.character, 34);
    } else {
        panic!("response is not a location");
    }

    let parser = Parser::new().expect("to create a parser");
    let response =
        handle_go_to_definition(&uri, &Position::new(4, 18), &state, &RwLock::new(parser));
    assert!(response.is_some());
    if let GotoDefinitionResponse::Scalar(response) = response.unwrap() {
        assert_eq!(response.uri.as_str(), uri.as_str());
        assert_eq!(response.range.start.line, 2);
        assert_eq!(response.range.start.character, 26);
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

#[test]
fn test_find_class_definition_in_same_folder_using_class_name_as_filename() {
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

    let path_str = format!("{}/{}", temp_dir.path().to_str().unwrap(), "MyClass.php");
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

#[test]
fn test_find_class_definition_in_different_folder() {
    let file_contents = r#"<?php
        use MyApp\Testing\MyClass;
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

    let class_map = DashMap::new();
    class_map.insert(
        String::from("MyApp\\Testing\\MyClass"),
        format!(
            "{}/{}",
            temp_dir.path().to_str().unwrap(),
            "src/testing/class.php"
        ),
    );

    let state = State::new(
        ast_map,
        document_map,
        RwLock::new(String::from(temp_dir_path.to_str().unwrap())),
        class_map,
    );

    prepare_php_file(temp_dir_path, file_contents);

    let file_contents = r#"<?php
        namespace MyApp\Testing;

        class MyClass {}
    "#;

    let path_str = format!(
        "{}/{}",
        temp_dir.path().to_str().unwrap(),
        "src/testing/class.php"
    );
    std::fs::create_dir_all(format!(
        "{}/{}",
        temp_dir.path().to_str().unwrap(),
        "src/testing"
    ))
    .expect("to create dir");
    let temp_dir_path = Path::new(&path_str);
    let class_uri = Url::from_file_path(temp_dir_path).unwrap();

    prepare_php_file(temp_dir_path, file_contents);

    let response = handle_go_to_definition(
        &target_uri,
        &Position::new(2, 19),
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

#[test]
fn test_find_enum_definition_in_same_folder() {
    let file_contents = r#"<?php
        function (MyEnum $test) {
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

        enum MyEnum {}
    "#;

    let path_str = format!("{}/{}", temp_dir.path().to_str().unwrap(), "enum.php");
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
        assert_eq!(response.range.start.character, 13);
    } else {
        panic!("response is not a location");
    }
}

#[test]
fn test_find_enum_definition_in_different_folder() {
    let file_contents = r#"<?php
        use MyApp\Testing\MyEnum;
        function (MyEnum $test) {
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

    let class_map = DashMap::new();
    class_map.insert(
        String::from("MyApp\\Testing\\MyEnum"),
        format!(
            "{}/{}",
            temp_dir.path().to_str().unwrap(),
            "src/testing/enum.php"
        ),
    );

    let state = State::new(
        ast_map,
        document_map,
        RwLock::new(String::from(temp_dir_path.to_str().unwrap())),
        class_map,
    );

    prepare_php_file(temp_dir_path, file_contents);

    let file_contents = r#"<?php
        namespace MyApp\Testing;

        enum MyEnum {}
    "#;

    let path_str = format!(
        "{}/{}",
        temp_dir.path().to_str().unwrap(),
        "src/testing/enum.php"
    );
    std::fs::create_dir_all(format!(
        "{}/{}",
        temp_dir.path().to_str().unwrap(),
        "src/testing"
    ))
    .expect("to create dir");
    let temp_dir_path = Path::new(&path_str);
    let class_uri = Url::from_file_path(temp_dir_path).unwrap();

    prepare_php_file(temp_dir_path, file_contents);

    let response = handle_go_to_definition(
        &target_uri,
        &Position::new(2, 19),
        &state,
        &RwLock::new(parser),
    );
    assert!(response.is_some());
    if let GotoDefinitionResponse::Scalar(response) = response.unwrap() {
        assert_eq!(response.uri.as_str(), class_uri.as_str());
        assert_eq!(response.range.start.line, 3);
        assert_eq!(response.range.start.character, 13);
    } else {
        panic!("response is not a location");
    }
}

#[test]
fn test_find_interface_definition_in_same_folder() {
    let file_contents = r#"<?php
        function (MyInterface $test) {
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

        interface MyInterface {}
    "#;

    let path_str = format!("{}/{}", temp_dir.path().to_str().unwrap(), "interface.php");
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
        assert_eq!(response.range.start.character, 18);
    } else {
        panic!("response is not a location");
    }
}

#[test]
fn test_find_interface_definition_in_different_folder() {
    let file_contents = r#"<?php
        use MyApp\Testing\MyInterface;
        function (MyInterface $test) {
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

    let class_map = DashMap::new();
    class_map.insert(
        String::from("MyApp\\Testing\\MyInterface"),
        format!(
            "{}/{}",
            temp_dir.path().to_str().unwrap(),
            "src/testing/interface.php"
        ),
    );

    let state = State::new(
        ast_map,
        document_map,
        RwLock::new(String::from(temp_dir_path.to_str().unwrap())),
        class_map,
    );

    prepare_php_file(temp_dir_path, file_contents);

    let file_contents = r#"<?php
        namespace MyApp\Testing;

        interface MyInterface {}
    "#;

    let path_str = format!(
        "{}/{}",
        temp_dir.path().to_str().unwrap(),
        "src/testing/interface.php"
    );
    std::fs::create_dir_all(format!(
        "{}/{}",
        temp_dir.path().to_str().unwrap(),
        "src/testing"
    ))
    .expect("to create dir");
    let temp_dir_path = Path::new(&path_str);
    let class_uri = Url::from_file_path(temp_dir_path).unwrap();

    prepare_php_file(temp_dir_path, file_contents);

    let response = handle_go_to_definition(
        &target_uri,
        &Position::new(2, 19),
        &state,
        &RwLock::new(parser),
    );
    assert!(response.is_some());
    if let GotoDefinitionResponse::Scalar(response) = response.unwrap() {
        assert_eq!(response.uri.as_str(), class_uri.as_str());
        assert_eq!(response.range.start.line, 3);
        assert_eq!(response.range.start.character, 18);
    } else {
        panic!("response is not a location");
    }
}

#[test]
fn test_go_to_class_definition_on_use_statement() {
    let file_contents = r#"<?php
        use MyApp\Testing\MyClass;
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

    let class_map = DashMap::new();
    class_map.insert(
        String::from("MyApp\\Testing\\MyClass"),
        format!(
            "{}/{}",
            temp_dir.path().to_str().unwrap(),
            "src/testing/class.php"
        ),
    );

    let state = State::new(
        ast_map,
        document_map,
        RwLock::new(String::from(temp_dir_path.to_str().unwrap())),
        class_map,
    );

    prepare_php_file(temp_dir_path, file_contents);

    let file_contents = r#"<?php
        namespace MyApp\Testing;

        class MyClass {}
    "#;

    let path_str = format!(
        "{}/{}",
        temp_dir.path().to_str().unwrap(),
        "src/testing/class.php"
    );
    std::fs::create_dir_all(format!(
        "{}/{}",
        temp_dir.path().to_str().unwrap(),
        "src/testing"
    ))
    .expect("to create dir");
    let temp_dir_path = Path::new(&path_str);
    let class_uri = Url::from_file_path(temp_dir_path).unwrap();

    prepare_php_file(temp_dir_path, file_contents);

    let response = handle_go_to_definition(
        &target_uri,
        &Position::new(1, 26),
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

#[test]
fn test_go_to_class_definition_on_constant_access_expression_within_same_folder() {
    let file_contents = r#"<?php
        echo MyClass::class;
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

        class MyClass {
            public static function test(): string
            {
                return "test";
            }
        }
    "#;

    let path_str = format!("{}/{}", temp_dir.path().to_str().unwrap(), "class.php");
    let temp_dir_path = Path::new(&path_str);
    let class_uri = Url::from_file_path(temp_dir_path).unwrap();

    prepare_php_file(temp_dir_path, file_contents);

    let response = handle_go_to_definition(
        &target_uri,
        &Position::new(1, 13),
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

#[test]
fn test_go_to_class_definition_on_constant_access_expression() {
    let file_contents = r#"<?php
        use MyApp\Testing\MyClass;
        echo MyClass::class;
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

    let class_map = DashMap::new();
    class_map.insert(
        String::from("MyApp\\Testing\\MyClass"),
        format!(
            "{}/{}",
            temp_dir.path().to_str().unwrap(),
            "src/testing/class.php"
        ),
    );

    let state = State::new(
        ast_map,
        document_map,
        RwLock::new(String::from(temp_dir_path.to_str().unwrap())),
        class_map,
    );

    prepare_php_file(temp_dir_path, file_contents);

    let file_contents = r#"<?php
        namespace MyApp\Testing;

        class MyClass {
            public static function test(): string
            {
                return "test";
            }
        }
    "#;

    let path_str = format!(
        "{}/{}",
        temp_dir.path().to_str().unwrap(),
        "src/testing/class.php"
    );
    std::fs::create_dir_all(format!(
        "{}/{}",
        temp_dir.path().to_str().unwrap(),
        "src/testing"
    ))
    .expect("to create dir");
    let temp_dir_path = Path::new(&path_str);
    let class_uri = Url::from_file_path(temp_dir_path).unwrap();

    prepare_php_file(temp_dir_path, file_contents);

    let response = handle_go_to_definition(
        &target_uri,
        &Position::new(2, 13),
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

#[test]
fn test_go_to_class_definition_on_inherited_classes_within_same_folder() {
    let file_contents = r#"<?php
        class MyClass extends BaseClass {}
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

        class BaseClass {}
    "#;

    let path_str = format!("{}/{}", temp_dir.path().to_str().unwrap(), "class.php");
    let temp_dir_path = Path::new(&path_str);
    let class_uri = Url::from_file_path(temp_dir_path).unwrap();

    prepare_php_file(temp_dir_path, file_contents);

    let response = handle_go_to_definition(
        &target_uri,
        &Position::new(1, 30),
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

#[test]
fn test_go_to_class_definition_on_inherited_classes() {
    let file_contents = r#"<?php
        use MyApp\Testing\BaseClass;
        class MyClass extends BaseClass {}
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

    let class_map = DashMap::new();
    class_map.insert(
        String::from("MyApp\\Testing\\BaseClass"),
        format!(
            "{}/{}",
            temp_dir.path().to_str().unwrap(),
            "src/testing/class.php"
        ),
    );

    let state = State::new(
        ast_map,
        document_map,
        RwLock::new(String::from(temp_dir_path.to_str().unwrap())),
        class_map,
    );

    prepare_php_file(temp_dir_path, file_contents);

    let file_contents = r#"<?php
        namespace MyApp\Testing;

        class BaseClass {}
    "#;

    let path_str = format!(
        "{}/{}",
        temp_dir.path().to_str().unwrap(),
        "src/testing/class.php"
    );
    std::fs::create_dir_all(format!(
        "{}/{}",
        temp_dir.path().to_str().unwrap(),
        "src/testing"
    ))
    .expect("to create dir");
    let temp_dir_path = Path::new(&path_str);
    let class_uri = Url::from_file_path(temp_dir_path).unwrap();

    prepare_php_file(temp_dir_path, file_contents);

    let response = handle_go_to_definition(
        &target_uri,
        &Position::new(2, 30),
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

#[test]
fn test_go_to_interface_definition_on_implements_statement_within_same_folder() {
    let file_contents = r#"<?php
        class MyClass implements SomeInterface {}
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

        interface SomeInterface {}
    "#;

    let path_str = format!("{}/{}", temp_dir.path().to_str().unwrap(), "interface.php");
    let temp_dir_path = Path::new(&path_str);
    let class_uri = Url::from_file_path(temp_dir_path).unwrap();

    prepare_php_file(temp_dir_path, file_contents);

    let response = handle_go_to_definition(
        &target_uri,
        &Position::new(1, 37),
        &state,
        &RwLock::new(parser),
    );
    assert!(response.is_some());
    if let GotoDefinitionResponse::Scalar(response) = response.unwrap() {
        assert_eq!(response.uri.as_str(), class_uri.as_str());
        assert_eq!(response.range.start.line, 3);
        assert_eq!(response.range.start.character, 18);
    } else {
        panic!("response is not a location");
    }
}

#[test]
fn test_go_to_interface_definition_on_implements_statement() {
    let file_contents = r#"<?php
        use MyApp\Testing\SomeInterface;
        class MyClass implements SomeInterface {}
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

    let class_map = DashMap::new();
    class_map.insert(
        String::from("MyApp\\Testing\\SomeInterface"),
        format!(
            "{}/{}",
            temp_dir.path().to_str().unwrap(),
            "src/testing/interface.php"
        ),
    );

    let state = State::new(
        ast_map,
        document_map,
        RwLock::new(String::from(temp_dir_path.to_str().unwrap())),
        class_map,
    );

    prepare_php_file(temp_dir_path, file_contents);

    let file_contents = r#"<?php
        namespace MyApp\Testing;

        interface SomeInterface {}
    "#;

    let path_str = format!(
        "{}/{}",
        temp_dir.path().to_str().unwrap(),
        "src/testing/interface.php"
    );
    std::fs::create_dir_all(format!(
        "{}/{}",
        temp_dir.path().to_str().unwrap(),
        "src/testing"
    ))
    .expect("to create dir");
    let temp_dir_path = Path::new(&path_str);
    let class_uri = Url::from_file_path(temp_dir_path).unwrap();

    prepare_php_file(temp_dir_path, file_contents);

    let response = handle_go_to_definition(
        &target_uri,
        &Position::new(2, 37),
        &state,
        &RwLock::new(parser),
    );
    assert!(response.is_some());
    if let GotoDefinitionResponse::Scalar(response) = response.unwrap() {
        assert_eq!(response.uri.as_str(), class_uri.as_str());
        assert_eq!(response.range.start.line, 3);
        assert_eq!(response.range.start.character, 18);
    } else {
        panic!("response is not a location");
    }
}

#[test]
fn test_go_to_class_definition_on_object_creation_expression_within_same_folder() {
    let main_content = r#"<?php
        $obj = new MyClass();
    "#;

    let class_content = r#"<?php
        namespace MyNamespace;

        class MyClass {}
    "#;

    let (state, temp_dir, target_uri, parser_lock) =
        setup_test_environment(main_content, vec![("class.php", class_content)], vec![]);

    let response =
        handle_go_to_definition(&target_uri, &Position::new(1, 19), &state, &parser_lock);

    let class_path = format!("{}/{}", temp_dir.path().to_str().unwrap(), "class.php");
    let class_uri = Url::from_file_path(Path::new(&class_path)).unwrap();
    assert_definition_response(response, &class_uri, 3, 14);
}

#[test]
fn test_go_to_class_definition_on_object_creation_expression() {
    let main_content = r#"<?php
        use MyApp\Testing\MyClass;
        $obj = new MyClass();
    "#;

    let class_content = r#"<?php
        namespace MyApp\Testing;

        class MyClass {}
    "#;

    let (state, temp_dir, target_uri, parser_lock) = setup_test_environment(
        main_content,
        vec![("src/testing/class.php", class_content)],
        vec![("MyApp\\Testing\\MyClass", "src/testing/class.php")],
    );

    let response =
        handle_go_to_definition(&target_uri, &Position::new(2, 19), &state, &parser_lock);

    let class_path = format!(
        "{}/{}",
        temp_dir.path().to_str().unwrap(),
        "src/testing/class.php"
    );
    let class_uri = Url::from_file_path(Path::new(&class_path)).unwrap();
    assert_definition_response(response, &class_uri, 3, 14);
}

#[test]
fn test_go_to_class_definition_on_static_method_call_statement_within_same_folder() {
    let main_content = r#"<?php
        echo MyClass::test();
    "#;

    let class_content = r#"<?php
        namespace MyNamespace;

        class MyClass {
            public static function test(): string
            {
                return "test";
            }
        }
    "#;

    let (state, temp_dir, target_uri, parser_lock) =
        setup_test_environment(main_content, vec![("class.php", class_content)], vec![]);

    let response =
        handle_go_to_definition(&target_uri, &Position::new(1, 13), &state, &parser_lock);

    let class_path = format!("{}/{}", temp_dir.path().to_str().unwrap(), "class.php");
    let class_uri = Url::from_file_path(Path::new(&class_path)).unwrap();
    assert_definition_response(response, &class_uri, 3, 14);
}

#[test]
fn test_go_to_class_definition_on_static_method_call_statement() {
    let main_content = r#"<?php
        use MyApp\Testing\MyClass;
        echo MyClass::test();
    "#;

    let class_content = r#"<?php
        namespace MyApp\Testing;

        class MyClass {
            public static function test(): string
            {
                return "test";
            }
        }
    "#;

    let (state, temp_dir, target_uri, parser_lock) = setup_test_environment(
        main_content,
        vec![("src/testing/class.php", class_content)],
        vec![("MyApp\\Testing\\MyClass", "src/testing/class.php")],
    );

    let response =
        handle_go_to_definition(&target_uri, &Position::new(2, 14), &state, &parser_lock);

    let class_path = format!(
        "{}/{}",
        temp_dir.path().to_str().unwrap(),
        "src/testing/class.php"
    );
    let class_uri = Url::from_file_path(Path::new(&class_path)).unwrap();
    assert_definition_response(response, &class_uri, 3, 14);
}

fn setup_test_environment(
    main_content: &str,
    additional_files: Vec<(&str, &str)>,
    class_mappings: Vec<(&str, &str)>,
) -> (State, TempDir, Url, RwLock<Parser>) {
    let temp_dir = TempDir::new().expect("to initialize temp dir");
    let path_str = format!("{}/{}", temp_dir.path().to_str().unwrap(), "test.php");
    let temp_dir_path = Path::new(&path_str);
    let target_uri = Url::from_file_path(temp_dir_path).unwrap();

    let mut parser = Parser::new().expect("to create a parser");
    let tree = parser.parse(main_content).expect("to parse file");

    let ast_map = DashMap::new();
    ast_map.insert(target_uri.clone(), tree);
    let document_map = DashMap::new();
    document_map.insert(target_uri.clone(), String::from(main_content));

    let class_map = DashMap::new();
    for (class_name, file_path) in class_mappings {
        class_map.insert(
            String::from(class_name),
            format!("{}/{}", temp_dir.path().to_str().unwrap(), file_path),
        );
    }

    let state = State::new(
        ast_map,
        document_map,
        RwLock::new(String::from(temp_dir_path.to_str().unwrap())),
        class_map,
    );

    prepare_php_file(temp_dir_path, main_content);

    for (file_path, file_content) in additional_files {
        let full_path = format!("{}/{}", temp_dir.path().to_str().unwrap(), file_path);

        if let Some(parent) = Path::new(&full_path).parent() {
            std::fs::create_dir_all(parent).expect("to create directory");
        }

        let file_path = Path::new(&full_path);
        prepare_php_file(file_path, file_content);
    }

    (state, temp_dir, target_uri, RwLock::new(parser))
}

fn assert_definition_response(
    response: Option<GotoDefinitionResponse>,
    expected_uri: &Url,
    expected_line: u32,
    expected_character: u32,
) {
    assert!(response.is_some());
    if let GotoDefinitionResponse::Scalar(location) = response.unwrap() {
        assert_eq!(location.uri.as_str(), expected_uri.as_str());
        assert_eq!(location.range.start.line, expected_line);
        assert_eq!(location.range.start.character, expected_character);
    } else {
        panic!("response is not a location");
    }
}

fn prepare_php_file(file_path: &Path, file_contents: &str) {
    let mut file = std::fs::File::create(file_path).expect("to create file");
    file.write_all(file_contents.as_bytes())
        .expect("to write to file");
}
