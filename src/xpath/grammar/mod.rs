//! This module contains the grammar for the XPath language.
//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-grammar

// Helpful links:
// https://github.com/rust-bakery/nom/blob/main/doc/making_a_new_parser_from_scratch.md

pub mod data_model;
mod expressions;
mod recipes;
mod terminal_symbols;
mod types;
mod xml_names;

use std::fmt::Display;

use enum_extract_macro::EnumExtract;
pub(crate) use expressions::xpath;
pub use expressions::Xpath;

use indextree::{Arena, NodeId};

use crate::{
    html::{DocumentNode, HtmlDocument, HtmlNode},
    xpath::grammar::data_model::{
        AttributeNode, CommentNode, ElementNode, NamespaceNode, PINode, TextNode, XpathDocumentNode,
    },
};

/// Nodes that are not part of the [`XpathItemTree`].
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Hash)]
pub enum NonTreeXpathNode {
    /// An attribute node.
    AttributeNode(AttributeNode),

    /// A namespace node.
    NamespaceNode(NamespaceNode),
}

impl Display for NonTreeXpathNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NonTreeXpathNode::AttributeNode(node) => write!(f, "{}", node),
            NonTreeXpathNode::NamespaceNode(node) => write!(f, "{}", node),
        }
    }
}

/// Nodes that are part of the [`XpathItemTree`].
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Hash, EnumExtract)]
pub enum XpathItemTreeNodeData {
    /// The root node of the document.
    DocumentNode(XpathDocumentNode),

    /// An element node.
    ///
    /// HTML tags are represented as element nodes.
    ElementNode(ElementNode),

    /// A processing instruction node.
    PINode(PINode),

    /// A comment node.
    CommentNode(CommentNode),

    /// A text node.
    TextNode(TextNode),
}

/// A node in the [`XpathItemTree`].
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Hash)]
pub struct XpathItemTreeNode<'a> {
    id: NodeId,

    /// The data associated with this node.
    pub data: &'a XpathItemTreeNodeData,
}

impl Display for XpathItemTreeNode<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.data {
            XpathItemTreeNodeData::DocumentNode(node) => write!(f, "{}", node),
            XpathItemTreeNodeData::ElementNode(node) => write!(f, "{}", node),
            XpathItemTreeNodeData::PINode(node) => write!(f, "{}", node),
            XpathItemTreeNodeData::CommentNode(node) => write!(f, "{}", node),
            XpathItemTreeNodeData::TextNode(node) => write!(f, "{}", node),
        }
    }
}

