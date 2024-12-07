//! <https://www.w3.org/TR/xpath-datamodel-31/#intro>

use std::fmt::{Debug, Display};

use enum_extract_macro::EnumExtract;
use indextree::{Arena, NodeId};
use ordered_float::OrderedFloat;

use super::{DisplayFormatting, TextIter, XpathItemTree, XpathItemTreeNode};

/// <https://www.w3.org/TR/xpath-datamodel-31/#dt-item>
#[derive(PartialEq, Eq, Debug, Clone, Hash, EnumExtract)]
pub enum XpathItem<'tree> {
    /// A node in the [`XpathItemTree`].
    ///
    ///  <https://www.w3.org/TR/xpath-datamodel-31/#dt-node>
    Node(&'tree XpathItemTreeNode),

    /// A function item.
    ///
    /// <https://www.w3.org/TR/xpath-datamodel-31/#dt-function-item>
    Function(Function),

    /// An atomic value.
    ///
    /// <https://www.w3.org/TR/xpath-datamodel-31/#dt-atomic-value>
    AnyAtomicType(AnyAtomicType),
}

impl<'tree> From<&'tree XpathItemTreeNode> for XpathItem<'tree> {
    fn from(node: &'tree XpathItemTreeNode) -> Self {
        XpathItem::Node(node)
    }
}

/// An atomic value.
///
/// <https://www.w3.org/TR/xpath-datamodel-31/#types-hierarchy>
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Hash)]
pub enum AnyAtomicType {
    /// A boolean value.
    Boolean(bool),

    /// An integer value.
    Integer(i64),

    /// A float value.
    Float(OrderedFloat<f32>),

    /// A double precision float value.
    Double(OrderedFloat<f64>),

    /// A string value.
    String(String),
}

impl Display for AnyAtomicType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnyAtomicType::Boolean(b) => write!(f, "{}", b),
            AnyAtomicType::Integer(i) => write!(f, "{}", i),
            AnyAtomicType::Float(fl) => write!(f, "{}", fl),
            AnyAtomicType::Double(d) => write!(f, "{}", d),
            AnyAtomicType::String(s) => write!(f, "{}", s),
        }
    }
}

/// <https://www.w3.org/TR/xpath-datamodel-31/#dt-function-item>
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Clone, Hash)]
pub struct Function {
    // TODO
}

impl Display for Function {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("Function::fmt")
    }
}

/// <https://www.w3.org/TR/xpath-datamodel-31/#DocumentNode>
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Hash, Clone)]
pub struct XpathDocumentNode {}

impl XpathDocumentNode {
    pub(crate) fn new() -> Self {
        Self {}
    }

    /// Get all text contained in this element and its descendants.
    ///
    /// # Arguments
    ///
    /// * `tree` - The tree that this document is a part of.
    ///
    /// # Returns
    ///
    /// A string of all text contained in this document and its descendants.
    pub fn text_content<'tree>(&self, tree: &'tree XpathItemTree) -> String {
        let strings: Vec<String> = tree
            .root_node
            .children(&tree.arena)
            .into_iter()
            .map(|x| tree.get(x))
            .map(|x| x.text_content(tree))
            .collect();

        let text = strings.join("");
        text
    }

    /// Text before the first subelement. This is either a string or the value None, if there was no text.
    ///
    /// Use [`XpathDocumentNode::text_content`] to get all text _including_ text in descendant nodes.
    ///
    /// # Arguments
    ///
    /// * `tree` - The tree that this document is a part of.
    ///
    /// # Returns
    ///
    /// A string of all text contained in this document.
    pub fn text<'tree>(&self, tree: &'tree XpathItemTree) -> Option<String> {
        let strings: Vec<String> = tree
            .root_node
            .children(&tree.arena)
            .into_iter()
            .map(|x| tree.get(x))
            .map(|x| x.text(tree))
            .filter_map(|x| x.map(|x| x.to_string()))
            .collect();

        strings.into_iter().next()
    }

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
        tree.root_node
            .children(&tree.arena)
            .map(|x| tree.get(x))
            .collect()
    }

    pub fn display<'tree>(
        &self,
        tree: &'tree XpathItemTree,
        formatting: DisplayFormatting,
    ) -> String {
        let children = self.children(tree);

        let element_strings: Vec<String> = children
            .iter()
            .filter_map(|x| x.as_element_node().ok())
            .map(|x| x.display(tree, formatting, 0))
            .collect();

        element_strings.join("\n")
    }
}

