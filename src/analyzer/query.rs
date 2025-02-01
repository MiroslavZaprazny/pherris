use tree_sitter::{Query, QueryError};
use tree_sitter_php::LANGUAGE_PHP;

pub fn class_decleration_query() -> Result<Query, QueryError> {
    Query::new(
        &LANGUAGE_PHP.into(),
        "(class_declaration
            (name) @class_name)",
    )
}

pub fn variable_decleration_query() -> Result<Query, QueryError> {
    Query::new(
        &LANGUAGE_PHP.into(),
        "(assignment_expression left: (variable_name) @declaration)",
    )
}

pub fn namespace_use_query() -> Result<Query, QueryError> {
    Query::new(
        &LANGUAGE_PHP.into(),
        "(namespace_use_clause
            (qualified_name) @namespace)",
    )
}

pub fn param_query() -> Result<Query, QueryError> {
    Query::new(
        &LANGUAGE_PHP.into(),
        "(simple_parameter (variable_name) @declaration)",
    )
}
