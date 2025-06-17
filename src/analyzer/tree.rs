use mago_ast::Node;
use mago_source::Source;
use mago_span::{HasPosition, HasSpan};
use tower_lsp::lsp_types::{Position, Range};
use tracing::debug;

pub fn get_node_for_position<'a>(
    node: &Node<'a>,
    source: &Source,
    current: &Position,
) -> Option<Node<'a>> {
    let range = Range {
        start: Position {
            line: (source.line_number(node.start_position().offset())) as u32,
            character: (source.column_number(node.start_position().offset())) as u32,
        },
        end: Position {
            line: (source.line_number(node.end_position().offset())) as u32,
            character: (source.column_number(node.end_position().offset())) as u32,
        },
    };

    debug!("pointer: {:?}", current);
    debug!("current node position: {:?}", range);

    if range.start.line >= current.line && range.end.line <= current.line {
        return Some(*node);
    }

    for node in node.children() {
        if let Some(n) = get_node_for_position(&node, source, current) {
            return Some(n);
        }
    }

    None
}
