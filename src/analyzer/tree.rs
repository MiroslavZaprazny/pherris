use mago_ast::{Node, NodeKind};
use mago_source::Source;
use mago_span::{HasPosition, HasSpan};
use tower_lsp::lsp_types::{Position, Range};

pub fn get_node_for_position<'a>(
    node: &Node<'a>,
    source: &Source,
    needle: &Position,
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

    if (range.start.line..=range.end.line).contains(&needle.line)
        && (range.start.character..=range.end.character).contains(&needle.character)
    {
        // maybe we shouldnt skip them but actually parse them later, but we dont really care about
        // these nodes for now
        if node.kind() != NodeKind::FunctionLikeParameterList
            && node.kind() != NodeKind::FunctionLikeParameter
            && node.kind() != NodeKind::FunctionLikeReturnTypeHint
            && node.kind() != NodeKind::Implements
            && node.kind() != NodeKind::UseItems
            && node.kind() != NodeKind::UseItemSequence
        {
            return Some(*node);
        }
    }

    for node in node.children() {
        if let Some(n) = get_node_for_position(&node, source, needle) {
            return Some(n);
        }
    }

    None
}
