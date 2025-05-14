use dashmap::DashMap;
use mago_interner::ThreadedInterner;
use mago_lexer::input::Input;
use mago_parser::parse;
use mago_source::{Source, SourceIdentifier};
use pherris::analyzer::parser::Parser;
use pherris::handlers::request::handle_go_to_definition;
use pherris::lsp::state::State;
use std::io::Write;
use std::path::Path;
use std::sync::RwLock;
use tempfile::TempDir;
use tower_lsp::lsp_types::{GotoDefinitionResponse, Position, Url};

#[test]
fn test_find_variable_declaration() {
    let main_content = r#"<?php
        $variable = 0;
        echo $variable;
    "#;

    let (state, _temp_dir, target_uri, parser_lock) =
        setup_test_environment(main_content, vec![], vec![]);

    let response =
        handle_go_to_definition(&target_uri, &Position::new(2, 15), &state, &parser_lock);

    assert_definition_response(response, &target_uri, 1, 8);
}

#[test]
fn test_find_shadowed_variable_declaration() {
    let main_content = r#"<?php
        $variable = 0;
        $variable = 1;
        echo $variable;
    "#;

    let (state, _temp_dir, target_uri, parser_lock) =
        setup_test_environment(main_content, vec![], vec![]);

    let response =
        handle_go_to_definition(&target_uri, &Position::new(2, 15), &state, &parser_lock);

    assert_definition_response(response, &target_uri, 2, 8);
}

#[test]
fn test_find_variable_definition_in_array_function() {
    let main_content = r#"<?php
        array_map(fn ($test) => $test, []);
    "#;

    let (state, _temp_dir, target_uri, parser_lock) =
        setup_test_environment(main_content, vec![], vec![]);

    let response =
        handle_go_to_definition(&target_uri, &Position::new(1, 35), &state, &parser_lock);

    assert_definition_response(response, &target_uri, 1, 22);
}

