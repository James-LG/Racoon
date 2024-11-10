use itertools::Itertools;
use skyscraper::xpath::{
    grammar::{DisplayFormatting, XpathItemTreeNode},
    XpathItemTree,
};

mod tests;

pub fn compare_documents(
    expected_doc: XpathItemTree,
    actual_doc: XpathItemTree,
    ignore_whitespace: bool,
) -> bool {
    let expected_root_node = expected_doc.root();
    let actual_root_node = actual_doc.root();

    compare_nodes(
        Some(expected_root_node),
        Some(actual_root_node),
        &expected_doc,
        &actual_doc,
        ignore_whitespace,
    )
}

fn compare_nodes(
    expected_node: Option<&XpathItemTreeNode>,
    actual_node: Option<&XpathItemTreeNode>,
    expected_doc: &XpathItemTree,
    actual_doc: &XpathItemTree,
    ignore_whitespace: bool,
) -> bool {
    match (expected_node, actual_node) {
        (
            Some(XpathItemTreeNode::DocumentNode(expected_document)),
            Some(XpathItemTreeNode::DocumentNode(actual_document)),
        ) => {
            // check children
            let expected_children = expected_document
                .children(&expected_doc)
                .into_iter()
                .filter(|node| {
                    if node.is_attribute_node() {
                        return false;
                    }
                    if ignore_whitespace {
                        if let XpathItemTreeNode::TextNode(text_node) = node {
                            return !text_node.content.trim().is_empty();
                        }
                    }
                    true
                });
            let actual_children =
                actual_document
                    .children(&actual_doc)
                    .into_iter()
                    .filter(|node| {
                        if node.is_attribute_node() {
                            return false;
                        }
                        if ignore_whitespace {
                            if let XpathItemTreeNode::TextNode(text_node) = node {
                                return !text_node.content.trim().is_empty();
                            }
                        }
                        true
                    });

            for cb in expected_children.zip_longest(actual_children) {
                let (expected_child, actual_child) = cb.left_and_right();

                if !compare_nodes(
                    expected_child,
                    actual_child,
                    expected_doc,
                    actual_doc,
                    ignore_whitespace,
                ) {
                    return false;
                }
            }
        }
        (
            Some(XpathItemTreeNode::ElementNode(expected_element)),
            Some(XpathItemTreeNode::ElementNode(actual_element)),
        ) => {
            // first check element names
            if expected_element.name != actual_element.name {
                print_differences(expected_node, expected_doc, actual_node, actual_doc);
                return false;
            }

            // next check attributes
            let expected_element_attributes = expected_element.attributes(expected_doc);
            let actual_element_attributes = actual_element.attributes(actual_doc);

            for ab in expected_element_attributes
                .into_iter()
                .zip_longest(actual_element_attributes)
            {
                let (expected_attribute, actual_attribute) = ab.left_and_right();

                if expected_attribute != actual_attribute {
                    print_differences(expected_node, expected_doc, actual_node, actual_doc);
                    return false;
                }
            }

            // finally check children
            let expected_children =
                expected_element
                    .children(&expected_doc)
                    .into_iter()
                    .filter(|node| {
                        if node.is_attribute_node() {
                            return false;
                        }
                        if ignore_whitespace {
                            if let XpathItemTreeNode::TextNode(text_node) = node {
                                return !text_node.content.trim().is_empty();
                            }
                        }
                        true
                    });
            let actual_children = actual_element
                .children(&actual_doc)
                .into_iter()
                .filter(|node| {
                    if node.is_attribute_node() {
                        return false;
                    }
                    if ignore_whitespace {
                        if let XpathItemTreeNode::TextNode(text_node) = node {
                            return !text_node.content.trim().is_empty();
                        }
                    }
                    true
                });

            for cb in expected_children.zip_longest(actual_children) {
                let (expected_child, actual_child) = cb.left_and_right();

                if !compare_nodes(
                    expected_child,
                    actual_child,
                    expected_doc,
                    actual_doc,
                    ignore_whitespace,
                ) {
                    return false;
                }
            }
        }
        _ => {
            if expected_node != actual_node {
                print_differences(expected_node, expected_doc, actual_node, actual_doc);
                return false;
            }
        }
    }
    true
}

pub fn print_differences(
    expected_node: Option<&XpathItemTreeNode>,
    expected_doc: &XpathItemTree,
    actual_node: Option<&XpathItemTreeNode>,
    actual_doc: &XpathItemTree,
) {
    println!(
        "---------------\nExpected document:\n{}\n---------------",
        expected_doc
    );
    println!(
        "---------------\nActual document:\n{}\n---------------",
        actual_doc
    );

    let expected_node_display = expected_node.map_or(String::new(), |n| {
        n.display(&expected_doc, DisplayFormatting::NoChildren)
    });
    println!("Expected node display:\n{}", expected_node_display);

    let actual_node_display = actual_node.map_or(String::new(), |n| {
        n.display(&actual_doc, DisplayFormatting::NoChildren)
    });
    println!("Actual node display:\n{}", actual_node_display);

    println!("Expected: {:?}", expected_node);
    println!("Actual: {:?}", actual_node);

    if let Some(expected) = expected_node {
        print_parent("Expected", expected, expected_doc);
    }

    if let Some(actual) = actual_node {
        print_parent("Actual", actual, actual_doc);
    }
}

fn print_parent(name: &str, node: &XpathItemTreeNode, doc: &XpathItemTree) {
    match node {
        XpathItemTreeNode::ElementNode(element) => {
            let parent = element.parent(doc);
            println!("{} Parent: {:?}", name, parent);
        }
        XpathItemTreeNode::TextNode(text) => {
            let parent = text.parent(doc);
            println!("{} Parent: {:?}", name, parent);
        }
        XpathItemTreeNode::AttributeNode(attribute) => {
            let parent = attribute.parent(doc);
            println!("{} Parent: {:?}", name, parent);
        }
        XpathItemTreeNode::CommentNode(comment) => {
            let parent = comment.parent(doc);
            println!("{} Parent: {:?}", name, parent);
        }
        _ => {}
    }
}