/// An element node such as an HTML tag.
///
/// <https://www.w3.org/TR/xpath-datamodel-31/#ElementNode>
#[derive(PartialEq, Eq, Hash, Clone)]
pub struct ElementNode {
    /// The ID of the element.
    ///
    /// Optional to enable construction of the tree before assigning IDs.
    /// Can be considered always Some in a valid tree.
    id: Option<NodeId>,

    /// The name of the element.
    pub name: String,

    /// The namespace of the element.
    pub namespace: Option<String>,
}

impl std::fmt::Debug for ElementNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // ignore id in debug output
        f.debug_struct("ElementNode")
            .field("name", &self.name)
            .finish()
    }
}

impl ElementNode {
    /// Create a new element node.
    pub(crate) fn new(name: String) -> Self {
        Self {
            id: None,
            name,
            namespace: None,
        }
    }

    /// Set the ID of the element.
    pub(crate) fn set_id(&mut self, id: NodeId) {
        self.id = Some(id);
    }

    /// Get the ID of the element.
    pub(crate) fn id(&self) -> NodeId {
        self.id.unwrap()
    }

    /// Get all attributes of the element.
    ///
    /// # Arguments
    ///
    /// * `tree` - The tree containing the element.
    ///
    /// # Returns
    ///
    /// A vector of all attributes of the element.
    pub fn attributes<'tree>(&self, tree: &'tree XpathItemTree) -> Vec<&'tree AttributeNode> {
        self.children(tree)
            .filter_map(|x| match x {
                XpathItemTreeNode::AttributeNode(attr) => Some(attr),
                _ => None,
            })
            .collect()
    }

    pub(crate) fn attributes_arena<'arena>(
        &self,
        arena: &'arena Arena<XpathItemTreeNode>,
    ) -> Vec<&'arena AttributeNode> {
        self.children_arena(arena)
            .filter_map(|x| match x {
                XpathItemTreeNode::AttributeNode(attr) => Some(attr),
                _ => None,
            })
            .collect()
    }

    /// Get the value of an attribute.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the attribute.
    ///
    /// # Returns
    ///
    /// The value of the attribute if it exists, or `None` if it does not.
    pub fn get_attribute<'tree>(
        &self,
        tree: &'tree XpathItemTree,
        name: &str,
    ) -> Option<&'tree str> {
        self.attributes(tree)
            .iter()
            .find(|x| x.name == name)
            .map(|x| &*x.value)
    }

    pub(crate) fn add_attribute<'arena>(
        &self,
        arena: &mut Arena<XpathItemTreeNode>,
        name: String,
        value: String,
    ) -> NodeId {
        let attr = arena.new_node(XpathItemTreeNode::AttributeNode(AttributeNode::new(
            name, value,
        )));
        self.id().append(attr, arena);

        attr
    }

    /// Get all direct child nodes of the given element.
    /// Note this _does_ include attribute nodes.
    ///
    /// # Arguments
    ///
    /// * `tree` - The tree containing the element.
    ///
    /// # Returns
    ///
    /// An iterator over the child nodes.
    pub fn children<'tree>(
        &self,
        tree: &'tree XpathItemTree,
    ) -> impl Iterator<Item = &'tree XpathItemTreeNode> {
        self.id().children(&tree.arena).map(|x| tree.get(x))
    }

    pub(crate) fn children_arena<'arena>(
        &self,
        arena: &'arena Arena<XpathItemTreeNode>,
    ) -> impl Iterator<Item = &'arena XpathItemTreeNode> {
        self.id()
            .children(arena)
            .map(|x| arena.get(x).expect("node missing from arena").get())
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
        tree.get(self.id()).parent(tree)
    }

    /// Get an iterator over all text contained in this element and its descendants.
    ///
    /// Includes whitespace text nodes.
    /// Text nodes are split by opening and closing tags contained in the current element.
    pub fn itertext<'this, 'tree>(&'this self, tree: &'tree XpathItemTree) -> TextIter<'this>
    where
        'tree: 'this,
    {
        TextIter::new(tree, tree.get(self.id()))
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
        self.itertext(tree).collect::<Vec<String>>().join("")
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
    pub fn text(&self, tree: &XpathItemTree) -> Option<String> {
        let strings: Vec<String> =
            // Get all children.
            Self::get_all_text_nodes(tree, self, false)
            .into_iter()
            .map(|x| x.content)
            .collect();

        strings.into_iter().next()
    }

    fn get_all_text_nodes(
        tree: &XpathItemTree,
        node: &ElementNode,
        recurse: bool,
    ) -> Vec<TextNode> {
        node
            // Get all children of the given node.
            .children(tree)
            // Combine all the direct and indirect children into a Vec.
            .fold(Vec::new(), |mut v, child| {
                match child {
                    XpathItemTreeNode::ElementNode(child_element) => {
                        if recurse {
                            // If this child is an element node, get all the text nodes in it.
                            v.extend(Self::get_all_text_nodes(tree, &child_element, recurse));
                        }
                    }
                    XpathItemTreeNode::TextNode(text) => {
                        // If this child is a text node, push it to the Vec.
                        v.push(text.clone());
                    }
                    _ => {}
                }
                v
            })
    }

    /// Get the [XpathItem] representation of the element.
    pub fn to_item<'tree>(&self, tree: &'tree XpathItemTree) -> XpathItem<'tree> {
        XpathItem::Node(tree.get(self.id()))
    }

    pub fn display<'tree>(
        &self,
        tree: &'tree XpathItemTree,
        formatting: DisplayFormatting,
        indent: usize,
    ) -> String {
        let displayed_children: Vec<String> = match formatting {
            DisplayFormatting::Pretty => {
                let children_without_attributes =
                    self.children(tree).filter(|x| !x.is_attribute_node());
                children_without_attributes
                    .map(|x| x.display(tree, formatting, indent + 1))
                    .filter(|x| !x.trim().is_empty())
                    .collect()
            }
            DisplayFormatting::NoChildren => Vec::new(),
        };

        let attributes = self.attributes(tree);
        let displayed_attributes: Vec<String> = attributes.iter().map(|x| x.to_string()).collect();

        // indent the element
        let indentation = "  ".repeat(indent);

        let mut display_string = String::new();

        // display the start tag
        if displayed_attributes.is_empty() {
            display_string.push_str(&format!("{}<{}>", indentation, self.name,));
        } else {
            display_string.push_str(&format!(
                "{}<{} {}>",
                indentation,
                self.name,
                displayed_attributes.join(" "),
            ));
        }

        // display the children
        if !displayed_children.is_empty() {
            display_string.push_str(&format!(
                "\n{}\n{}",
                &displayed_children.join("\n"),
                indentation
            ));
        }

        // display the end tag
        display_string.push_str(&format!("</{}>", self.name));

        return display_string;
    }
}

