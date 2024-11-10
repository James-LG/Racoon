//! <https://html.spec.whatwg.org/multipage/parsing.html>

use std::f32::consts::E;

use indextree::{Arena, NodeId};
use log::warn;
use nom::error;
use thiserror::Error;
use tokenizer::{CommentToken, HtmlToken, Parser, TagToken, TagTokenType, TokenizerState};

use crate::{
    vecpointer::VecPointerRef,
    xpath::{
        grammar::{
            data_model::{
                AttributeNode, CommentNode, ElementNode, TextNode, XpathDocumentNode, XpathItem,
            },
            XpathItemTreeNode,
        },
        Xpath, XpathItemTree,
    },
};

use super::DocumentNode;

mod chars;
pub mod document_builder;
mod insertion_mode_impls;
mod tokenizer;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum InsertionMode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    InHeadNoscript,
    AfterHead,
    InBody,
    Text,
    InTable,
    InTableText,
    InCaption,
    InColumnGroup,
    InTableBody,
    InRow,
    InCell,
    InSelect,
    InSelectInTable,
    InTemplate,
    AfterBody,
    InFrameset,
    AfterFrameset,
    AfterAfterBody,
    AfterAfterFrameset,
}

#[derive(Debug)]
pub(crate) enum HtmlParseErrorType {
    AbruptClosingOfEmptyComment,
    AbruptDoctypePublicIdentifier,
    AbruptDoctypeSystemIdentifier,
    AbsenceOfDigitsInNumericCharacterReference,
    CdataInHtmlContent,
    CharacterReferenceOutsideUnicodeRange,
    ControlCharacterInInputStream,
    ControlCharacterReference,
    DuplicateAttribute,
    EndTagWithAttributes,
    EndTagWithTrailingSolidus,
    EofBeforeTagName,
    EofInCdata,
    EofInComment,
    EofInDoctype,
    EofInScriptHtmlCommentLikeText,
    EofInTag,
    IncorrectlyClosedComment,
    IncorrectlyOpenedComment,
    InvalidCharacterSequenceAfterDoctypeName,
    InvalidFirstCharacterOfTagName,
    MissingAttributeValue,
    MissingDoctypeName,
    MissingDoctypePublicIdentifier,
    MissingDoctypeSystemIdentifier,
    MissingEndTagName,
    MissingQuoteBeforeDoctypePublicIdentifier,
    MissingQuoteBeforeDoctypeSystemIdentifier,
    MissingSemicolonAfterCharacterReference,
    MissingWhitespaceAfterDoctypePublicKeyword,
    MissingWhitespaceAfterDoctypeSystemKeyword,
    MissingWhitespaceBeforeDoctypeName,
    MissingWhitespaceBetweenAttributes,
    MissingWhitespaceBetweenDoctypePublicAndSystemIdentifiers,
    NestedComment,
    NoncharacterCharacterReference,
    NoncharacterInInputStream,
    NonVoidHtmlElementStartTagWithTrailingSolidus,
    NullCharacterReference,
    SurrogateCharacterReference,
    SurrogateInInputStream,
    UnexpectedCharacterAfterDoctypeSystemIdentifier,
    UnexpectedCharacterInAttributeName,
    UnexpectedCharacterInUnquotedAttributeValue,
    UnexpectedEqualsSignBeforeAttributeName,
    UnexpectedNullCharacter,
    UnexpectedQuestionMarkInsteadOfTagName,
    UnexpectedSolidusInTag,
    UnknownNamedCharacterReference,
}

#[derive(Debug, Error)]
#[error("parse error: {message}")]
pub struct HtmlParseError {
    pub message: String,
}

impl HtmlParseError {
    pub fn new(message: &str) -> Self {
        HtmlParseError {
            message: message.to_string(),
        }
    }
}

pub fn parse(text: &str) -> Result<XpathItemTree, HtmlParseError> {
    let mut parser = HtmlParser::new();
    parser.parse(text)
}

/// <https://infra.spec.whatwg.org/#html-namespace>
pub(crate) const HTML_NAMESPACE: &str = "http://www.w3.org/1999/xhtml";

/// <https://infra.spec.whatwg.org/#svg-namespace>
pub(crate) const SVG_NAMESPACE: &str = "http://www.w3.org/2000/svg";

pub(crate) static ELEMENT_IN_SCOPE_TYPES: [&str; 9] = [
    "applet", "caption", "html", "table", "td", "th", "marquee", "object", "template",
];
pub(crate) static GENERATE_IMPLIED_END_TAG_TYPES: [&str; 10] = [
    "dd", "dt", "li", "optgroup", "option", "p", "rb", "rp", "rt", "rtc",
];

