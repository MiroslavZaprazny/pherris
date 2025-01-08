use std::{path::PathBuf, sync::RwLock};

use streaming_iterator::StreamingIterator;
use tower_lsp::lsp_types::{GotoDefinitionParams, GotoDefinitionResponse, Location, Url};
use tracing::debug;
use tree_sitter::{Node, Query, QueryCursor, Tree};

use crate::{
    analyzer::{
        parser::Parser,
        utils::{
            find_nearest_location, get_node_for_point, get_point_from_position,
            get_position_from_point, print_tree,
        },
    },
    lsp::{state::State, utils::get_variable_locations_for_query},
};

pub fn handle_go_to_definition(
    params: &GotoDefinitionParams,
    state: &State,
    parser: &RwLock<Parser>,
) -> Option<GotoDefinitionResponse> {
    let tree = state
        .ast_map
        .get(&params.text_document_position_params.text_document.uri)
        .expect("to get the tree");

    let document = state
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
            let location = find_variable_declaration(
                &current_node,
                &document,
                &params.text_document_position_params.text_document.uri,
                &tree,
            )
            .expect("to find variable declaration");

            return Some(GotoDefinitionResponse::Scalar(location));
        }
        "named_type" => {
            let location = find_class_definition(
                &current_node,
                &document,
                &tree,
                &params.text_document_position_params.text_document.uri,
                state,
                parser,
            )
            .expect("to find class definition");

            return Some(GotoDefinitionResponse::Scalar(location));
        }
        _ => return None,
    }
}

//TODO move to analyzer crate
//instead of state we should pass in the path i guess
fn find_class_definition(
    current_node: &Node,
    document: &str,
    tree: &Tree,
    current_uri: &Url,
    state: &State,
    parser: &RwLock<Parser>,
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

            let path = state.class_map.get(fqn);

            if path.is_none() {
                continue;
            }

            if fqn.ends_with(format!("\\{}", class_name).as_str()) {
                debug!("found: {}", fqn);
                let path = path.unwrap();
                let location = get_class_declaration_location(
                    &PathBuf::from(&path.to_owned()),
                    class_name,
                    parser,
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

        if let Some(location) = get_class_declaration_location(&path, class_name, parser) {
            return Some(location);
        }
    }

    None
}

//TODO move to analyzer crate
fn get_class_declaration_location(
    path: &PathBuf,
    class_name: &str,
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