/// An attribute node.
///
/// <https://www.w3.org/TR/xpath-datamodel-31/#AttributeNode>
#[derive(Eq, Clone, Hash)]
pub struct AttributeNode {
    /// The ID of the attribute.
    ///
    /// Optional to enable construction of the tree before assigning IDs.
    /// Can be considered always Some in a valid tree.
    id: Option<NodeId>,

    /// The name of the attribute.
    pub name: String,

    /// The value of the attribute.
    pub value: String,
}

impl Debug for AttributeNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // ignore id in debug output
        f.debug_struct("AttributeNode")
            .field("name", &self.name)
            .field("value", &self.value)
            .finish()
    }
}

impl AttributeNode {
    /// Create a new attribute node.
    pub(crate) fn new(name: String, value: String) -> Self {
        Self {
            id: None,
            name,
            value,
        }
    }

    /// Set the ID of the attribute.
    pub(crate) fn set_id(&mut self, id: NodeId) {
        self.id = Some(id);
    }

    /// Get the ID of the attribute.
    pub(crate) fn id(&self) -> NodeId {
        self.id.unwrap()
    }

    /// Get the parent of the attribute.
    ///
    /// # Arguments
    ///
    /// * `tree` - The tree containing the attribute.
    ///
    /// # Returns
    ///
    /// The parent of the attribute if it exists, or `None` if it does not.
    pub fn parent<'tree>(&self, tree: &'tree XpathItemTree) -> Option<&'tree XpathItemTreeNode> {
        tree.get(self.id()).parent(tree)
    }
}

