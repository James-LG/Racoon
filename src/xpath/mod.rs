//! Parse and apply XPath expressions to HTML documents.
//!
//! Important pages:
//!
//! - [parse] - Parse a string into an [Xpath] expression.
//! - [Xpath::apply] - Apply an [Xpath] expression to an [XpathItemTree].
//! - [XpathItemTree] - A tree of [XpathItem]s that can be searched using an [Xpath] expression.
//!
//! # Example: get links with the `/@href` xpath step
//!
//! ```rust
//! # use std::error::Error;
//! #
//! use skyscraper::html;
//! use skyscraper::xpath::{self, XpathItemTree};
//!
//! # fn main() -> Result<(), Box<dyn Error>> {
//! let text = r##"
//! <html>
//!     <body>
//!         <a href="https://example1.com">Example 1</a>
//!         <a href="https://example2.com">Example 2</a>
//!     </body>
//! </html>"##;
//!
//! // Parse the HTML text
//! let document = html::parse(text)?;
//! let xpath_item_tree = XpathItemTree::from(&document);
//!
//! let xpath = xpath::parse("//a/@href")?;
//!
//! // Apply the XPath expression to our HTML document
//! let items = xpath.apply(&xpath_item_tree)?;
//!
//! let attributes: Vec<&str> = items
//!     .iter()
//!     .map(|item| item
//!         .extract_as_node() // we know it's a node all attributes are on nodes
//!         .extract_as_attribute_node() // we know it's an attribute node
//!         .value
//!         .as_str()
//!     )
//!     .collect();
//!
//! assert_eq!(attributes, vec!["https://example1.com", "https://example2.com"]);
//!
//! # Ok(())
//! # }
//! ```
//!
//! # Example: get links programatically
//!
//! ```rust
//! # use std::error::Error;
//! #
//! use skyscraper::html;
//! use skyscraper::xpath::{self, XpathItemTree};
//!
//! # fn main() -> Result<(), Box<dyn Error>> {
//! let text = r##"
//! <html>
//!     <body>
//!         <a href="https://example1.com">Example 1</a>
//!         <a href="https://example2.com">Example 2</a>
//!     </body>
//! </html>"##;
//!
//! // Parse the HTML text
//! let document = html::parse(text)?;
//! let xpath_item_tree = XpathItemTree::from(&document);
//!
//! let xpath = xpath::parse("//a")?;
//!
//! // Apply the XPath expression to our HTML document
//! let items = xpath.apply(&xpath_item_tree)?;
//!
//! let attributes: Vec<&str> = items
//!     .iter()
//!     .filter_map(|item| item
//!         .extract_as_node() // we know it's a node
//!         .extract_as_element_node() // we know it's an element node
//!         .get_attribute(&xpath_item_tree, "href")
//!     )
//!     .collect();
//!
//! assert_eq!(attributes, vec!["https://example1.com", "https://example2.com"]);
//!
//! # Ok(())
//! # }
//! ```
//!
//! # Example: get text using the `/text()` xpath step
//!
//! ```rust
//! # use std::error::Error;
//! #
//! use skyscraper::html;
//! use skyscraper::xpath::{self, XpathItemTree};
//!
//! # fn main() -> Result<(), Box<dyn Error>> {
//! let text = r##"
//! <html>
//!     <body>
//!         <div>Example 1</div>
//!         <div>Example 2</div>
//!     </body>
//! </html>"##;
//!
//! // Parse the HTML text
//! let document = html::parse(text)?;
//! let xpath_item_tree = XpathItemTree::from(&document);
//!
//! let xpath = xpath::parse("//div/text()")?;
//!
//! // Apply the XPath expression to our HTML document
//! let items = xpath.apply(&xpath_item_tree)?;
//!
//! let text_contents: Vec<String> = items
//!     .iter()
//!     .map(|item| item
//!         .extract_as_node() // we know it's a node because text is a type of node
//!         .extract_as_text_node() // we know it's a text node
//!         .content
//!         .to_string()
//!     )
//!     .collect();
//!
//! assert_eq!(text_contents, vec!["Example 1", "Example 2"]);
//!
//! # Ok(())
//! # }
//! ```
//!
//! # Example: get text programatically
//!
//! ```rust
//! # use std::error::Error;
//! #
//! use skyscraper::html;
//! use skyscraper::xpath::{self, XpathItemTree};
//!
//! # fn main() -> Result<(), Box<dyn Error>> {
//! let text = r##"
//! <html>
//!     <body>
//!         <div>Example 1</div>
//!         <div>Example 2</div>
//!     </body>
//! </html>"##;
//!
//! // Parse the HTML text
//! let document = html::parse(text)?;
//! let xpath_item_tree = XpathItemTree::from(&document);
//!
//! let xpath = xpath::parse("//div")?;
//!
//! // Apply the XPath expression to our HTML document
//! let items = xpath.apply(&xpath_item_tree)?;
//!
//! let text_contents: Vec<String> = items
//!     .iter()
//!     .map(|item| item
//!         .extract_as_node() // we know it's a node because text is type of node
//!         .extract_as_element_node() // we know it's an element node
//!         .text_content(&xpath_item_tree)
//!     )
//!     .collect();
//!
//! assert_eq!(text_contents, vec!["Example 1", "Example 2"]);
//!
//! # Ok(())
//! # }
//! ```

