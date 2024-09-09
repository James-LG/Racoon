//! This module contains the grammar for the XPath language.
//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-grammar>

// Helpful links:
// https://github.com/rust-bakery/nom/blob/main/doc/making_a_new_parser_from_scratch.md

pub mod data_model;
mod expressions;
mod recipes;
mod terminal_symbols;
mod types;
mod whitespace_recipes;
mod xml_names;

use std::{fmt::Display, iter};

use enum_extract_macro::EnumExtract;
pub(crate) use expressions::xpath;
pub use expressions::Xpath;

use indextree::{Arena, NodeId};

use crate::{
    html::{DocumentNode, HtmlDocument, HtmlNode},
    xpath::grammar::data_model::{
        AttributeNode, CommentNode, ElementNode, PINode, TextNode, XpathDocumentNode,
    },
};

/// Nodes that are part of the [`XpathItemTree`].
#[derive(PartialEq, Eq, Debug, Hash, EnumExtract, Clone)]
pub enum XpathItemTreeNode {
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

    /// An attribute node.
    AttributeNode(AttributeNode),
}

impl XpathItemTreeNode {
    /// Get all children of the document.
    ///
    /// # Arguments
    ///
    /// * `tree` - The tree containing the document.
    ///
    /// # Returns
    ///
    /// A vector of all children of the document.
    pub fn children<'tree>(&self, tree: &'tree XpathItemTree) -> Vec<&'tree XpathItemTreeNode> {
        match self {
            XpathItemTreeNode::DocumentNode(node) => node.children(tree),
            XpathItemTreeNode::ElementNode(node) => node.children(tree).collect(),
            XpathItemTreeNode::PINode(_) => vec![],
            XpathItemTreeNode::CommentNode(_) => vec![],
            XpathItemTreeNode::TextNode(_) => vec![],
            XpathItemTreeNode::AttributeNode(_) => vec![],
        }
    }

    /// Get all descendants of the element.
    ///
    /// # Arguments
    ///
    /// * `tree` - The tree containing the element.
    ///
    /// # Returns
    ///
    /// An iterator over all descendants of the element.
    pub fn descendants<'tree>(
        &'tree self,
        tree: &'tree XpathItemTree,
    ) -> impl Iterator<Item = &'tree XpathItemTreeNode> + 'tree {
        tree.root_node
            .descendants(&tree.arena)
            .map(|node_id| tree.get(node_id))
    }

    /// Get the parent of the element.
    ///
    /// # Arguments
    ///
    /// * `tree` - The tree containing the element.
    ///
    /// # Returns
    ///
    /// The parent of the element if it exists, or `None` if it does not.
    pub fn parent<'tree>(&self, tree: &'tree XpathItemTree) -> Option<&'tree XpathItemTreeNode> {
        let id = match self {
            XpathItemTreeNode::ElementNode(e) => Some(e.id()),
            XpathItemTreeNode::TextNode(t) => Some(t.id()),
            XpathItemTreeNode::AttributeNode(a) => Some(a.id()),
            _ => None,
        };

        id.and_then(|id| {
            let parent_id = tree.arena.get(id).unwrap().parent()?;
            Some(tree.get(parent_id))
        })
    }

    /// Get an iterator over all text contained in this node and its descendants.
    ///
    /// Includes whitespace text nodes.
    /// Text nodes are split by opening and closing tags contained in the current node.
    pub fn itertext<'this, 'tree>(&'this self, tree: &'tree XpathItemTree) -> TextIter<'this>
    where
        'tree: 'this,
    {
        TextIter::new(tree, self)
    }

    /// Get all text contained in this element and its descendants.
    ///
    /// # Arguments
    ///
    /// * `tree` - The tree that this element is a part of.
    ///
    /// # Returns
    ///
    /// A string of all text contained in this element and its descendants.
    pub fn text_content<'tree>(&self, tree: &'tree XpathItemTree) -> String {
        match self {
            XpathItemTreeNode::DocumentNode(node) => node.text_content(tree),
            XpathItemTreeNode::ElementNode(node) => node.text_content(tree),
            XpathItemTreeNode::PINode(_) => String::from(""),
            XpathItemTreeNode::CommentNode(_) => String::from(""),
            XpathItemTreeNode::TextNode(node) => node.content.to_string(),
            XpathItemTreeNode::AttributeNode(_) => String::from(""),
        }
    }

    /// Text before the first subelement. This is either a string or the value None, if there was no text.
    ///
    /// Use [`ElementNode::text_content`] to get all text _including_ text in descendant nodes.
    ///
    /// # Arguments
    ///
    /// * `tree` - The tree that this element is a part of.
    ///
    /// # Returns
    ///
    /// A string of all text contained in this element.
    pub fn text<'tree>(&self, tree: &'tree XpathItemTree) -> Option<String> {
        match self {
            XpathItemTreeNode::DocumentNode(node) => node.text(tree),
            XpathItemTreeNode::ElementNode(node) => node.text(tree),
            XpathItemTreeNode::PINode(_) => None,
            XpathItemTreeNode::CommentNode(_) => None,
            XpathItemTreeNode::TextNode(node) => Some(node.content.to_string()),
            XpathItemTreeNode::AttributeNode(_) => None,
        }
    }

    pub fn display(&self, tree: &XpathItemTree, formatting: DisplayFormatting) -> String {
        match self {
            XpathItemTreeNode::DocumentNode(node) => node.display(tree, formatting),
            XpathItemTreeNode::ElementNode(node) => node.display(tree, formatting),
            XpathItemTreeNode::PINode(node) => node.to_string(),
            XpathItemTreeNode::CommentNode(node) => node.to_string(),
            XpathItemTreeNode::TextNode(node) => node.to_string(),
            XpathItemTreeNode::AttributeNode(node) => node.to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DisplayFormatting {
    Pretty,
    NoChildren,
}

/// An iterator over all text contained in a element and its descendants.
pub struct TextIter<'a> {
    iter_chain: Box<dyn Iterator<Item = String> + 'a>,
}

impl<'a> TextIter<'a> {
    pub(crate) fn new(tree: &'a XpathItemTree, node: &'a XpathItemTreeNode) -> TextIter<'a> {
        let mut iter_chain: Box<dyn Iterator<Item = String>> = Box::new(iter::empty());

        for child in node.children(tree) {
            match child {
                XpathItemTreeNode::TextNode(text) => {
                    iter_chain = Box::new(iter_chain.chain(iter::once(text.content.clone())));
                }
                XpathItemTreeNode::ElementNode(_child_element) => {
                    iter_chain = Box::new(iter_chain.chain(TextIter::new(tree, child)));
                }
                _ => {}
            }
        }

        TextIter { iter_chain }
    }
}