/// <https://html.spec.whatwg.org/multipage/parsing.html#special>
pub(crate) static SPECIAL_ELEMENTS: [&str; 83] = [
    "address",
    "applet",
    "area",
    "article",
    "aside",
    "base",
    "basefont",
    "bgsound",
    "blockquote",
    "body",
    "br",
    "button",
    "caption",
    "center",
    "col",
    "colgroup",
    "dd",
    "details",
    "dir",
    "div",
    "dl",
    "dt",
    "embed",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "frame",
    "frameset",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "head",
    "header",
    "hgroup",
    "hr",
    "html",
    "iframe",
    "img",
    "input",
    "keygen",
    "li",
    "link",
    "listing",
    "main",
    "marquee",
    "menu",
    "meta",
    "nav",
    "noembed",
    "noframes",
    "noscript",
    "object",
    "ol",
    "p",
    "param",
    "plaintext",
    "pre",
    "script",
    "search",
    "section",
    "select",
    "source",
    "style",
    "summary",
    "table",
    "tbody",
    "td",
    "template",
    "textarea",
    "tfoot",
    "th",
    "thead",
    "title",
    "tr",
    "track",
    "ul",
    "wbr",
    "xmp",
];

pub(crate) struct CreateAnElementForTheTokenResult {
    element: ElementNode,
    attributes: Vec<AttributeNode>,
}

#[derive(Debug, Clone)]
pub(crate) enum NodeOrMarker {
    Node(NodeEntry),
    Marker,
}

#[derive(Debug, Clone)]
pub(crate) struct NodeEntry {
    node_id: NodeId,
    token: TagToken,
}

pub struct HtmlParser {
    error_handler: Box<dyn ParseErrorHandler>,
    insertion_mode: InsertionMode,
    template_insertion_modes: Vec<InsertionMode>,
    original_insertion_mode: Option<InsertionMode>,
    open_elements: Vec<NodeId>,
    context_element: Option<NodeId>,
    arena: Arena<XpathItemTreeNode>,
    root_node: Option<NodeId>,
    foster_parenting: bool,
    frameset_ok: bool,
    active_formatting_elements: Vec<NodeOrMarker>,
    head_element_pointer: Option<NodeId>,
    form_element_pointer: Option<NodeId>,
}

impl HtmlParser {
    pub fn new() -> Self {
        HtmlParser {
            error_handler: Box::new(DefaultParseErrorHandler),
            insertion_mode: InsertionMode::Initial,
            template_insertion_modes: Vec::new(),
            original_insertion_mode: None,
            open_elements: Vec::new(),
            context_element: None,
            arena: Arena::new(),
            root_node: None,
            foster_parenting: false,
            frameset_ok: true,
            active_formatting_elements: Vec::new(),
            head_element_pointer: None,
            form_element_pointer: None,
        }
    }

