mod dimensions;
mod edge_sizes;
mod layout_box;
mod rect;

pub use dimensions::Dimensions;
pub use edge_sizes::EdgeSizes;
pub use layout_box::LayoutBox;
pub use rect::Rect;

use crate::style::StyledNode;

pub enum BoxType<'a> {
    BlockNode(&'a StyledNode),
    InlineNode(&'a StyledNode),
    AnonymousBlock,
}

pub enum Display {
    Inline,
    Block,
    None,
}

/// Transform a style tree into a layout tree.
pub fn layout_tree<'a>(node: &'a StyledNode, mut containing_block: Dimensions) -> LayoutBox<'a> {
    // The layout algorithm expects the container height to start at 0.
    // TODO: Save the initial containing block height, for calculating percent heights.
    containing_block.content.height = 0.0;

    let mut root_box = build_layout_tree(node);
    root_box.layout(containing_block);
    root_box
}

// Build the tree of LayoutBoxes, but don't perform any layout calculations yet.
fn build_layout_tree<'a>(style_node: &'a StyledNode) -> LayoutBox<'a> {
    // Create the root box.
    let mut root = LayoutBox::new(match style_node.display() {
        Display::Block => BoxType::BlockNode(style_node),
        Display::Inline => BoxType::InlineNode(style_node),
        Display::None => panic!("Root node has display: none."),
    });

    // Create the descendant boxes.
    for child in &style_node.children {
        match child.display() {
            Display::Block => root.children.push(build_layout_tree(child)),
            Display::Inline => root
                .get_inline_container()
                .children
                .push(build_layout_tree(child)),
            Display::None => {} // Skip nodes with `display: none;`
        }
    }

    root
}