impl<'a> XpathItemTreeNode<'a> {
    /// Get all the child nodes of this node.
    ///
    /// # Arguments
    ///
    /// * `tree` - The tree that this node is a part of.
    ///
    /// # Returns
    ///
    /// An iterator over the child nodes of this node.
    pub fn children(&self, tree: &'a XpathItemTree) -> impl Iterator<Item = XpathItemTreeNode<'a>> {
        self.id
            .children(&tree.arena)
            .into_iter()
            .map(move |id| tree.get(id))
    }

    /// Get the parent node of this node.
    ///
    /// # Arguments
    ///
    /// * `tree` - The tree that this node is a part of.
    ///
    /// # Returns
    ///
    /// The parent node of this node, or `None` if this node is the root node.
    pub fn parent(&self, tree: &'a XpathItemTree) -> Option<XpathItemTreeNode<'a>> {
        tree.get_index_node(self.id).parent().map(|id| tree.get(id))
    }

    /// Get all text contained in this node and its descendants.
    ///
    /// Use [`XpathItemTreeNode::text`] to get _only_ text contained directly in this node.
    ///
    /// # Arguments
    ///
    /// * `tree` - The tree that this node is a part of.
    ///
    /// # Returns
    ///
    /// A string of all text contained in this node and its descendants.
    pub fn all_text(&self, tree: &'a XpathItemTree) -> String {
        self.text_internal(tree, true)
    }

    /// Get text directly contained in this node.
    ///
    /// Use [`XpathItemTreeNode::all_text`] to get all text _including_ text in descendant nodes.
    ///
    /// # Arguments
    ///
    /// * `tree` - The tree that this node is a part of.
    ///
    /// # Returns
    ///
    /// A string of all text contained in this node.
    pub fn text(&self, tree: &'a XpathItemTree) -> String {
        self.text_internal(tree, false)
    }

    fn text_internal(&self, tree: &'a XpathItemTree, recurse: bool) -> String {
        fn get_all_text_nodes(
            tree: &XpathItemTree,
            node: &XpathItemTreeNode,
            recurse: bool,
        ) -> Vec<TextNode> {
            node
                // Get all children of the given node.
                .children(tree)
                // Combine all the direct and indirect children into a Vec.
                .fold(Vec::new(), |mut v, child| {
                    // If this child is a text node, push it to the Vec.
                    if let XpathItemTreeNodeData::TextNode(text) = child.data {
                        v.push(text.clone());
                    }
                    // Otherwise, if this is a recursive search, get all the text nodes descending from this child.
                    else if recurse {
                        v.extend(get_all_text_nodes(tree, &child, recurse));
                    }
                    v
                })
        }

        let strings: Vec<String> =
            // Get all children.
            get_all_text_nodes(tree, self, recurse)
            .into_iter()
            // Filter out all whitespace-only text nodes
            .filter_map(|x| {
                if x.only_whitespace {
                    None
                } else {
                    Some(x.content)
                }
            })
            .collect();

        // Merge all text into a single string.
        // Space delimited.
        let text = strings.join(" ");

        text
    }
}

/// A tree of [`XpathItemTreeNode`]s.
pub struct XpathItemTree {
    /// The index tree that stores the nodes.
    arena: Arena<XpathItemTreeNodeData>,

    /// The root node of the document.
    root_node: NodeId,
}

impl XpathItemTree {
    fn get_index_node(&self, id: NodeId) -> &indextree::Node<XpathItemTreeNodeData> {
        self.arena
            .get(id)
            .expect("xpath item node missing from tree")
    }

    fn get(&self, id: NodeId) -> XpathItemTreeNode<'_> {
        let indextree_node = self.get_index_node(id);

        let data = indextree_node.get();
        XpathItemTreeNode { id, data }
    }

    fn root(&self) -> XpathItemTreeNode<'_> {
        self.get(self.root_node)
    }
}

impl From<&HtmlDocument> for XpathItemTree {
    fn from(html_document: &HtmlDocument) -> Self {
        fn internal_from(
            current_html_node: &DocumentNode,
            html_document: &HtmlDocument,
            item_arena: &mut Arena<XpathItemTreeNodeData>,
        ) -> NodeId {
            let html_node = html_document
                .get_html_node(&current_html_node)
                .expect("html document missing expected node");

            let root_item = match html_node {
                HtmlNode::Tag(tag) => {
                    let attributes = tag
                        .attributes
                        .iter()
                        .map(|a| AttributeNode {
                            name: a.0.to_string(),
                            value: a.1.to_string(),
                        })
                        .collect();
                    XpathItemTreeNodeData::ElementNode(ElementNode {
                        name: tag.name.to_string(),
                        attributes,
                    })
                }
                HtmlNode::Text(text) => XpathItemTreeNodeData::TextNode(TextNode {
                    content: text.value.to_string(),
                    only_whitespace: text.only_whitespace,
                }),
            };

            let root_item_id = item_arena.new_node(root_item);

            for child in current_html_node.children(&html_document) {
                let child_node = internal_from(&child, html_document, item_arena);
                root_item_id.append(child_node, item_arena);
            }

            root_item_id
        }

        let mut item_arena = Arena::<XpathItemTreeNodeData>::new();
        let root_node_id =
            item_arena.new_node(XpathItemTreeNodeData::DocumentNode(XpathDocumentNode {}));
        let first_child = internal_from(&html_document.root_node, &html_document, &mut item_arena);
        root_node_id.append(first_child, &mut item_arena);

        XpathItemTree {
            arena: item_arena,
            root_node: root_node_id,
        }
    }
}
