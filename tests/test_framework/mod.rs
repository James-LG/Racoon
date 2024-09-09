use itertools::Itertools;
use skyscraper::xpath::{
    grammar::{DisplayFormatting, XpathItemTreeNode},
    XpathItemTree,
};

pub fn compare_documents(
    expected_doc: XpathItemTree,
    actual_doc: XpathItemTree,
    ignore_whitespace: bool,
) -> bool {
    let expected_root_descendants = expected_doc
        .root()
        .descendants(&expected_doc)
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
    let actual_root_descendants = actual_doc.root().descendants(&actual_doc).filter(|node| {
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

    for eb in expected_root_descendants.zip_longest(actual_root_descendants) {
        let (expected_node, actual_node) = eb.left_and_right();

        println!("Expected node: {:?}", expected_node);
        println!("Actual node: {:?}", actual_node);

        if let (
            Some(XpathItemTreeNode::ElementNode(expected_element)),
            Some(XpathItemTreeNode::ElementNode(actual_element)),
        ) = (expected_node, actual_node)
        {
            // first check element names
            if expected_element.name != actual_element.name {
                print_differences(expected_node, &expected_doc, actual_node, &actual_doc);
                return false;
            }

            // next check attributes
            let expected_element_attributes = expected_element.attributes(&expected_doc);
            let actual_element_attributes = actual_element.attributes(&actual_doc);

            for ab in expected_element_attributes
                .into_iter()
                .zip_longest(actual_element_attributes)
            {
                let (expected_attribute, actual_attribute) = ab.left_and_right();

                if expected_attribute != actual_attribute {
                    print_differences(expected_node, &expected_doc, actual_node, &actual_doc);
                    return false;
                }
            }
        } else {
            if expected_node != actual_node {
                print_differences(expected_node, &expected_doc, actual_node, &actual_doc);
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
