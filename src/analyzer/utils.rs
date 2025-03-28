use mago_ast::Program;
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
