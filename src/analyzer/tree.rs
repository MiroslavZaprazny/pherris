use mago_ast::{Node, NodeKind};
use mago_source::Source;
use mago_span::{HasPosition, HasSpan};
use tower_lsp::lsp_types::{Position, Range};

pub fn get_node_for_position<'a>(
    node: &Node<'a>,
    contents: &str,
    source: &Source,
    current: &Position,
    target_node_kind: NodeKind,
) -> Option<Node<'a>> {
    if node.kind() == target_node_kind {
        // let text = &contents[node.start_position().offset()..node.end_position().offset()];
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

        if range.start.line <= current.line && range.end.line >= current.line {
            return Some(*node);
        }

        // let name = names.get(&node.position());
        // debug!("text: {:?}", text);
        // debug!("range: {:?}", range);
    }

    for node in node.children() {
        if let Some(n) = get_node_for_position(&node, contents, source, current, target_node_kind) {
            return Some(n);
        }
    }

    None
}