use thiserror::Error;

use self::{
    grammar::{data_model::XpathItem, xpath},
    xpath_item_set::XpathItemSet,
};

pub mod grammar;
pub mod query;
pub mod xpath_item_set;

pub use self::grammar::{Xpath, XpathItemTree};

/// Error that occurs when parsing an [Xpath] expression.
#[derive(PartialEq, Debug, Error)]
#[error("Error parsing expression: {msg}")]
pub struct ExpressionParseError {
    msg: String,
}

/// Parse a string into an [Xpath] expression.
///
/// # Example
///
/// ```rust
/// use skyscraper::xpath::parse;
///
/// let xpath = parse("//div[@class='yes']/parent::div/div[@class='duplicate']")
///    .expect("xpath is invalid");
/// ```
pub fn parse(input: &str) -> Result<Xpath, ExpressionParseError> {
    xpath(input).map(|x| x.1).map_err(|e| ExpressionParseError {
        msg: format!("{}", e),
    })
}

/// Error that occurs when applying an [Xpath] expression to an [XpathItemTree].
#[derive(PartialEq, Debug, Error)]
#[error("Error applying expression {msg}")]
pub struct ExpressionApplyError {
    msg: String,
}

impl ExpressionApplyError {
    pub(crate) fn new(msg: String) -> Self {
        Self { msg }
    }
}

#[derive(Debug)]
pub(crate) struct XpathExpressionContext<'tree> {
    item_tree: &'tree XpathItemTree,
    item: XpathItem<'tree>,
    position: usize,

    // size is part of the XPath expression context spec, and will be used eventually
    #[allow(unused)]
    size: usize,

    /// `true` if this expression is being applied to the root item tree;
    /// `false` if this expression is being applied to a specific item in the tree.
    ///
    /// This should not be modified for the entire evaluation cycle of an expression.
    is_root_level: bool, // TODO: This should be `is_initial_step`; it's not used for the root level
}

impl<'tree> XpathExpressionContext<'tree> {
    pub fn new(
        item_tree: &'tree XpathItemTree,
        items: &XpathItemSet<'tree>,
        position: usize,
        is_root_level: bool,
    ) -> Self {
        Self {
            item_tree,
            item: items[position - 1].clone(), // Position is 1-based
            position: position,
            size: items.len(),
            is_root_level,
        }
    }

    pub fn new_single(
        item_tree: &'tree XpathItemTree,
        item: XpathItem<'tree>,
        is_root_level: bool,
    ) -> Self {
        Self {
            item_tree,
            item,
            position: 1,
            size: 1,
            is_root_level,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_should_handle_multiple_double_slashes() {
        // arrange
        let xpath_text = r###"//hello//world"###;

        // act
        let xpath = parse(xpath_text).unwrap();

        // assert
        assert_eq!(xpath.to_string(), xpath_text);
    }

    #[test]
    fn parse_should_handle_reverse_step_after_double_slash() {
        // arrange
        let xpath_text = r###"//hello//parent::world"###;

        // act
        let xpath = parse(xpath_text).unwrap();

        // assert
        assert_eq!(xpath.to_string(), xpath_text);
    }
}