    pub fn parse(&mut self, text: &str) -> Result<XpathItemTree, HtmlParseError> {
        // set document node as the root node
        let document_node_id = self
            .arena
            .new_node(XpathItemTreeNode::DocumentNode(XpathDocumentNode::new()));

        self.root_node = Some(document_node_id);

        let mut open_elements: Vec<XpathItemTreeNode> = Vec::new();

        let chars: Vec<char> = text.chars().collect();
        let input_stream = VecPointerRef::new(&chars);
        let mut tokenizer = tokenizer::Tokenizer::new(input_stream, Box::new(self));
        let mut tokenizer_error_handler = tokenizer::DefaultTokenizerErrorHandler;

        tokenizer.set_error_handler(Box::new(&tokenizer_error_handler));

        while !tokenizer.is_terminated() {
            tokenizer.step()?;
        }

        let arena = std::mem::replace(&mut self.arena, Arena::new());
        let document = XpathItemTree::new(arena, document_node_id);
        Ok(document)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#current-node>
    pub(crate) fn current_node(&self) -> Option<&XpathItemTreeNode> {
        self.open_elements
            .last()
            .and_then(|id| self.arena.get(*id).map(|node| node.get()))
    }

    pub(crate) fn current_node_id(&self) -> Option<NodeId> {
        self.open_elements.last().map(|id| *id)
    }

    pub(crate) fn current_node_id_result(&self) -> Result<NodeId, HtmlParseError> {
        self.current_node_id()
            .ok_or(HtmlParseError::new("no current node"))
    }

    pub(crate) fn current_node_as_element(&self) -> Option<&ElementNode> {
        self.current_node().and_then(|node| match node {
            XpathItemTreeNode::ElementNode(element) => Some(element),
            _ => None,
        })
    }

    pub(crate) fn current_node_as_element_result(&self) -> Result<&ElementNode, HtmlParseError> {
        self.current_node_as_element()
            .ok_or(HtmlParseError::new("current node is not an element"))
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#current-template-insertion-mode>
    pub(crate) fn current_template_insertion_mode(&self) -> Option<InsertionMode> {
        self.template_insertion_modes.last().map(|mode| *mode)
    }

    pub(crate) fn top_node(&self) -> Option<&XpathItemTreeNode> {
        self.open_elements
            .first()
            .map(|id| self.arena.get(*id).unwrap().get())
    }

    pub(crate) fn top_node_mut(&mut self) -> Option<&mut XpathItemTreeNode> {
        self.open_elements
            .first()
            .map(|id| self.arena.get_mut(*id).unwrap().get_mut())
    }

    pub(crate) fn new_node(&mut self, node: XpathItemTreeNode) -> NodeId {
        let id = self.arena.new_node(node);

        let node: &mut XpathItemTreeNode = self.arena.get_mut(id).unwrap().get_mut();

        if let XpathItemTreeNode::ElementNode(element) = node {
            element.set_id(id);
        } else if let XpathItemTreeNode::AttributeNode(attribute) = node {
            attribute.set_id(id);
        }

        id
    }

    pub(crate) fn open_elements_as_nodes(&self) -> Vec<&XpathItemTreeNode> {
        self.open_elements
            .iter()
            .map(|id| self.arena.get(*id).unwrap().get())
            .collect()
    }

    pub(crate) fn open_elements_has_element(&self, tag_name: &str) -> bool {
        self.open_elements
            .iter()
            .any(|id| match self.arena.get(*id).unwrap().get() {
                XpathItemTreeNode::ElementNode(element) => element.name == tag_name,
                _ => false,
            })
    }

    pub(crate) fn handle_error(&self, error: HtmlParserError) -> Result<(), HtmlParseError> {
        match error {
            HtmlParserError::MinorError(err) => {
                dbg!(err);
                Ok(())
            }
            HtmlParserError::FatalError(err) => Err(HtmlParseError::new(&err)),
        }
    }

    pub(crate) fn add_attribute_to_element(
        &mut self,
        element_id: NodeId,
        name: String,
        value: String,
    ) -> Result<(), HtmlParseError> {
        let attribute = AttributeNode::new(name, value);
        let item_id = self.new_node(XpathItemTreeNode::AttributeNode(attribute));

        element_id.append(item_id, &mut self.arena);

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#insert-an-html-element>
    pub(crate) fn insert_an_html_element(
        &mut self,
        token: TagToken,
    ) -> Result<NodeId, HtmlParseError> {
        self.insert_foreign_element(token, HTML_NAMESPACE, false)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#insert-a-foreign-element>
    pub(crate) fn insert_foreign_element(
        &mut self,
        token: TagToken,
        namespace: &str,
        only_add_to_element_stack: bool,
    ) -> Result<NodeId, HtmlParseError> {
        let adjusted_insertion_location = if only_add_to_element_stack {
            None
        } else {
            Some(self.appropriate_place_for_inserting_a_node(None)?)
        };

        let result = self.create_an_element_for_the_token(token, namespace)?;

        // insert the result
        let element_id = self.insert_create_an_element_for_the_token_result(result)?;

        // append the element to the adjusted insertion location
        if let Some(adjusted_insertion_location) = adjusted_insertion_location {
            #[cfg(feature = "debug_prints")]
            {
                let element = self.arena.get(adjusted_insertion_location).unwrap().get();
                println!("child of: {:?}", element);
            }
            adjusted_insertion_location.append(element_id, &mut self.arena);
        }

        Ok(element_id)
    }

    pub(crate) fn insert_create_an_element_for_the_token_result(
        &mut self,
        result: CreateAnElementForTheTokenResult,
    ) -> Result<NodeId, HtmlParseError> {
        // add the element to the arena
        #[cfg(feature = "debug_prints")]
        println!("inserting element: {:?}", result.element);
        let element_id = self.new_node(XpathItemTreeNode::ElementNode(result.element));

        // add the attributes to the element
        for attribute in result.attributes {
            self.add_attribute_to_element(element_id, attribute.name, attribute.value)?;
        }

        self.open_elements.push(element_id);

        Ok(element_id)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#insert-an-element-at-the-adjusted-insertion-location>
    pub(crate) fn insert_element_at_adjusted_insertion_location(
        &mut self,
        element_id: NodeId,
    ) -> Result<(), HtmlParseError> {
        let adjusted_insertion_location = self.appropriate_place_for_inserting_a_node(None)?;

        adjusted_insertion_location.append(element_id, &mut self.arena);

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#insert-a-comment>
    pub(crate) fn insert_a_comment(
        &mut self,
        comment: CommentToken,
        parent_override: Option<NodeId>,
    ) -> Result<(), HtmlParseError> {
        let comment_id = CommentNode::create(comment.data, &mut self.arena);

        let adjusted_insertion_location = if let Some(parent) = parent_override {
            parent
        } else {
            self.appropriate_place_for_inserting_a_node(None)?
        };

        adjusted_insertion_location.append(comment_id, &mut self.arena);

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#appropriate-place-for-inserting-a-node>
    pub(crate) fn appropriate_place_for_inserting_a_node(
        &self,
        override_target: Option<NodeId>,
    ) -> Result<NodeId, HtmlParseError> {
        let target = if let Some(override_target) = override_target {
            override_target
        } else {
            let open_elements: Vec<&XpathItemTreeNode> = self
                .open_elements
                .iter()
                .map(|id| self.arena.get(*id).unwrap().get())
                .collect();

            #[cfg(feature = "debug_prints")]
            println!("open elements: {:?}", open_elements);

            self.open_elements
                .last()
                .cloned()
                .ok_or(HtmlParseError::new("no current node to insert a node into"))?
        };

        let adjusted_insertion_location = if self.foster_parenting {
            let last_template = self.get_last_element_by_tag_name("template");
            let last_table = self.get_last_element_by_tag_name("table");

            // if there is a last template element and either there is no last table element or the last table element is lower in the stack of open elements than the last template element
            // then the adjusted insertion location is inside the last template element's template contents.
            todo!()
        } else {
            target
        };

        Ok(adjusted_insertion_location)
    }

    fn get_last_element_by_tag_name(&self, tag_name: &str) -> Option<(usize, NodeId)> {
        for i in (0..self.open_elements.len()).rev() {
            let node_id = self.open_elements[i];
            if let Some(node) = self.arena.get(node_id) {
                if let XpathItemTreeNode::ElementNode(element) = node.get() {
                    if element.name == tag_name {
                        return Some((i, node_id));
                    }
                }
            }
        }

        None
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#create-an-element-for-the-token>
    pub(crate) fn create_an_element_for_the_token(
        &mut self,
        token: TagToken,
        namespace: &str,
    ) -> Result<CreateAnElementForTheTokenResult, HtmlParseError> {
        let local_name = token.tag_name;
        let element = self.create_element(local_name, namespace, None, None)?;

        // add the attributes
        let attributes: Vec<AttributeNode> = token
            .attributes
            .into_iter()
            .map(|attribute| AttributeNode::new(attribute.name, attribute.value))
            .collect();

        Ok(CreateAnElementForTheTokenResult {
            element,
            attributes,
        })
    }

    /// <https://dom.spec.whatwg.org/#concept-create-element>
    pub(crate) fn create_element(
        &mut self,
        local_name: String,
        namespace: &str,
        prefix: Option<&str>,
        is: Option<&str>,
    ) -> Result<ElementNode, HtmlParseError> {
        // TODO: namespace?
        let element = ElementNode::new(local_name);

        Ok(element)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#reconstruct-the-active-formatting-elements>
    pub(crate) fn reconstruct_the_active_formatting_elements(
        &mut self,
    ) -> Result<(), HtmlParseError> {
        fn step_4_rewind(
            parser: &mut HtmlParser,
            entry: &NodeEntry,
            entry_index: usize,
        ) -> Result<(), HtmlParseError> {
            let new_entry_index = parser
                .active_formatting_elements
                .iter()
                .position(|e| {
                    if let NodeOrMarker::Node(NodeEntry { node_id, .. }) = e {
                        return *node_id == entry.node_id;
                    } else {
                        return false;
                    }
                })
                .and_then(|i| if i == 0 { None } else { Some(i - 1) });

            // if there are no entries before entry_id in the list of active formatting elements, then jump to step 8 (create)
            match new_entry_index {
                None => return step_8_create(parser, entry, entry_index),
                Some(new_entry_index) => {
                    let new_entry = parser
                        .active_formatting_elements
                        .get(new_entry_index)
                        .expect("could not get new entry")
                        .clone();

                    if let NodeOrMarker::Node(new_entry) = new_entry {
                        // if new_entry is not a marker and is not in the list of open elements, then jump to step 4 (rewind)
                        if !parser.open_elements.contains(&new_entry.node_id) {
                            return step_4_rewind(parser, &new_entry, new_entry_index);
                        }
                    }

                    return step_7_advance(parser, new_entry_index);
                }
            }
        }

        fn step_7_advance(
            parser: &mut HtmlParser,
            entry_index: usize,
        ) -> Result<(), HtmlParseError> {
            let (new_index, new_entry) = parser
                .active_formatting_elements
                .iter()
                .enumerate()
                .skip(entry_index + 1)
                .find_map(|(i, e)| {
                    if let NodeOrMarker::Node(entry) = e {
                        return Some((i, entry));
                    }

                    None
                })
                .map(|(i, e)| (i, e.clone()))
                .expect("could not get new entry");

            return step_8_create(parser, &new_entry, new_index);
        }

        fn step_8_create(
            parser: &mut HtmlParser,
            entry: &NodeEntry,
            index: usize,
        ) -> Result<(), HtmlParseError> {
            let element = parser.insert_an_html_element(entry.token.clone())?;

            // replace the entry
            let new_entry = NodeEntry {
                node_id: element,
                token: entry.token.clone(),
            };

            // replace the entry in the list of active formatting elements
            parser.active_formatting_elements[index] = NodeOrMarker::Node(new_entry.clone());

            // if the entry was not the last entry in the list of active formatting elements, then jump to advance
            if index != parser.active_formatting_elements.len() - 1 {
                return step_7_advance(parser, index);
            }

            return Ok(());
        }

        if self.active_formatting_elements.is_empty() {
            return Ok(());
        }

        let entry = match self.active_formatting_elements.last().unwrap() {
            NodeOrMarker::Node(entry) => entry.clone(),
            NodeOrMarker::Marker => return Ok(()),
        };

        if self.open_elements.contains(&entry.node_id) {
            return Ok(());
        }

        step_4_rewind(self, &entry, self.active_formatting_elements.len() - 1)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#insert-a-character>
    pub(crate) fn insert_character(&mut self, data: Vec<char>) -> Result<(), HtmlParseError> {
        let adjusted_insertion_location_id = self.appropriate_place_for_inserting_a_node(None)?;
        let node = self
            .arena
            .get(adjusted_insertion_location_id)
            .unwrap()
            .get();

        if let XpathItemTreeNode::DocumentNode(_) = node {
            // The DOM will no let Document nodes have Text node children, so they are dropped on the floor.
            return Ok(());
        }

        // the adjusted insertion location in this implementation returns the parent node id
        // where we are expected to insert the new node as the last child of this parent node.
        // this means the previous sibling of the adjusted insertion location is the current last child of the parent node before inserting the new node.
        let prev_sibling_id = self
            .arena
            .get(adjusted_insertion_location_id)
            .unwrap()
            .last_child();

        let prev_sibling: Option<&mut XpathItemTreeNode> =
            prev_sibling_id.map(|id| self.arena.get_mut(id).unwrap().get_mut());

        if let Some(&mut XpathItemTreeNode::TextNode(ref mut text)) = prev_sibling {
            // If the adjusted insertion location's last child is a Text node, append the data to that Text node.
            text.content.extend(data.iter());
        } else {
            // Otherwise, insert a new Text node with the data as its data.
            let string = data.iter().collect::<String>();
            let text = XpathItemTreeNode::TextNode(TextNode::new(string));
            let text_id = self.new_node(text);

            self.arena
                .get_mut(text_id)
                .unwrap()
                .get_mut()
                .as_text_node_mut()
                .unwrap()
                .set_id(text_id);

            adjusted_insertion_location_id.append(text_id, &mut self.arena);
        }

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#has-an-element-in-the-specific-scope>
    pub(crate) fn has_an_element_in_the_specific_scope(
        &self,
        tag_names: Vec<&str>,
        element_types: Vec<&str>,
    ) -> bool {
        for node_id in self.open_elements.iter().rev() {
            if let Some(node) = self.arena.get(*node_id) {
                if let XpathItemTreeNode::ElementNode(element) = node.get() {
                    if tag_names.contains(&element.name.as_str()) {
                        return true;
                    }

                    if element_types.contains(&element.name.as_str()) {
                        return false;
                    }
                }
            }
        }

        false
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#has-an-element-in-scope>
    pub(crate) fn has_an_element_in_scope(&self, tag_name: &str) -> bool {
        self.has_an_element_in_the_specific_scope(vec![tag_name], ELEMENT_IN_SCOPE_TYPES.to_vec())
    }

    pub(crate) fn has_an_element_in_scope_by_tag_names(&self, tag_names: Vec<&str>) -> bool {
        self.has_an_element_in_the_specific_scope(tag_names, ELEMENT_IN_SCOPE_TYPES.to_vec())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#has-an-element-in-button-scope>
    pub(crate) fn has_an_element_in_button_scope(&self, tag_name: &str) -> bool {
        let mut element_types = ELEMENT_IN_SCOPE_TYPES.to_vec();
        element_types.push("button");

        self.has_an_element_in_the_specific_scope(vec![tag_name], element_types)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#has-an-element-in-list-item-scope>
    pub(crate) fn has_an_element_in_list_item_scope(&self, tag_name: &str) -> bool {
        let mut element_types = ELEMENT_IN_SCOPE_TYPES.to_vec();
        element_types.push("ol");
        element_types.push("ul");

        self.has_an_element_in_the_specific_scope(vec![tag_name], element_types)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#close-a-p-element>
    pub(crate) fn close_a_p_element(&mut self) -> Result<(), HtmlParseError> {
        self.generate_implied_end_tags(Some("p"))?;

        // If the current node is not a p element, then this is a parse error.
        if let Some(node) = self.current_node() {
            if let XpathItemTreeNode::ElementNode(element) = node {
                if element.name != "p" {
                    return self.handle_error(HtmlParserError::MinorError(
                        "closing a p element that is not the current node".to_string(),
                    ));
                }
            }
        }

        // Pop elements until a p element is popped.
        self.pop_until_tag_name("p")?;
        Ok(())
    }

    pub(crate) fn pop_until_tag_name(&mut self, tag_name: &str) -> Result<(), HtmlParseError> {
        self.pop_until_tag_name_one_of(vec![tag_name])
    }
    pub(crate) fn pop_until_tag_name_one_of(
        &mut self,
        tag_names: Vec<&str>,
    ) -> Result<(), HtmlParseError> {
        while let Some(node_id) = self.open_elements.pop() {
            let node = self.arena.get(node_id).unwrap().get();
            if let XpathItemTreeNode::ElementNode(element) = node {
                if tag_names.contains(&element.name.as_str()) {
                    break;
                }
            }
        }

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#generate-implied-end-tags>
    pub(crate) fn generate_implied_end_tags(
        &mut self,
        exclude_element: Option<&str>,
    ) -> Result<(), HtmlParseError> {
        while let Some(node) = self.current_node() {
            if let XpathItemTreeNode::ElementNode(element) = node {
                // if the element is excluded, then stop
                if let Some(exclude_element) = exclude_element {
                    if element.name == exclude_element {
                        break;
                    }
                }

                // if it is not in the list of implied end tag types, then stop
                if !GENERATE_IMPLIED_END_TAG_TYPES.contains(&element.name.as_str()) {
                    break;
                }
            }

            // otherwise keep popping elements from the stack
            self.open_elements.pop();
        }

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#using-the-rules-for>
    pub(crate) fn using_the_rules_for(
        &mut self,
        token: HtmlToken,
        insertion_mode: InsertionMode,
    ) -> Result<(), HtmlParseError> {
        let before_insertion_mode = self.insertion_mode;
        self.insertion_mode = insertion_mode;
        self.token_emitted(token)?;

        // if the insertion mode was not changed while processing the token, then set it back to the original value
        if self.insertion_mode == insertion_mode {
            self.insertion_mode = before_insertion_mode;
        }

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#generic-rcdata-element-parsing-algorithm>
    pub(crate) fn generic_rcdata_element_parsing_algorithm(
        &mut self,
        token: TagToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        self.insert_an_html_element(token)?;

        self.original_insertion_mode = Some(self.insertion_mode);
        self.insertion_mode = InsertionMode::Text;

        Ok(Acknowledgement {
            self_closed: false,
            tokenizer_state: Some(TokenizerState::RCDATA),
        })
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#push-onto-the-list-of-active-formatting-elements>
    pub(crate) fn push_onto_the_list_of_active_formatting_elements(
        &mut self,
        element_id: NodeId,
        token: TagToken,
    ) -> Result<(), HtmlParseError> {
        let element = self
            .arena
            .get(element_id)
            .unwrap()
            .get()
            .as_element_node()
            .map_err(|_| HtmlParseError::new("node is not an element node"))?;

        let elements_since_marker = self.active_formatting_elements.iter().map_while(
            |node_or_marker| match node_or_marker {
                NodeOrMarker::Node(entry) => {
                    let node = self.arena.get(entry.node_id).unwrap().get();
                    match node {
                        XpathItemTreeNode::ElementNode(element) => Some(element),
                        _ => None,
                    }
                }
                _ => None,
            },
        );

        let element_attributes = element.attributes_arena(&self.arena);
        let matching_elements = elements_since_marker
            .filter(|e| {
                if e.name != element.name || e.namespace != element.namespace {
                    return false;
                }

                let e_attributes = e.attributes_arena(&self.arena);
                if e_attributes.len() != element_attributes.len() {
                    return false;
                }

                for (i, attribute) in e_attributes.iter().enumerate() {
                    if attribute.name != element_attributes[i].name
                        || attribute.value != element_attributes[i].value
                    {
                        return false;
                    }
                }

                true
            })
            .collect::<Vec<&ElementNode>>();

        if matching_elements.len() >= 3 {
            // remove the earliest matching element from the list
            let earliest_element = matching_elements[0];
            let earliest_element_id = earliest_element.id();
            self.active_formatting_elements.retain(|node_or_marker| {
                if let NodeOrMarker::Node(entry) = node_or_marker {
                    return entry.node_id != earliest_element_id;
                }

                true
            });
        }

        self.active_formatting_elements
            .push(NodeOrMarker::Node(NodeEntry {
                node_id: element_id,
                token,
            }));

        Ok(())
    }

    /// Gets active formatting elements between the end of the list and the last marker,
    /// if there is a marker, or the start of the list otherwise.
    pub(crate) fn active_formatting_elements_until_marker(
        &self,
    ) -> impl Iterator<Item = &ElementNode> {
        self.active_formatting_elements
            .iter()
            .rev()
            .map_while(|node_or_marker| {
                if let NodeOrMarker::Node(entry) = node_or_marker {
                    let node = self.arena.get(entry.node_id).unwrap().get();
                    if let XpathItemTreeNode::ElementNode(element) = node {
                        return Some(element);
                    }
                }

                None
            })
    }

    pub(crate) fn remove_from_active_formatting_elements(
        &mut self,
        element_id: NodeId,
    ) -> Result<(), HtmlParseError> {
        self.active_formatting_elements.retain(|node_or_marker| {
            if let NodeOrMarker::Node(entry) = node_or_marker {
                return entry.node_id != element_id;
            }

            true
        });

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#clear-the-list-of-active-formatting-elements-up-to-the-last-marker>
    pub(crate) fn clear_the_list_of_active_formatting_elements_up_to_the_last_marker(
        &mut self,
    ) -> Result<(), HtmlParseError> {
        let last_marker_index = self
            .active_formatting_elements
            .iter()
            .rev()
            .position(|e| matches!(e, NodeOrMarker::Marker))
            .unwrap_or(self.active_formatting_elements.len());

        self.active_formatting_elements =
            self.active_formatting_elements[..last_marker_index].to_vec();

        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#reset-the-insertion-mode-appropriately>
    pub(crate) fn reset_the_insertion_mode_appropriately(&mut self) -> Result<(), HtmlParseError> {
        fn step_3_loop(
            parser: &mut HtmlParser,
            node_id: NodeId,
            last: bool,
        ) -> Result<(), HtmlParseError> {
            let mut last = last;
            if node_id == parser.open_elements[0] {
                last = true;

                // TODO: html fragment parsing algorithm
            }

            return step_4(parser, node_id, last);
        }

        fn step_4(
            parser: &mut HtmlParser,
            node_id: NodeId,
            last: bool,
        ) -> Result<(), HtmlParseError> {
            fn step_4_3_loop(
                parser: &mut HtmlParser,
                ancestor_id: NodeId,
                last: bool,
            ) -> Result<(), HtmlParseError> {
                if ancestor_id == parser.open_elements[0] {
                    return step_4_8_done(parser);
                }

                // let ancestor be the node before ancestor in the stack of open elements
                let ancestor_position = parser
                    .open_elements
                    .iter()
                    .position(|id| *id == ancestor_id)
                    .ok_or(HtmlParseError::new(
                        "ancestor id not found in open elements",
                    ))?;
                let ancestor_id =
                    parser
                        .open_elements
                        .get(ancestor_position - 1)
                        .ok_or(HtmlParseError::new(
                            "ancestor id not found in open elements",
                        ))?;

                let ancestor = parser
                    .arena
                    .get(*ancestor_id)
                    .unwrap()
                    .get()
                    .as_element_node()
                    .map_err(|_| HtmlParseError::new("ancestor is not an element node"))?;

                if ancestor.name == "template" {
                    return step_4_8_done(parser);
                }

                if ancestor.name == "table" {
                    parser.insertion_mode = InsertionMode::InSelectInTable;
                    return Ok(());
                }

                return step_4_3_loop(parser, *ancestor_id, last);
            }

            fn step_4_8_done(parser: &mut HtmlParser) -> Result<(), HtmlParseError> {
                parser.insertion_mode = InsertionMode::InSelect;
                Ok(())
            }
            let node = parser
                .arena
                .get(node_id)
                .unwrap()
                .get()
                .as_element_node()
                .map_err(|_| HtmlParseError::new("node is not an element node"))?;

            if node.name == "select" {
                return step_4_3_loop(parser, node_id, last);
            }

            if (node.name == "td" || node.name == "th") && !last {
                parser.insertion_mode = InsertionMode::InCell;
                return Ok(());
            }

            if node.name == "tr" {
                parser.insertion_mode = InsertionMode::InRow;
                return Ok(());
            }

            if node.name == "tbody" || node.name == "thead" || node.name == "tfoot" {
                parser.insertion_mode = InsertionMode::InTableBody;
                return Ok(());
            }

            if node.name == "caption" {
                parser.insertion_mode = InsertionMode::InCaption;
                return Ok(());
            }

            if node.name == "colgroup" {
                parser.insertion_mode = InsertionMode::InColumnGroup;
                return Ok(());
            }

            if node.name == "table" {
                parser.insertion_mode = InsertionMode::InTable;
                return Ok(());
            }

            if node.name == "template" {
                parser.insertion_mode = parser
                    .current_template_insertion_mode()
                    .ok_or(HtmlParseError::new("no current template insertion mode"))?;
                return Ok(());
            }

            if node.name == "head" {
                parser.insertion_mode = InsertionMode::InHead;
                return Ok(());
            }

            if node.name == "body" {
                parser.insertion_mode = InsertionMode::InBody;
                return Ok(());
            }

            if node.name == "frameset" {
                parser.insertion_mode = InsertionMode::InFrameset;
                return Ok(());
            }

            if node.name == "html" {
                if parser.head_element_pointer.is_none() {
                    parser.insertion_mode = InsertionMode::BeforeHead;
                } else {
                    parser.insertion_mode = InsertionMode::AfterHead;
                }
                return Ok(());
            }

            if last {
                parser.insertion_mode = InsertionMode::InBody;
                return Ok(());
            }

            // let node be the node before node in the stack of open elements
            let node_position = parser
                .open_elements
                .iter()
                .position(|id| *id == node_id)
                .ok_or(HtmlParseError::new(
                    "ancestor id not found in open elements",
                ))?;
            let node_id =
                parser
                    .open_elements
                    .get(node_position - 1)
                    .ok_or(HtmlParseError::new(
                        "ancestor id not found in open elements",
                    ))?;

            return step_3_loop(parser, *node_id, last);
        }

        let last = false;
        let node_id = self.current_node_id_result()?;

        step_3_loop(self, node_id, last)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#generic-raw-text-element-parsing-algorithm>
    pub(crate) fn generic_raw_text_element_parsing_algorithm(
        &mut self,
        token: TagToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        self.insert_an_html_element(token)?;

        self.original_insertion_mode = Some(self.insertion_mode);
        self.insertion_mode = InsertionMode::Text;

        Ok(Acknowledgement {
            self_closed: false,
            tokenizer_state: Some(TokenizerState::RAWTEXT),
        })
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#stop-parsing>
    pub(crate) fn stop_parsing(&mut self) -> Result<(), HtmlParseError> {
        // mostly scripting stuff that is unsupported by Skyscraper
        Ok(())
    }

    fn adjusted_current_node_id(&self) -> Result<NodeId, HtmlParseError> {
        if let Some(context_element) = self.context_element {
            if self.open_elements.len() == 1 {
                return Ok(context_element);
            }
        }

        self.current_node_id_result()
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#generate-all-implied-end-tags-thoroughly>
    pub(crate) fn generate_all_implied_end_tags_thoroughly(
        &mut self,
    ) -> Result<(), HtmlParseError> {
        while let Some(node) = self.current_node() {
            if let XpathItemTreeNode::ElementNode(element) = node {
                if ![
                    "caption", "colgroup", "dd", "dt", "li", "optgroup", "option", "p", "rb", "rp",
                    "rt", "rtc", "tbody", "td", "tfoot", "th", "thead", "tr",
                ]
                .contains(&element.name.as_str())
                {
                    break;
                }
            }

            self.open_elements.pop();
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum HtmlParserError {
    #[error("minor error: {0}")]
    MinorError(String),
    #[error("fatal error: {0}")]
    FatalError(String),
}

pub(crate) struct Acknowledgement {
    pub self_closed: bool,
    pub tokenizer_state: Option<tokenizer::TokenizerState>,
}

impl Acknowledgement {
    fn no() -> Self {
        Acknowledgement {
            self_closed: false,
            tokenizer_state: None,
        }
    }

    fn yes() -> Self {
        Acknowledgement {
            self_closed: true,
            tokenizer_state: None,
        }
    }
}

impl Parser for HtmlParser {
    fn token_emitted(&mut self, token: HtmlToken) -> Result<Acknowledgement, HtmlParseError> {
        let self_closing = match &token {
            HtmlToken::TagToken(tag) => tag.self_closing(),
            _ => false,
        };

        #[cfg(feature = "debug_prints")]
        {
            if let HtmlToken::TagToken(TagTokenType::StartTag(token)) = &token {
                println!("start tag: {}", token.tag_name);
            }

            if let HtmlToken::TagToken(TagTokenType::EndTag(token)) = &token {
                println!("end tag: {}", token.tag_name);
            }
        }

        let acknowledgement = match self.insertion_mode {
            InsertionMode::Initial => self.initial_insertion_mode(token),
            InsertionMode::BeforeHtml => self.before_html_insertion_mode(token),
            InsertionMode::BeforeHead => self.before_head_insertion_mode(token),
            InsertionMode::InHead => self.in_head_insertion_mode(token),
            InsertionMode::InHeadNoscript => todo!(),
            InsertionMode::AfterHead => self.after_head_insertion_mode(token),
            InsertionMode::InBody => self.in_body_insertion_mode(token),
            InsertionMode::Text => self.text_insertion_mode(token),
            InsertionMode::InTable => todo!(),
            InsertionMode::InTableText => todo!(),
            InsertionMode::InCaption => todo!(),
            InsertionMode::InColumnGroup => todo!(),
            InsertionMode::InTableBody => todo!(),
            InsertionMode::InRow => todo!(),
            InsertionMode::InCell => todo!(),
            InsertionMode::InSelect => todo!(),
            InsertionMode::InSelectInTable => todo!(),
            InsertionMode::InTemplate => self.in_template_insertion_mode(token),
            InsertionMode::AfterBody => self.after_body_insertion_mode(token),
            InsertionMode::InFrameset => todo!(),
            InsertionMode::AfterFrameset => todo!(),
            InsertionMode::AfterAfterBody => self.after_after_body_insertion_mode(token),
            InsertionMode::AfterAfterFrameset => todo!(),
        }?;

        if self_closing && !acknowledgement.self_closed {
            self.error_handler
                .error_emitted(HtmlParseErrorType::NonVoidHtmlElementStartTagWithTrailingSolidus)?;
        }

        Ok(acknowledgement)
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#adjusted-current-node>
    fn adjusted_current_node(&self) -> Option<&XpathItemTreeNode> {
        if let Some(context_element) = self.context_element {
            if self.open_elements.len() == 1 {
                return Some(
                    self.arena
                        .get(context_element)
                        .expect("context element not in arena")
                        .get(),
                );
            }
        }

        self.current_node()
    }
}

pub trait ParseErrorHandler {
    fn error_emitted(&self, error: HtmlParseErrorType) -> Result<(), HtmlParseError>;
}

pub struct DefaultParseErrorHandler;

impl ParseErrorHandler for DefaultParseErrorHandler {
    fn error_emitted(&self, error: HtmlParseErrorType) -> Result<(), HtmlParseError> {
        Err(HtmlParseError {
            message: format!("{:?}", error),
        })
    }
}