#[test]
fn test_find_variable_definition_in_function() {
    let main_content = r#"<?php
        function ($test) {
            echo $test;
    "#;

    let (state, _temp_dir, target_uri, parser_lock) =
        setup_test_environment(main_content, vec![], vec![]);

    let response =
        handle_go_to_definition(&target_uri, &Position::new(2, 20), &state, &parser_lock);

    assert_definition_response(response, &target_uri, 1, 18);
}

#[test]
fn test_find_variable_definition_in_foreach() {
    let main_content = r#"<?php
        $list = [];
        foreach ($list as $element) {
            echo $element;
        }
    "#;

    let (state, _temp_dir, target_uri, parser_lock) =
        setup_test_environment(main_content, vec![], vec![]);

    let response =
        handle_go_to_definition(&target_uri, &Position::new(3, 18), &state, &parser_lock);

    assert_definition_response(response, &target_uri, 2, 26);
}

#[test]
fn test_find_variable_definition_in_foreach_pair() {
    let main_content = r#"<?php
        $list = [];
        foreach ($list as $key => $val) {
            echo $val;
            echo $key;
        }
    "#;

    let (state, _temp_dir, target_uri, parser_lock) =
        setup_test_environment(main_content, vec![], vec![]);

    let response =
        handle_go_to_definition(&target_uri, &Position::new(3, 18), &state, &parser_lock);

    assert_definition_response(response, &target_uri, 2, 34);

    let response =
        handle_go_to_definition(&target_uri, &Position::new(4, 18), &state, &parser_lock);

    assert_definition_response(response, &target_uri, 2, 26);
}

#[test]
fn test_find_class_definition_in_same_folder() {
    let main_content = r#"<?php
        function (MyClass $test) {
            echo $test;
        }
    "#;

    let class_content = r#"<?php
        namespace MyApp\Testing;

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
fn test_find_class_definition_in_same_folder_using_class_name_as_filename() {
    let main_content = r#"<?php
        function (MyClass $test) {
            echo $test;
        }
    "#;

    let class_content = r#"<?php
        namespace MyApp\Testing;

        class MyClass {}
    "#;

    let (state, temp_dir, target_uri, parser_lock) =
        setup_test_environment(main_content, vec![("MyClass.php", class_content)], vec![]);

    let response =
        handle_go_to_definition(&target_uri, &Position::new(1, 19), &state, &parser_lock);

    let class_path = format!("{}/{}", temp_dir.path().to_str().unwrap(), "MyClass.php");
    let class_uri = Url::from_file_path(Path::new(&class_path)).unwrap();

    assert_definition_response(response, &class_uri, 3, 14);
}

#[test]
fn test_find_class_definition_in_different_folder() {
    let main_content = r#"<?php
        use MyApp\Testing\MyClass;
        function (MyClass $test) {
            echo $test;
        }
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
fn test_find_enum_definition_in_same_folder() {
    let main_content = r#"<?php
        function (MyEnum $test) {
            echo $test;
        }
    "#;

    let class_content = r#"<?php
        namespace MyApp\Testing;

        enum MyEnum {}
    "#;

    let (state, temp_dir, target_uri, parser_lock) =
        setup_test_environment(main_content, vec![("class.php", class_content)], vec![]);

    let response =
        handle_go_to_definition(&target_uri, &Position::new(1, 19), &state, &parser_lock);

    let class_path = format!("{}/{}", temp_dir.path().to_str().unwrap(), "class.php");
    let class_uri = Url::from_file_path(Path::new(&class_path)).unwrap();

    assert_definition_response(response, &class_uri, 3, 13);
}

#[test]
fn test_find_enum_definition_in_different_folder() {
    let main_content = r#"<?php
        use MyApp\Testing\MyEnum;
        function (MyEnum $test) {
            echo $test;
        }
    "#;

    let class_content = r#"<?php
        namespace MyApp\Testing;

        enum MyEnum {}
    "#;

    let (state, temp_dir, target_uri, parser_lock) = setup_test_environment(
        main_content,
        vec![("src/testing/class.php", class_content)],
        vec![("MyApp\\Testing\\MyEnum", "src/testing/class.php")],
    );

    let response =
        handle_go_to_definition(&target_uri, &Position::new(2, 19), &state, &parser_lock);

    let class_path = format!(
        "{}/{}",
        temp_dir.path().to_str().unwrap(),
        "src/testing/class.php"
    );
    let class_uri = Url::from_file_path(Path::new(&class_path)).unwrap();

    assert_definition_response(response, &class_uri, 3, 13);
}

#[test]
fn test_find_interface_definition_in_same_folder() {
    let main_content = r#"<?php
        function (MyInterface $test) {
            echo $test;
        }
    "#;

    let class_content = r#"<?php
        namespace MyApp\Testing;

        interface MyInterface {}
    "#;

    let (state, temp_dir, target_uri, parser_lock) =
        setup_test_environment(main_content, vec![("class.php", class_content)], vec![]);

    let response =
        handle_go_to_definition(&target_uri, &Position::new(1, 19), &state, &parser_lock);

    let class_path = format!("{}/{}", temp_dir.path().to_str().unwrap(), "class.php");
    let class_uri = Url::from_file_path(Path::new(&class_path)).unwrap();

    assert_definition_response(response, &class_uri, 3, 18);
}

#[test]
fn test_find_interface_definition_in_different_folder() {
    let main_content = r#"<?php
        use MyApp\Testing\MyInterface;
        function (MyInterface $test) {
            echo $test;
        }
    "#;

    let class_content = r#"<?php
        namespace MyApp\Testing;

        interface MyInterface {}
    "#;

    let (state, temp_dir, target_uri, parser_lock) = setup_test_environment(
        main_content,
        vec![("src/testing/class.php", class_content)],
        vec![("MyApp\\Testing\\MyInterface", "src/testing/class.php")],
    );

    let response =
        handle_go_to_definition(&target_uri, &Position::new(2, 19), &state, &parser_lock);

    let class_path = format!(
        "{}/{}",
        temp_dir.path().to_str().unwrap(),
        "src/testing/class.php"
    );
    let class_uri = Url::from_file_path(Path::new(&class_path)).unwrap();

    assert_definition_response(response, &class_uri, 3, 18);
}

#[test]
fn test_go_to_class_definition_on_use_statement() {
    let main_content = r#"<?php
        use MyApp\Testing\MyClass;
        function (MyClass $test) {
            echo $test;
        }
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
        handle_go_to_definition(&target_uri, &Position::new(1, 26), &state, &parser_lock);

    let class_path = format!(
        "{}/{}",
        temp_dir.path().to_str().unwrap(),
        "src/testing/class.php"
    );
    let class_uri = Url::from_file_path(Path::new(&class_path)).unwrap();

    assert_definition_response(response, &class_uri, 3, 14);
}

#[test]
fn test_go_to_class_definition_on_constant_access_expression_within_same_folder() {
    let main_content = r#"<?php
        echo MyClass::class;
    "#;

    let class_content = r#"<?php
        namespace MyApp\Testing;

        class MyClass {}
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
fn test_go_to_class_definition_on_constant_access_expression() {
    let main_content = r#"<?php
        use MyApp\Testing\MyClass;
        echo MyClass::class;
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
        handle_go_to_definition(&target_uri, &Position::new(2, 13), &state, &parser_lock);

    let class_path = format!(
        "{}/{}",
        temp_dir.path().to_str().unwrap(),
        "src/testing/class.php"
    );
    let class_uri = Url::from_file_path(Path::new(&class_path)).unwrap();

    assert_definition_response(response, &class_uri, 3, 14);
}

#[test]
fn test_go_to_class_definition_on_inherited_classes_within_same_folder() {
    let main_content = r#"<?php
        class MyClass extends BaseClass {}
    "#;

    let class_content = r#"<?php
        namespace MyApp\Testing;

        class BaseClass {}
    "#;

    let (state, temp_dir, target_uri, parser_lock) =
        setup_test_environment(main_content, vec![("class.php", class_content)], vec![]);

    let response =
        handle_go_to_definition(&target_uri, &Position::new(1, 30), &state, &parser_lock);

    let class_path = format!("{}/{}", temp_dir.path().to_str().unwrap(), "class.php");
    let class_uri = Url::from_file_path(Path::new(&class_path)).unwrap();

    assert_definition_response(response, &class_uri, 3, 14);
}

#[test]
fn test_go_to_class_definition_on_inherited_classes() {
    let main_content = r#"<?php
        use MyApp\Testing\BaseClass;
        class MyClass extends BaseClass {}
    "#;

    let class_content = r#"<?php
        namespace MyApp\Testing;

        class BaseClass {}
    "#;

    let (state, temp_dir, target_uri, parser_lock) = setup_test_environment(
        main_content,
        vec![("src/testing/class.php", class_content)],
        vec![("MyApp\\Testing\\BaseClass", "src/testing/class.php")],
    );

    let response =
        handle_go_to_definition(&target_uri, &Position::new(2, 30), &state, &parser_lock);

    let class_path = format!(
        "{}/{}",
        temp_dir.path().to_str().unwrap(),
        "src/testing/class.php"
    );
    let class_uri = Url::from_file_path(Path::new(&class_path)).unwrap();

    assert_definition_response(response, &class_uri, 3, 14);
}

#[test]
fn test_go_to_interface_definition_on_implements_statement_within_same_folder() {
    let main_content = r#"<?php
        class MyClass implements SomeInterface {}
    "#;

    let class_content = r#"<?php
        namespace MyApp\Testing;

        interface SomeInterface {}
    "#;

    let (state, temp_dir, target_uri, parser_lock) =
        setup_test_environment(main_content, vec![("interface.php", class_content)], vec![]);

    let response =
        handle_go_to_definition(&target_uri, &Position::new(1, 37), &state, &parser_lock);

    let class_path = format!("{}/{}", temp_dir.path().to_str().unwrap(), "interface.php");
    let class_uri = Url::from_file_path(Path::new(&class_path)).unwrap();
    assert_definition_response(response, &class_uri, 3, 18);
}

#[test]
fn test_go_to_interface_definition_on_implements_statement() {
    let main_content = r#"<?php
        use MyApp\Testing\SomeInterface;
        class MyClass implements SomeInterface {}
    "#;

    let class_content = r#"<?php
        namespace MyApp\Testing;

        interface SomeInterface {}
    "#;

    let (state, temp_dir, target_uri, parser_lock) = setup_test_environment(
        main_content,
        vec![("src/testing/interface.php", class_content)],
        vec![("MyApp\\Testing\\SomeInterface", "src/testing/interface.php")],
    );

    let response =
        handle_go_to_definition(&target_uri, &Position::new(2, 37), &state, &parser_lock);

    let class_path = format!(
        "{}/{}",
        temp_dir.path().to_str().unwrap(),
        "src/testing/interface.php"
    );
    let class_uri = Url::from_file_path(Path::new(&class_path)).unwrap();
    assert_definition_response(response, &class_uri, 3, 18);
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

    let interner = ThreadedInterner::new();
    let source_id = SourceIdentifier::dummy();
    let input = Input::new(source_id, main_content.as_bytes());
    let (program, _) = parse(&interner, input);
    let document_program = DashMap::new();
    document_program.insert(target_uri.clone(), program);

    let state = State::new(
        document_program,
        document_map,
        RwLock::new(String::from(temp_dir_path.to_str().unwrap())),
        class_map,
        ast_map,
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
