use tower_lsp::lsp_types::{Location, Position};
use tracing::debug;
use tree_sitter::{Node, Point, Tree};

pub fn get_point_from_position(position: &Position) -> Point {
    Point {
        row: position.line as usize,
        column: position.character as usize,
    }
}

pub fn get_position_from_point(point: &Point) -> Position {
    Position {
        line: point.row as u32,
        character: point.column as u32,
    }
}

pub fn get_node_for_point(tree: &Tree, point: Point) -> Option<Node> {
    tree.root_node().descendant_for_point_range(point, point)
}

pub fn find_nearest_location(a: Position, b: Vec<Location>) -> Option<Location> {
    b.into_iter()
        .min_by_key(|location| a.line.abs_diff(location.range.start.line))
}

pub fn print_tree(tree: &Tree) {
    let root_node = tree.root_node();

    print_node(root_node, 0);
}

pub fn print_node(node: Node, depth: usize) {
    let indent = "  ".repeat(depth);

    debug!(
        "{}{} [{}, {}] - [{}, {}]",
        indent,
        node.kind(),
        node.start_position().row,
        node.start_position().column,
        node.end_position().row,
        node.end_position().column
    );

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        print_node(child, depth + 1);
    }
}