impl Display for AttributeNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}=\"{}\"", self.name, self.value)
    }
}

impl PartialEq for AttributeNode {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.value == other.value
    }
}

/// <https://www.w3.org/TR/xpath-datamodel-31/#ProcessingInstructionNode>
#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Hash, Clone)]
pub struct PINode {
    // TODO
}

impl Display for PINode {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("PINode::fmt")
    }
}

/// <https://www.w3.org/TR/xpath-datamodel-31/#CommentNode>
#[derive(PartialOrd, Eq, Ord, Debug, Hash, Clone)]
pub struct CommentNode {
    /// The value of the comment.
    pub content: String,

    /// The ID of the comment node.
    id: Option<NodeId>,
}

impl CommentNode {
    /// Create a new comment node.
    pub(crate) fn new(content: String) -> Self {
        Self { content, id: None }
    }

    pub(crate) fn create(content: String, arena: &mut Arena<XpathItemTreeNode>) -> NodeId {
        let node_id = arena.new_node(XpathItemTreeNode::CommentNode(CommentNode::new(content)));

        arena
            .get_mut(node_id)
            .unwrap()
            .get_mut()
            .as_comment_node_mut()
            .unwrap()
            .set_id(node_id);

        node_id
    }

    /// Set the ID of the comment node.
    pub(crate) fn set_id(&mut self, id: NodeId) {
        self.id = Some(id);
    }

    /// Get the ID of the comment node.
    pub(crate) fn id(&self) -> NodeId {
        self.id.unwrap()
    }

    /// Get the parent of the comment.
    ///
    /// # Arguments
    ///
    /// * `tree` - The tree containing the comment.
    ///
    /// # Returns
    ///
    /// The parent of the comment if it exists, or `None` if it does not.
    pub fn parent<'tree>(&self, tree: &'tree XpathItemTree) -> Option<&'tree XpathItemTreeNode> {
        tree.get(self.id()).parent(tree)
    }
}

impl Display for CommentNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<!--{}-->", self.content)
    }
}

impl PartialEq for CommentNode {
    fn eq(&self, other: &Self) -> bool {
        self.content == other.content
    }
}

/// <https://www.w3.org/TR/xpath-datamodel-31/#TextNode>
#[derive(Eq, Hash, Clone)]
pub struct TextNode {
    /// The ID of the text node.
    ///
    /// Optional to enable construction of the tree before assigning IDs.
    /// Can be considered always Some in a valid tree.
    id: Option<NodeId>,

    /// The value of the text node.
    pub content: String,
}

impl Debug for TextNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // ignore id in debug output
        f.debug_struct("TextNode")
            .field("content", &self.content)
            .finish()
    }
}

impl TextNode {
    /// Create a new text node.
    pub(crate) fn new(content: String) -> Self {
        Self { id: None, content }
    }

    /// Set the ID of the text node.
    pub(crate) fn set_id(&mut self, id: NodeId) {
        self.id = Some(id);
    }

    /// Get the ID of the text node.
    pub(crate) fn id(&self) -> NodeId {
        self.id.unwrap()
    }

    /// Whether the text contains only whitespace.
    pub fn is_whitespace(&self) -> bool {
        self.content.trim().is_empty()
    }

    /// Get the parent of the text.
    ///
    /// # Arguments
    ///
    /// * `tree` - The tree containing the text.
    ///
    /// # Returns
    ///
    /// The parent of the text if it exists, or `None` if it does not.
    pub fn parent<'tree>(&self, tree: &'tree XpathItemTree) -> Option<&'tree XpathItemTreeNode> {
        tree.get(self.id()).parent(tree)
    }

    pub fn display(
        &self,
        _tree: &XpathItemTree,
        formatting: DisplayFormatting,
        indent: usize,
    ) -> String {
        let indentation = "  ".repeat(indent);
        let string = match formatting {
            DisplayFormatting::NoChildren => self.content.clone(),
            DisplayFormatting::Pretty => self.content.clone().trim().to_string(),
        };

        format!("{}{}", indentation, string)
    }
}

impl PartialEq for TextNode {
    fn eq(&self, other: &Self) -> bool {
        self.content == other.content
    }
}

impl Display for TextNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.content)
    }
}
