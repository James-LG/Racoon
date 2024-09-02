//! Create [HtmlDocuments](HtmlDocument) from textual input.

// modules
pub mod malformed_html_handlers;
pub mod parse_options;

// re-exports
pub use parse_options::ParseOptionsBuilder;

// regular imports
use std::iter::Peekable;

use crate::html::tokenizer::{self, LexError, Token};
use indextree::{Arena, Node, NodeId};
use thiserror::Error;

use self::{malformed_html_handlers::MismatchedTagHandlerContext, parse_options::ParseOptions};

use super::{DocumentNode, HtmlDocument, HtmlNode, HtmlTag, HtmlText, VOID_TAGS};

/// An error occuring during the parsing of text to an [HtmlDocument].
#[derive(Error, Debug)]
pub enum ParseError {
    /// An error occuring during the tokenization of HTML text.
    #[error("Lex error {0}")]
    LexError(#[from] LexError),

    /// Another tag was started before the previous tag was closed.
    ///
    /// The starting and closing of tags in this scenario is not referring to
    /// the contents of the tag, but the tag definition itself as shown below.
    ///
    /// ```text
    /// <div  </div>
    /// ^   ^ ^^   ^
    /// │   │ ││   └ Close the "EndTag"
    /// │   │ └┴ Error: Open the "EndTag"
    /// │   └ Missing: Close the "StartTag"
    /// └ Open the "StartTag"
    /// ```
    #[error("Start tag encountered before previous tag was closed")]
    OpenTagBeforePreviousClosed,

    /// Attempted to close a tag before it was opened
    ///
    /// The starting and closing of tags in this scenario is not referring to
    /// the contents of the tag, but the tag definition itself as shown below.
    ///
    /// ```text
    ///  div> </div>
    /// ^   ^ ^^   ^
    /// │   │ ││   │ Close the "EndTag"
    /// │   │ └┴ Open the "EndTag"
    /// │   └ Error: Close the "StartTag"
    /// └ Missing: Open the "StartTag"
    /// ```
    #[error("Tag close encountered before a tag was opened")]
    TagClosedBeforeOpened,

    /// Attempted to apply an end tag to an [HtmlNode::Text].
    ///
    /// This would most likely caused by a bug in the parser or tokenizer,
    /// rather than being due to malformed HTML input.
    #[error("End tag attempted to close a text node")]
    EndTagForTextNode,

    /// Caused when the end tag name does not match the start tag name.
    ///
    /// ```text
    /// <div></span>
    ///  ^^^   ^^^^
    ///  │││   └┴┴┴ Error: "span" does not match "div"
    ///  └┴┴ Open a tag named "div"
    /// ```
    #[error("End tag name `{end_name}` mismatched open tag name `{open_name}`")]
    EndTagMismatch {
        /// The name of the ending tag. E.g. `</div` = "div".
        end_name: String,
        /// The name of the starting tag. E.g. `<div` = "div".
        open_name: String,
    },

    /// An identifier token such as "div" was found outside of a tag.
    ///
    /// This would most likely be caused by a bug in the tokenizer, as
    /// any string outside of a tag should be marked as a text token.
    ///
    /// For example this should *not* cause an error:
    /// ```text
    /// <div>div</div>
    ///      ^^^
    ///      └┴┴ Should be marked as "text" not "identifier"
    /// ```
    #[error("Identifier `{identifier}` encountered outside of tag.")]
    IdentifierOutsideTag {
        /// The identifier that was found outside of a tag.
        identifier: String,
    },

    /// Attempted to add an HTML attribute such as `id="node"` to an [HtmlNode::Text].
    #[error("Attempted to add attribute to text node")]
    AttributeOnTextNode,

    /// Missing a quoted literal value after an assignment sign in a tag.
    ///
    /// ```text
    /// <div id =  />
    ///  ^^^ ^^ ^ ^
    ///  │││ ││ │ └ Error: Missing value such as "node"
    ///  │││ ││ └ Assignment sign
    ///  │││ └┴ Identifier "id"
    ///  └┴┴ Tag name "div"
    /// ```
    #[error("Expected literal after assignment sign {tag_name}")]
    MissingLiteralAfterAssignmentSign {
        /// The name of the tag with the error.
        tag_name: String,
    },

    /// Token iterator ended while a tag was still being parsed.
    ///
    /// ```text
    /// <div
    ///     ^
    ///     └ Error: End of tokens before tag was closed
    /// ```
    #[error("Unexpected end of tokens")]
    UnexpectedEndOfTokens,

    /// Text was encountered while a tag definition was still open.
    #[error("Text encountered before previous tag was closed")]
    TextBeforePreviousClosed,

    /// Could not find the root node of the document.
    ///
    /// Check if the document is empty or malformed.
    #[error("No root node found")]
    MissingRootNode,

    /// Special <!DOCTYPE html> tag was missing the `html` type.
    ///
    /// ```text
    /// <!DOCTYPE >
    ///          ^
    ///          └ Error: Missing "html"
    /// ```
    #[error("Expected identifier `html` after !DOCTYPE")]
    MissingHtmlAfterDoctype,

    /// Special <!DOCTYPE html> tag was missing the tag close token.
    ///
    /// ```text
    /// <!DOCTYPE html
    ///                ^
    ///                └ Error: Missing tag close ">"
    /// ```
    #[error("Expected tag close after !DOCSTRING html")]
    MissingTagCloseAfterDoctype,
}

/// Represents the current state of a [Parser].
#[derive(Default)]
pub struct ParserState {
    arena: Arena<HtmlNode>,
    root_key_o: Option<NodeId>,
    cur_key_o: Option<NodeId>,
    has_tag_open: bool,
}

/// Handles the parsing of HTML text into an [HtmlDocument].
///
/// ```rust
/// # use std::error::Error;
/// # fn main() -> Result<(), Box<dyn Error>> {
/// use skyscraper::html::parse::{Parser, ParseOptionsBuilder, malformed_html_handlers::CloseMismatchedTagHandler};
/// let input = r#"<html>hi</html>"#;
///
/// let options = ParseOptionsBuilder::new()
///     .with_mismatched_tag_handler(Box::new(CloseMismatchedTagHandler::new(None)))
///     .build();
///
/// let html = Parser::new(options).parse(input)?;
/// # Ok(())
/// # }
/// ```
pub struct Parser {
    options: ParseOptions,
}

impl Parser {
    /// Creates a new [Parser].
    pub fn new(options: ParseOptions) -> Self {
        Self { options }
    }

