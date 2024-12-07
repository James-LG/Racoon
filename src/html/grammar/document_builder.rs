use indextree::{Arena, NodeId};
use thiserror::Error;

use crate::xpath::{
    grammar::{
        data_model::{
            AttributeNode, CommentNode, ElementNode, TextNode, XpathDocumentNode, XpathItem,
        },
        XpathItemTreeNode,
    },
    XpathItemTree,
};

#[derive(Debug, Error)]
#[error("DocumentBuilderError: {message}")]
pub struct DocumentBuilderError {
    message: String,
}
pub struct DocumentBuilder {
    arena: Arena<XpathItemTreeNode>,
    funcs: Vec<
        Box<
            dyn FnOnce(
                &mut Arena<XpathItemTreeNode>,
                NodeId,
            ) -> Result<NodeId, DocumentBuilderError>,
        >,
    >,
}

impl DocumentBuilder {
    pub fn new() -> Self {
        Self {
            arena: Arena::new(),
            funcs: Vec::new(),
        }
    }

    pub fn add_element(
        mut self,
        tag_name: &str,
        f: impl FnOnce(ElementBuilder) -> ElementBuilder + 'static,
    ) -> Self {
        let tag_name = tag_name.to_string();
        self.funcs.push(Box::new(move |arena, parent_id| {
            f(ElementBuilder::new(
                tag_name.clone(),
                Some(parent_id),
                arena,
            ))
            .build()
        }));

        self
    }

    pub fn add_comment(mut self, comment: &str) -> Self {
        let comment = comment.to_string();
        self.funcs.push(Box::new(move |arena, _| {
            let child_id =
                arena.new_node(XpathItemTreeNode::CommentNode(CommentNode::new(comment)));

            arena
                .get_mut(child_id)
                .unwrap()
                .get_mut()
                .as_comment_node_mut()
                .unwrap()
                .set_id(child_id);

            Ok(child_id)
        }));

        self
    }

    pub fn build(mut self) -> Result<XpathItemTree, DocumentBuilderError> {
        let document_node_id = self
            .arena
            .new_node(XpathItemTreeNode::DocumentNode(XpathDocumentNode::new()));

        for func in self.funcs {
            let child_id = func(&mut self.arena, document_node_id)?;
            document_node_id.append(child_id, &mut self.arena);
        }

        let document = XpathItemTree::new(self.arena, document_node_id);

        Ok(document)
    }
}

pub struct ElementBuilder<'arena> {
    parent_id: Option<NodeId>,
    arena: &'arena mut Arena<XpathItemTreeNode>,
    funcs: Vec<
        Box<
            dyn FnOnce(
                &mut Arena<XpathItemTreeNode>,
                NodeId,
            ) -> Result<NodeId, DocumentBuilderError>,
        >,
    >,
    tag_name: String,
}

impl<'arena> ElementBuilder<'arena> {
    pub fn new(
        tag_name: String,
        parent_id: Option<NodeId>,
        arena: &'arena mut Arena<XpathItemTreeNode>,
    ) -> Self {
        Self {
            parent_id,
            arena,
            funcs: Vec::new(),
            tag_name: tag_name.to_string(),
        }
    }

    pub fn add_element(
        mut self,
        tag_name: &str,
        f: impl FnOnce(ElementBuilder) -> ElementBuilder + 'static,
    ) -> Self {
        let tag_name = tag_name.to_string();
        self.funcs.push(Box::new(move |arena, parent_id| {
            f(ElementBuilder::new(
                tag_name.clone(),
                Some(parent_id),
                arena,
            ))
            .build()
        }));

        self
    }

    pub fn add_attributes_str(mut self, attributes: Vec<(&str, &str)>) -> Self {
        for (name, value) in attributes {
            self = self.add_attribute_str(name, value);
        }

        self
    }

    pub fn add_attribute_str(mut self, name: &str, value: &str) -> Self {
        self.add_attribute(AttributeNode::new(name.to_string(), value.to_string()))
    }

    pub fn add_attribute(mut self, attribute: AttributeNode) -> Self {
        self.funcs.push(Box::new(move |arena, _| {
            let child_id = arena.new_node(XpathItemTreeNode::AttributeNode(attribute));

            arena
                .get_mut(child_id)
                .unwrap()
                .get_mut()
                .as_attribute_node_mut()
                .unwrap()
                .set_id(child_id);

            Ok(child_id)
        }));

        self
    }

    pub fn add_text(mut self, text: &str) -> Self {
        let text = text.to_string();
        self.funcs.push(Box::new(move |arena, _| {
            let child_id = arena.new_node(XpathItemTreeNode::TextNode(TextNode::new(text)));

            arena
                .get_mut(child_id)
                .unwrap()
                .get_mut()
                .as_text_node_mut()
                .unwrap()
                .set_id(child_id);

            Ok(child_id)
        }));

        self
    }

    pub fn add_comment(mut self, comment: &str) -> Self {
        let comment = comment.to_string();
        self.funcs.push(Box::new(move |arena, _| {
            let child_id =
                arena.new_node(XpathItemTreeNode::CommentNode(CommentNode::new(comment)));

            arena
                .get_mut(child_id)
                .unwrap()
                .get_mut()
                .as_comment_node_mut()
                .unwrap()
                .set_id(child_id);

            Ok(child_id)
        }));

        self
    }

    pub fn build(mut self) -> Result<NodeId, DocumentBuilderError> {
        let element_id = self
            .arena
            .new_node(XpathItemTreeNode::ElementNode(ElementNode::new(
                self.tag_name,
            )));

        self.arena
            .get_mut(element_id)
            .unwrap()
            .get_mut()
            .as_element_node_mut()
            .unwrap()
            .set_id(element_id);

        for func in self.funcs {
            let child_id = func(&mut self.arena, element_id)?;
            element_id.append(child_id, &mut self.arena);
        }

        if let Some(parent_id) = self.parent_id {
            parent_id.append(element_id, self.arena);
        }

        Ok(element_id)
    }
}
