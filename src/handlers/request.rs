use std::{path::Path, sync::RwLock};

use streaming_iterator::StreamingIterator;
use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Position, Url};
use tree_sitter::{Node, Query, QueryCursor, Tree};

use crate::{
    analyzer::{
        parser::Parser,
        query::{named_type_declaration_query, namespace_use_query, variable_declaration_query},
        utils::{
            find_nearest_location, get_node_for_point, get_point_from_position,
            get_position_from_point,
        },
    },
    lsp::state::State,
};

pub fn handle_go_to_definition(
    uri: &Url,
    position: &Position,
    state: &State,
    parser: &RwLock<Parser>,
) -> Option<GotoDefinitionResponse> {
    let tree = state.ast_map.get(uri).expect("to get the tree");

    let document = state.document_map.get(uri).expect("to get the document");

    let current_point = get_point_from_position(position);
    let current_node = get_node_for_point(&tree, current_point).expect("to get node");

    let parent = current_node
        .parent()
        .expect("to get parent of current node");

    match parent.kind() {
        "variable_name" | "member_access_expression" => {
            let location = find_variable_declaration(&current_node, &document, uri, &tree)
                .expect("to find variable declaration");

            Some(GotoDefinitionResponse::Scalar(location))
        }
        "named_type"
        | "use_declaration"
        | "qualified_name"
        | "class_constant_access_expression"
        | "base_clause"
        | "class_interface_clause"
        | "object_creation_expression"
        | "scoped_call_expression" => {
            let location =
                find_named_type_definition(&current_node, &document, &tree, uri, state, parser)
                    .expect("to find named type definition");

            Some(GotoDefinitionResponse::Scalar(location))
        }
        _ => None,
    }
}

//TODO move to analyzer crate
//instead of state we should pass in the path i guess
fn find_named_type_definition(
    current_node: &Node,
    document: &str,
    tree: &Tree,
    current_uri: &Url,
    state: &State,
    parser: &RwLock<Parser>,
) -> Option<Location> {
    let named_type_name = current_node
        .utf8_text(document.as_bytes())
        .expect("to get class name");

    let query = namespace_use_query().expect("to create query");
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), document.as_bytes());

    let file_path = current_uri.to_file_path().unwrap();
    let current_dir = file_path.parent().unwrap();

    while let Some(match_) = matches.next() {
        for capture in match_.captures {
            let fqn = capture
                .node
                .utf8_text(document.as_bytes())
                .expect("to get use statement");

            let path = state.class_map.get(fqn);

            if path.is_none() {
                continue;
            }

            if fqn.ends_with(format!("\\{}", named_type_name).as_str()) {
                let path = path.unwrap();

                if let Some(location) = get_named_type_declaration_location(
                    Path::new(path.as_str()),
                    named_type_name,
                    parser,
                ) {
                    return Some(location);
                }
            }
        }
    }

    // first try to check the current_dir/class_name.php
    let str_path = &format!("{}/{}.php", current_dir.to_str().unwrap(), named_type_name);
    let path = Path::new(str_path);
    if path.exists() {
        if let Some(location) = get_named_type_declaration_location(path, named_type_name, parser) {
            return Some(location);
        }
    }

    //if there is no use statement try searching the current directory for the class
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

        if let Some(location) = get_named_type_declaration_location(&path, named_type_name, parser)
        {
            return Some(location);
        }
    }

    None
}

//TODO move to analyzer crate
fn get_named_type_declaration_location(
    path: &Path,
    name: &str,
    parser: &RwLock<Parser>,
) -> Option<Location> {
    if path.is_dir() {
        return None;
    }

    let content = std::fs::read_to_string(path).expect("to read destination file");

    let tree = parser
        .write()
        .unwrap()
        .parse(content.clone())
        .expect("to parse file");

    if let Some(location) = capture_named_type_location(
        &named_type_declaration_query().expect("to create query"),
        name,
        &tree,
        content.as_bytes(),
        path,
    ) {
        return Some(location);
    }

    None
}

fn capture_named_type_location(
    query: &Query,
    name: &str,
    tree: &Tree,
    content: &[u8],
    path: &Path,
) -> Option<Location> {
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(query, tree.root_node(), content);

    while let Some(match_) = matches.next() {
        for capture in match_.captures {
            let node = capture.node;
            let node_text = node.utf8_text(content).expect("to get class name");
            if node_text == name {
                return Some(Location::new(
                    Url::from_file_path(path).unwrap(),
                    tower_lsp::lsp_types::Range::new(
                        get_position_from_point(&node.start_position()),
                        get_position_from_point(&node.end_position()),
                    ),
                ));
            }
        }
    }

    None
}

fn find_variable_declaration(
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

    let var_declare_query =
        variable_declaration_query().expect("to create variable declaration query");

    let var_name = current_node
        .utf8_text(document.as_bytes())
        .expect("to get current variable name");

    let locations =
        match_variable_locations_for_query(var_name, &var_declare_query, tree, document, uri);

    find_nearest_location(
        get_position_from_point(&current_node.start_position()),
        locations,
    )
}

fn match_variable_locations_for_query(
    var_name: &str,
    query: &Query,
    tree: &Tree,
    document: &str,
    uri: &Url,
) -> Vec<Location> {
    let mut out = Vec::new();
    let mut cursor = QueryCursor::new();

    let mut matches = cursor.matches(query, tree.root_node(), document.as_bytes());

    while let Some(match_) = matches.next() {
        for capture in match_.captures {
            let declare_var_name = capture
                .node
                .utf8_text(document.as_bytes())
                .expect("a text")
                .trim_start_matches('$');

            if declare_var_name == var_name {
                let range = tower_lsp::lsp_types::Range::new(
                    get_position_from_point(&capture.node.start_position()),
                    get_position_from_point(&capture.node.end_position()),
                );

                out.push(Location::new(uri.clone(), range))
            }
        }
    }

    out
}