    /// Parses `text` into an [HtmlDocument].
    pub fn parse(&self, text: &str) -> Result<HtmlDocument, ParseError> {
        let tokens = tokenizer::lex(text)?;

        let mut state = ParserState {
            arena: Arena::new(),
            root_key_o: None,
            cur_key_o: None,
            has_tag_open: false,
        };

        let mut tokens = tokens.into_iter().peekable();

        while let Some(token) = tokens.next() {
            match token {
                Token::StartTag(tag_name) => {
                    // Skip the special doctype tag so a proper root is selected.
                    if is_doctype(&tag_name, &mut tokens)? {
                        continue;
                    }

                    if state.has_tag_open {
                        return Err(ParseError::OpenTagBeforePreviousClosed);
                    }

                    state.has_tag_open = true;

                    let node = HtmlNode::Tag(HtmlTag::new(tag_name));
                    let node_key = state.arena.new_node(node);

                    if let Some(cur_key) = state.cur_key_o {
                        cur_key.append(node_key, &mut state.arena);
                    }

                    state.cur_key_o = Some(node_key);
                    if state.root_key_o.is_none() {
                        state.root_key_o = state.cur_key_o;
                    }
                }
                Token::TagClose => {
                    if !state.has_tag_open {
                        return Err(ParseError::TagClosedBeforeOpened);
                    }

                    // This will be none if the root node was just closed.
                    if state.cur_key_o.is_some() {
                        let cur_tree_node = get_tree_node(state.cur_key_o, &state.arena);
                        match cur_tree_node.get() {
                            HtmlNode::Tag(cur_tag) => {
                                // If this is an unpaired tag, the tag begins and ends with this token.
                                if VOID_TAGS.contains(&cur_tag.name.as_str()) {
                                    // Set current key to the parent of this tag.
                                    state.cur_key_o = cur_tree_node.parent();
                                }
                            }
                            HtmlNode::Text(_) => return Err(ParseError::EndTagForTextNode),
                        }
                    }

                    state.has_tag_open = false;
                }
                Token::EndTag(tag_name) => {
                    if state.has_tag_open {
                        return Err(ParseError::OpenTagBeforePreviousClosed);
                    }

                    state.has_tag_open = true;

                    let cur_tree_node = get_tree_node(state.cur_key_o, &state.arena);
                    let cur_html_node = cur_tree_node.get().clone();

                    match cur_html_node {
                        HtmlNode::Tag(cur_tag) => {
                            if cur_tag.name != tag_name {
                                let handler_context = MismatchedTagHandlerContext {
                                    open_tag_name: &cur_tag.name,
                                    close_tag_name: &tag_name,
                                    parser_state: &mut state,
                                };

                                self.options
                                    .mismatched_tag_handler
                                    .invoke(handler_context)?;
                            }
                        }
                        HtmlNode::Text(_) => return Err(ParseError::EndTagForTextNode),
                    }

                    // Set current key to the parent of this tag.
                    let cur_tree_node = get_tree_node(state.cur_key_o, &state.arena);
                    state.cur_key_o = cur_tree_node.parent();
                }
                Token::TagCloseAndEnd => {
                    if !state.has_tag_open {
                        return Err(ParseError::TagClosedBeforeOpened);
                    }

                    state.has_tag_open = false;

                    let cur_tree_node = get_tree_node(state.cur_key_o, &state.arena);
                    if let HtmlNode::Text(_) = cur_tree_node.get() {
                        return Err(ParseError::EndTagForTextNode);
                    }

                    // Set current key to the parent of this tag.
                    state.cur_key_o = cur_tree_node.parent();
                }
                Token::Identifier(iden) => {
                    if !state.has_tag_open {
                        return Err(ParseError::IdentifierOutsideTag { identifier: iden });
                    }

                    let token = tokens.peek().ok_or(ParseError::UnexpectedEndOfTokens)?;

                    if let Token::AssignmentSign = token {
                        tokens.next();

                        let token = tokens.next().ok_or(ParseError::UnexpectedEndOfTokens)?;

                        let cur_tree_node = get_mut_tree_node(state.cur_key_o, &mut state.arena);
                        match cur_tree_node.get_mut() {
                            HtmlNode::Tag(tag) => {
                                if let Token::Literal(lit) = token {
                                    tag.attributes.insert(iden, lit);
                                } else {
                                    return Err(ParseError::MissingLiteralAfterAssignmentSign {
                                        tag_name: tag.name.to_string(),
                                    });
                                }
                            }
                            HtmlNode::Text(_) => return Err(ParseError::AttributeOnTextNode),
                        }
                    } else {
                        // Attribute has no value; e.g., <script defer></script>
                        let cur_tree_node = get_mut_tree_node(state.cur_key_o, &mut state.arena);
                        match cur_tree_node.get_mut() {
                            HtmlNode::Tag(tag) => {
                                tag.attributes.insert(iden, String::from(""));
                            }
                            HtmlNode::Text(_) => return Err(ParseError::AttributeOnTextNode),
                        }
                    }
                }
                Token::Text(text) => {
                    if state.has_tag_open {
                        return Err(ParseError::TextBeforePreviousClosed);
                    }

                    let node = HtmlNode::Text(HtmlText::from_str(&text));
                    let node_key = state.arena.new_node(node);

                    if let Some(cur_key) = state.cur_key_o {
                        cur_key.append(node_key, &mut state.arena);
                    }
                }
                _ => (),
            }
        }

        if let Some(root_key) = state.root_key_o {
            return Ok(HtmlDocument::new(state.arena, DocumentNode::new(root_key)));
        }

        Err(ParseError::MissingRootNode)
    }
}

/// Parses HTML text into an [HtmlDocument].
///
/// This is a convenience method that creates a new [Parser] with default [ParseOptions].
/// To change the options, use [Parser] directly.
pub fn parse(text: &str) -> Result<HtmlDocument, ParseError> {
    let options = ParseOptionsBuilder::new().build();
    Parser::new(options).parse(text)
}

fn is_doctype(
    tag_name: &String,
    tokens: &mut Peekable<std::vec::IntoIter<Token>>,
) -> Result<bool, ParseError> {
    if tag_name == "!DOCTYPE" {
        let token = tokens.next().ok_or(ParseError::UnexpectedEndOfTokens)?;

        if let Token::Identifier(iden) = token {
            if iden != "html" {
                return Err(ParseError::MissingHtmlAfterDoctype);
            }

            for token in tokens {
                if let Token::TagClose = token {
                    return Ok(true);
                }
            }
            return Err(ParseError::MissingTagCloseAfterDoctype);
        } else {
            return Err(ParseError::MissingHtmlAfterDoctype);
        }
    }

    Ok(false)
}

fn get_tree_node(key: Option<NodeId>, arena: &Arena<HtmlNode>) -> &Node<HtmlNode> {
    let key = key.expect("Attempted to get a node on a none value");
    arena.get(key).expect("Node not found in arena")
}

fn get_mut_tree_node(key: Option<NodeId>, arena: &mut Arena<HtmlNode>) -> &mut Node<HtmlNode> {
    let key = key.expect("Attempted to get a node on a none value");
    arena.get_mut(key).expect("Node not found in arena")
}

#[cfg(test)]
pub mod test_helpers {
    use std::collections::HashMap;

