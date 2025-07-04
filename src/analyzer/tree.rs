use mago_ast::Node;
use mago_source::Source;
use mago_span::{HasPosition, HasSpan};
use tower_lsp::lsp_types::{Position, Range};

pub fn get_node_for_position<'a>(
    node: &Node<'a>,
    source: &Source,
    needle: &Position,
) -> Option<Node<'a>> {
    let range = get_range(node, source);

    if (range.start.line..=range.end.line).contains(&needle.line)
        && (range.start.character..=range.end.character).contains(&needle.character)
    {
        return Some(*node);
    }

    for node in node.children() {
        if let Some(n) = get_node_for_position(&node, source, needle) {
            return Some(n);
        }
    }

    None
}

//move somwhere else?
pub fn get_range(node: impl HasSpan, source: &Source) -> Range {
    Range {
        start: Position {
            line: (source.line_number(node.start_position().offset())) as u32,
            character: (source.column_number(node.start_position().offset())) as u32,
        },
        end: Position {
            line: (source.line_number(node.end_position().offset())) as u32,
            character: (source.column_number(node.end_position().offset())) as u32,
        },
    }
}

//move somwhere else?
pub fn range_contains_position(range: &Range, position: &Position) -> bool {
    (range.start.line..=range.end.line).contains(&position.line)
        && (range.start.character..=range.end.character).contains(&position.character)
}
//move somwhere else?
pub fn get_node_name(document: &str, node: impl HasSpan) -> String {
    document[node.start_position().offset()..node.end_position().offset()].to_string()
}
