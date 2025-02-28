use tree_sitter::{Query, QueryError};
use tree_sitter_php::LANGUAGE_PHP;

pub fn named_type_declaration_query() -> Result<Query, QueryError> {
    Query::new(
        &LANGUAGE_PHP.into(),
        "(class_declaration
            (name) @class_name)

        (interface_declaration
            (name) @interface)

        (enum_declaration
            (name) @enum)

        (trait_declaration
            (name) @trait)
        ",
    )
}

pub fn variable_declaration_query() -> Result<Query, QueryError> {
    Query::new(
        &LANGUAGE_PHP.into(),
        "(assignment_expression 
            left: (variable_name) @variable_assignemnt)

        (foreach_statement 
            (variable_name) @foreach_declaration)

        (foreach_statement 
            (pair (variable_name) @foreach_pair_declaration))

        (simple_parameter
            (variable_name) @parameter_declaration)
        ",
    )
}

pub fn namespace_use_query() -> Result<Query, QueryError> {
    Query::new(
        &LANGUAGE_PHP.into(),
        "(namespace_use_clause
            (qualified_name) @namespace)",
    )
}

pub fn error_query() -> Result<Query, QueryError> {
    Query::new(&LANGUAGE_PHP.into(), "(ERROR) @general_error")
}