    use crate::html::{DocumentNode, HtmlDocument, HtmlNode};

    pub fn assert_tag(
        document: &HtmlDocument,
        doc_node: DocumentNode,
        tag_name: &str,
        attributes: Option<HashMap<&str, &str>>,
    ) -> Vec<DocumentNode> {
        let html_node = document.get_html_node(&doc_node).unwrap();

        let tag = html_node.extract_as_tag();
        assert_eq!(String::from(tag_name), tag.name);

        if let Some(attributes) = attributes {
            for attr in attributes {
                assert_eq!(&String::from(attr.1), tag.attributes.get(attr.0).unwrap());
            }
        }

        return doc_node
            .children(&document)
            // filter out all whitespace-only texts
            .filter(|x| {
                let html_node = document.get_html_node(x).unwrap();
                match html_node {
                    HtmlNode::Tag(_) => true,
                    HtmlNode::Text(text) => !text.only_whitespace,
                }
            })
            .collect();
    }

    pub fn assert_text(document: &HtmlDocument, key: DocumentNode, text: &str) {
        let html_node = document.get_html_node(&key).unwrap();

        let node_text = html_node.extract_as_text();
        assert_eq!(text, node_text.value().trim());
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{
        test_helpers::{assert_tag, assert_text},
        *,
    };

    #[test]
    fn parse_works() {
        // arrange
        let text = "<html><a class=\"beans\"></a><b><ba/>yo</b></html>";

        // act
        let result = parse(text).unwrap();

        // assert
        // <html>
        let children = assert_tag(&result, result.root_node, "html", None);

        // <html> -> <a class="beans">
        {
            let key = children[0];
            let mut attributes = HashMap::new();
            attributes.insert("class", "beans");
            assert_tag(&result, key, "a", Some(attributes));
        }

        // <html> -> <b>
        {
            let key = children[1];
            let children = assert_tag(&result, key, "b", None);

            // <html> -> <b> -> <ba>
            {
                let key = children[0];
                assert_tag(&result, key, "ba", None);
            }

            // <html> -> <b> -> text()
            {
                let key = children[1];
                assert_text(&result, key, "yo");
            }
        }
    }

    #[test]
    fn parse_should_trim_newlines_out_of_start_tag_names() {
        // arrange
        let text = r#"
            <div
                id="hi"
                class="bye">
            </div>
            "#;

        // act
        let result = parse(text).unwrap();

        // assert
        // <div>
        let mut attributes = HashMap::new();
        attributes.insert("id", "hi");
        attributes.insert("class", "bye");
        assert_tag(&result, result.root_node, "div", Some(attributes));
    }

    #[test]
    fn parse_should_trim_newlines_out_of_end_tag_names() {
        // arrange
        let text = r#"
            <div>
            </div
            >
            "#;

        // act
        let result = parse(text).unwrap();

        // assert
        // <div>
        assert_tag(&result, result.root_node, "div", None);
    }

    #[test]
    fn parse_should_handle_attributes_without_value() {
        // arrange
        let html = r###"<script defer></script>"###;

        // act
        let result = parse(html).unwrap();

        // assert
        // <script>
        let key = result.root_node;
        let mut attributes = HashMap::new();
        attributes.insert("defer", "");
        assert_tag(&result, key, "script", Some(attributes));
    }

    #[test]
    fn parse_should_handle_attributes_without_value_other_attributes() {
        // arrange
        let html = r###"<script defer src="hi"></script>"###;

        // act
        let result = parse(html).unwrap();

        // assert
        // <script>
        let key = result.root_node;
        let mut attributes = HashMap::new();
        attributes.insert("defer", "");
        attributes.insert("src", "hi");
        assert_tag(&result, key, "script", Some(attributes));
    }

    #[test]
    fn parse_should_handle_text_with_triangle_brackets() {
        // arrange
        let html = r###"<div>foo > bar < baz</div>"###;

        // act
        let result = parse(html).unwrap();

        // assert
        // <div>
        let key = result.root_node;
        let children = assert_tag(&result, key, "div", None);

        // <div> -> -> text()
        {
            let key = children[0];
            assert_text(&result, key, "foo > bar < baz");
        }
    }

    #[test]
    fn parse_should_include_tag_like_text_in_script_tags() {
        // arrange
        let html = r###"<script>foo<bar></baz></script>"###;

        // act
        let result = parse(html).unwrap();

        // assert
        // <script>
        let key = result.root_node;
        let children = assert_tag(&result, key, "script", None);

        // <script> -> -> text()
        {
            let key = children[0];
            assert_text(&result, key, "foo<bar></baz>");
        }
    }

    #[test]
    fn parse_should_handle_single_tags() {
        // arrange
        let html = r###"
        <div>
            <br><hr>
            <meta charset="UTF-8">
            <link rel="stylesheet">
            <img width="500">
            <input type="submit">
            <col class="centercol">
            <area>
            <base>
            <embed>
            <keygen>
            <param>
            <source>
            <track>
            <wbr>
        </div>"###;

        // act
        let result = parse(html).unwrap();

        // assert
        // <div>
        let mut children = assert_tag(&result, result.root_node, "div", None).into_iter();

        // <div> -> <br>
        {
            let key = children.next().unwrap();
            assert_tag(&result, key, "br", None);
        }

        // <div> -> <hr>
        {
            let key = children.next().unwrap();
            assert_tag(&result, key, "hr", None);
        }

        // <div> -> <meta>
        {
            let key = children.next().unwrap();
            let mut attributes = HashMap::new();
            attributes.insert("charset", "UTF-8");
            assert_tag(&result, key, "meta", Some(attributes));
        }

        // <div> -> <link>
        {
            let key = children.next().unwrap();
            let mut attributes = HashMap::new();
            attributes.insert("rel", "stylesheet");
            assert_tag(&result, key, "link", Some(attributes));
        }

        // <div> -> <img>
        {
            let key = children.next().unwrap();
            let mut attributes = HashMap::new();
            attributes.insert("width", "500");
            assert_tag(&result, key, "img", Some(attributes));
        }

        // <div> -> <input>
        {
            let key = children.next().unwrap();
            let mut attributes = HashMap::new();
            attributes.insert("type", "submit");
            assert_tag(&result, key, "input", Some(attributes));
        }

        // <div> -> <col>
        {
            let key = children.next().unwrap();
            let mut attributes = HashMap::new();
            attributes.insert("class", "centercol");
            assert_tag(&result, key, "col", Some(attributes));
        }

        // <div> -> <area>
        {
            let key = children.next().unwrap();
            assert_tag(&result, key, "area", None);
        }

        // <div> -> <base>
        {
            let key = children.next().unwrap();
            assert_tag(&result, key, "base", None);
        }

        // <div> -> <embed>
        {
            let key = children.next().unwrap();
            assert_tag(&result, key, "embed", None);
        }

        // <div> -> <keygen>
        {
            let key = children.next().unwrap();
            assert_tag(&result, key, "keygen", None);
        }

        // <div> -> <param>
        {
            let key = children.next().unwrap();
            assert_tag(&result, key, "param", None);
        }

        // <div> -> <source>
        {
            let key = children.next().unwrap();
            assert_tag(&result, key, "source", None);
        }

        // <div> -> <track>
        {
            let key = children.next().unwrap();
            assert_tag(&result, key, "track", None);
        }

        // <div> -> <wbr>
        {
            let key = children.next().unwrap();
            assert_tag(&result, key, "wbr", None);
        }
    }

    #[test]
    fn parse_should_work_with_html() {
        // arrange
        let html = r###"<!DOCTYPE html>
        <!-- saved from url=(0026)https://www.rust-lang.org/ -->
        <html lang="en-US">
            <head>
                <title>Rust Programming Language</title>
                <meta name="viewport" content="width=device-width,initial-scale=1.0">
        
                <!-- Twitter card -->
                <meta name="twitter:card" content="summary">
            </head>
            <body>
                <main>
                    <section id="language-values" class="green">
                        <div class="w-100 mw-none ph3 mw8-m mw9-l center f3">
                            <header class="pb0">
                                <h2>
                                Why Rust?
                                </h2>
                            </header>
                            <div class="flex-none flex-l">
                                <section class="w-100 pv2 pv0-l mt4">
                                    <h3 class="f2 f1-l">Performance</h3>
                                    <p class="f3 lh-copy">
                                    Rust is blazingly fast and memory-efficient: with no runtime or
                                    garbage collector, it can power performance-critical services, run on
                                    embedded devices, and easily integrate with other languages.
                                    </p>
                                </section>
                            </div>
                        </div>
                    </section>
                </main>
                <script src="./Rust Programming Language_files/languages.js.download"/>
            </body>
        </html>"###;

        // act
        let result = parse(html).unwrap();

        // assert
        // <html>
        let mut children = assert_tag(&result, result.root_node, "html", None).into_iter();

        // <html> -> <head>
        {
            let key = children.next().unwrap();
            let mut children = assert_tag(&result, key, "head", None).into_iter();

            // <html> -> <head> -> <title>
            {
                let key = children.next().unwrap();
                let mut children = assert_tag(&result, key, "title", None).into_iter();

                // <html> -> head> -> <title> -> text()
                {
                    let key = children.next().unwrap();
                    assert_text(&result, key, "Rust Programming Language");
                }
            }

            // <html> -> <head> -> <meta name="viewport" content="width=device-width,initial-scale=1.0">
            {
                let key = children.next().unwrap();
                let mut attributes = HashMap::new();
                attributes.insert("name", "viewport");
                attributes.insert("content", "width=device-width,initial-scale=1.0");
                assert_tag(&result, key, "meta", Some(attributes));
            }

            // <html> -> <head> -> <meta name="twitter:card" content="summary">
            {
                let key = children.next().unwrap();
                let mut attributes = HashMap::new();
                attributes.insert("name", "twitter:card");
                attributes.insert("content", "summary");
                assert_tag(&result, key, "meta", Some(attributes));
            }
        }

        // <html> -> <body>
        {
            let key = children.next().unwrap();
            let mut children = assert_tag(&result, key, "body", None).into_iter();

            // <html> -> <body> -> <main>
            {
                let key = children.next().unwrap();
                let mut children = assert_tag(&result, key, "main", None).into_iter();

                // <html> -> <body> -> <main> -> <section id="language-values" class="green">
                {
                    let key = children.next().unwrap();
                    let mut attributes = HashMap::new();
                    attributes.insert("id", "language-values");
                    attributes.insert("class", "green");
                    let mut children =
                        assert_tag(&result, key, "section", Some(attributes)).into_iter();

                    // <html> -> <body> -> <main> -> <section> -> <div class="w-100 mw-none ph3 mw8-m mw9-l center f3">
                    {
                        let key = children.next().unwrap();
                        let mut attributes = HashMap::new();
                        attributes.insert("class", "w-100 mw-none ph3 mw8-m mw9-l center f3");
                        let mut children =
                            assert_tag(&result, key, "div", Some(attributes)).into_iter();

                        // <html> -> <body> -> <main> -> <section> -> <div> -> <header class="pb0">
                        {
                            let key = children.next().unwrap();
                            let mut attributes = HashMap::new();
                            attributes.insert("class", "pb0");
                            let mut children =
                                assert_tag(&result, key, "header", Some(attributes)).into_iter();

                            // <html> -> <body> -> <main> -> <section> -> <div> -> <header> -> <h2>
                            {
                                let key = children.next().unwrap();
                                let mut children = assert_tag(&result, key, "h2", None).into_iter();

                                // <html> -> <body> -> <main> -> <section> -> <div> -> <header> -> <h2> -> text()
                                {
                                    let key = children.next().unwrap();
                                    assert_text(&result, key, "Why Rust?")
                                }
                            }
                        }

                        // <html> -> <body> -> <main> -> <section> -> <div> -> <div class="flex-none flex-l">
                        {
                            let key = children.next().unwrap();
                            let mut attributes = HashMap::new();
                            attributes.insert("class", "flex-none flex-l");
                            let mut children =
                                assert_tag(&result, key, "div", Some(attributes)).into_iter();

                            // <html> -> <body> -> <main> -> <section> -> <div> -> <div> -> <section class="w-100 pv2 pv0-l mt4">
                            {
                                let key = children.next().unwrap();
                                let mut attributes = HashMap::new();
                                attributes.insert("class", "w-100 pv2 pv0-l mt4");
                                let mut children =
                                    assert_tag(&result, key, "section", Some(attributes))
                                        .into_iter();

                                // <html> -> <body> -> <main> -> <section> -> <div> -> <div> -> <section> -> <h3 class="f2 f1-l">
                                {
                                    let key = children.next().unwrap();
                                    let mut attributes = HashMap::new();
                                    attributes.insert("class", "f2 f1-l");
                                    let mut children =
                                        assert_tag(&result, key, "h3", Some(attributes))
                                            .into_iter();

                                    // <html> -> <body> -> <main> -> <section> -> <div> -> <div> -> <section> -> <h3> -> text()
                                    {
                                        let key = children.next().unwrap();
                                        assert_text(&result, key, "Performance");
                                    }
                                }

                                // <html> -> <body> -> <main> -> <section> -> <div> -> <div> -> <section> -> <p class="f3 lh-copy">
                                {
                                    let key = children.next().unwrap();
                                    let mut attributes = HashMap::new();
                                    attributes.insert("class", "f3 lh-copy");
                                    let mut children =
                                        assert_tag(&result, key, "p", Some(attributes)).into_iter();

                                    // <html> -> <body> -> <main> -> <section> -> <div> -> <div> -> <section> -> <p> -> text()
                                    {
                                        let mut t = String::from("Rust is blazingly fast and memory-efficient: with no runtime or");
                                        t = format!("{}\n                                    garbage collector, it can power performance-critical services, run on", t);
                                        t = format!("{}\n                                    embedded devices, and easily integrate with other languages.", t);

                                        let key = children.next().unwrap();
                                        assert_text(&result, key, &t);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // <html> -> <body> -> <script src="./Rust Programming Language_files/languages.js.download"/>
            {
                let key = children.next().unwrap();
                let mut attributes = HashMap::new();
                attributes.insert(
                    "src",
                    "./Rust Programming Language_files/languages.js.download",
                );
                assert_tag(&result, key, "script", Some(attributes));
            }
        }
    }
}