impl<'a> Iterator for TextIter<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter_chain.next()
    }
}

/// A tree of [`XpathItemTreeNode`]s.
///
/// This tree can be searched using an [`Xpath`] expression.
///
/// This tree is created from an [`HtmlDocument`],
/// and bridges the gap between the [html](crate::html) and [xpath](crate::xpath) modules.
///
/// # Example
///
/// ```rust
/// use skyscraper::html;
/// use skyscraper::xpath::{self, XpathItemTree};
///
/// let text = "<html></html";
///
/// let document = html::parse(text).unwrap();
/// let xpath_item_tree = XpathItemTree::from(&document);
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct XpathItemTree {
    /// The index tree that stores the nodes.
    pub(crate) arena: Arena<XpathItemTreeNode>,

    /// The root node of the document.
    pub(crate) root_node: NodeId,
}

impl XpathItemTree {
    pub(crate) fn new(arena: Arena<XpathItemTreeNode>, root_node: NodeId) -> Self {
        XpathItemTree { arena, root_node }
    }

    pub(crate) fn get_index_node(&self, id: NodeId) -> &indextree::Node<XpathItemTreeNode> {
        self.arena
            .get(id)
            .expect("xpath item node missing from tree")
    }

    pub(crate) fn get(&self, id: NodeId) -> &XpathItemTreeNode {
        let indextree_node = self.get_index_node(id);

        indextree_node.get()
    }

    /// Get the document's root node.
    pub fn root(&self) -> &XpathItemTreeNode {
        self.get(self.root_node)
    }

    /// Get an iterator over all nodes in the tree.
    pub fn iter(&self) -> impl Iterator<Item = &XpathItemTreeNode> {
        self.arena.iter().map(|node| {
            let id = self.arena.get_node_id(node).unwrap();
            self.get(id)
        })
    }
}

impl Display for XpathItemTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.root().display(self, DisplayFormatting::Pretty)
        )
    }
}

impl From<&HtmlDocument> for XpathItemTree {
    fn from(html_document: &HtmlDocument) -> Self {
        fn internal_from(
            current_html_node: &DocumentNode,
            html_document: &HtmlDocument,
            item_arena: &mut Arena<XpathItemTreeNode>,
        ) -> NodeId {
            let html_node = html_document
                .get_html_node(&current_html_node)
                .expect("html document missing expected node");

            let root_item_id = match html_node {
                HtmlNode::Tag(tag) => {
                    let node =
                        XpathItemTreeNode::ElementNode(ElementNode::new(tag.name.to_string()));

                    let item_id = item_arena.new_node(node);
                    item_arena
                        .get_mut(item_id)
                        .unwrap()
                        .get_mut()
                        .as_element_node_mut()
                        .unwrap()
                        .set_id(item_id);

                    let attributes: Vec<AttributeNode> = tag
                        .attributes
                        .iter()
                        .map(|(name, value)| {
                            AttributeNode::new(name.to_string(), value.to_string())
                        })
                        .collect();

                    for attribute in attributes {
                        let attribute_node = XpathItemTreeNode::AttributeNode(attribute);
                        let attribute_id = item_arena.new_node(attribute_node);

                        item_id.append(attribute_id, item_arena);

                        item_arena
                            .get_mut(attribute_id)
                            .unwrap()
                            .get_mut()
                            .as_attribute_node_mut()
                            .unwrap()
                            .set_id(attribute_id);
                    }

                    item_id
                }
                HtmlNode::Text(text) => {
                    let node = XpathItemTreeNode::TextNode(TextNode::new(text.value.to_string()));

                    let item_id = item_arena.new_node(node);
                    item_arena
                        .get_mut(item_id)
                        .unwrap()
                        .get_mut()
                        .as_text_node_mut()
                        .unwrap()
                        .set_id(item_id);

                    item_id
                }
            };

            for child in current_html_node.children(&html_document) {
                let child_node = internal_from(&child, html_document, item_arena);
                root_item_id.append(child_node, item_arena);
            }

            root_item_id
        }

        let mut item_arena = Arena::<XpathItemTreeNode>::new();
        let root_node_id =
            item_arena.new_node(XpathItemTreeNode::DocumentNode(XpathDocumentNode {}));
        let first_child = internal_from(&html_document.root_node, &html_document, &mut item_arena);
        root_node_id.append(first_child, &mut item_arena);

        XpathItemTree {
            arena: item_arena,
            root_node: root_node_id,
        }
    }
}
