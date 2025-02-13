use tree_sitter::{Query, QueryError};
use tree_sitter_php::LANGUAGE_PHP;

pub fn class_declaration_query() -> Result<Query, QueryError> {
    Query::new(
        &LANGUAGE_PHP.into(),
        "(class_declaration
            (name) @class_name)",
    )
}

pub fn interface_declaration_query() -> Result<Query, QueryError> {
    Query::new(
        &LANGUAGE_PHP.into(),
        "(interface_declaration
            (name) @class_name)",
    )
}

pub fn enum_declaration_query() -> Result<Query, QueryError> {
    Query::new(
        &LANGUAGE_PHP.into(),
        "(enum_declaration
            (name) @class_name)",
    )
}

pub fn trait_declaration_query() -> Result<Query, QueryError> {
    Query::new(
        &LANGUAGE_PHP.into(),
        "(trait_declaration
            (name) @class_name)",
    )
}

pub fn variable_declaration_query() -> Result<Query, QueryError> {
    Query::new(
        &LANGUAGE_PHP.into(),
        "(assignment_expression left: (variable_name) @declaration)",
    )
}

pub fn variable_declaration_foreach_query() -> Result<Query, QueryError> {
    Query::new(
        &LANGUAGE_PHP.into(),
        "(foreach_statement (variable_name) @name)",
    )
}

pub fn variable_declaration_foreach_pair_query() -> Result<Query, QueryError> {
    Query::new(
        &LANGUAGE_PHP.into(),
        "(foreach_statement (pair (variable_name) @name))",
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
