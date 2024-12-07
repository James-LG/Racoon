//! Functions for querying an [XpathItemTree] using an xpath expression.
//!
//! This module contains the functions used to scrape documents using xpath expressions.

use thiserror::Error;

use crate::xpath::{
    self, xpath_item_set::XpathItemSet, ExpressionApplyError, ExpressionParseError, XpathItemTree,
};

use super::{
    grammar::data_model::{AttributeNode, ElementNode, XpathItem},
    Xpath,
};

impl Xpath {
    /// Find attributes in an [XpathItemTree] using an xpath expression.
    pub fn find_attributes<'tree>(
        &self,
        tree: &'tree XpathItemTree,
    ) -> Result<Vec<&'tree AttributeNode>, ExpressionApplyError> {
        let items = self.apply(tree)?;

        let mut attributes: Vec<&AttributeNode> = Vec::new();
        for item in items {
            let attribute = item
                .as_node()
                .and_then(|node| node.as_attribute_node())
                .map_err(|e| ExpressionApplyError::new(e.to_string()))?;

            attributes.push(attribute);
        }

        Ok(attributes)
    }

    /// Find elements in an [XpathItemTree] using an xpath expression.
    pub fn find_elements<'tree>(
        &self,
        tree: &'tree XpathItemTree,
    ) -> Result<Vec<&'tree ElementNode>, ExpressionApplyError> {
        let items = self.apply(tree)?;

        let mut elements: Vec<&ElementNode> = Vec::new();
        for item in items {
            let element = item
                .as_node()
                .and_then(|node| node.as_element_node())
                .map_err(|e| ExpressionApplyError::new(e.to_string()))?;

            elements.push(element);
        }

        Ok(elements)
    }

    /// Find elements from an [XpathItem] in an [XpathItemTree] using an xpath expression.
    /// The expression will be evaluated relative to the given item.
    pub fn find_elements_from_item<'tree>(
        &self,
        tree: &'tree XpathItemTree,
        item: XpathItem<'tree>,
    ) -> Result<Vec<&'tree ElementNode>, ExpressionApplyError> {
        let items = self.apply_to_item(tree, item)?;

        let mut elements: Vec<&'tree ElementNode> = Vec::new();
        for item in items {
            let element = item
                .as_node()
                .and_then(|node| node.as_element_node())
                .map_err(|e| ExpressionApplyError::new(e.to_string()))?;

            elements.push(element);
        }

        Ok(elements)
    }

    /// Find elements from an [ElementNode] in an [XpathItemTree] using an xpath expression.
    /// The expression will be evaluated relative to the given element.
    pub fn find_elements_from_element<'tree>(
        &self,
        tree: &'tree XpathItemTree,
        element: &'tree ElementNode,
    ) -> Result<Vec<&'tree ElementNode>, ExpressionApplyError> {
        let item = element.to_item(tree);

        self.find_elements_from_item(tree, item)
    }
}

/// Error that occurs when using a function that both parses a string into an [Xpath](crate::xpath::Xpath) expression and immediately applies it to a [XpathItemTree].
#[derive(PartialEq, Debug, Error)]
pub enum ParseApplyError {
    /// Error parsing the xpath expression.
    #[error("Failed to parse xpath: {0}")]
    ParseError(#[from] ExpressionParseError),

    /// Error applying the xpath expression.
    #[error("Failed to apply xpath: {0}")]
    ApplyError(#[from] ExpressionApplyError),

    /// An assumption made by the function was incorrect.
    ///
    /// This probably means the xpath expression does not return the expected type of item.
    /// Try modifying the xpath expression, or using a different function.
    #[error("Assumption error: {0}")]
    AssumptionError(String),
}

/// Find items in an [XpathItemTree] using an xpath expression.
///
/// **Note**: If the same xpath expression is being used multiple times, consider using the [xpath::parse] function to parse the expression once,
/// and then using the [Xpath::find] method on the parsed expression.
/// Parsing xpath expression takes effort, so it's best to do it once and then reuse the parsed expression.
///
/// # Example
///
/// ```rust
/// use skyscraper::html;
/// use skyscraper::xpath::{XpathItemTree, query::find, grammar::data_model::ElementNode};
///
/// let html = html::parse("<html><body><div>Example 1</div><div>Example 2</div></body></html>").unwrap();
/// let tree = XpathItemTree::from(&html);
///
/// let items = find(&tree, "//div").unwrap();
///
/// let nodes: Vec<&ElementNode> = items
///     .into_iter()
///     .map(|item| item.extract_as_node().extract_as_element_node())
///     .collect();
/// ```
pub fn find<'tree>(
    tree: &'tree XpathItemTree,
    xpath: &str,
) -> Result<XpathItemSet<'tree>, ParseApplyError> {
    let xpath = xpath::parse(xpath)?;

    Ok(xpath.apply(tree)?)
}

/// Find attributes in an [XpathItemTree] using an xpath expression.
///
/// **Note**: If the same xpath expression is being used multiple times, consider using the [xpath::parse] function to parse the expression once,
/// and then using the [Xpath::find_attributes] method on the parsed expression.
/// Parsing xpath expression takes effort, so it's best to do it once and then reuse the parsed expression.
///
/// # Example
///
/// ```rust
/// use skyscraper::html;
/// use skyscraper::xpath::{XpathItemTree, query::find_attributes, grammar::data_model::AttributeNode};
///
/// let html = html::parse("<html><body><div id=\"example\">Example 1</div></body></html>").unwrap();
/// let tree = XpathItemTree::from(&html);
///
/// let attributes = find_attributes(&tree, "//div/@id").unwrap();
///
/// let attribute = attributes[0];
///
/// assert_eq!(attribute.name, "id");
/// assert_eq!(attribute.value, "example");
/// ```
pub fn find_attributes<'tree>(
    tree: &'tree XpathItemTree,
    xpath: &str,
) -> Result<Vec<&'tree AttributeNode>, ParseApplyError> {
    let xpath = xpath::parse(xpath)?;

    Ok(xpath.find_attributes(tree)?)
}

/// Find elements in an [XpathItemTree] using an xpath expression.
///
/// **Note**: If the same xpath expression is being used multiple times, consider using the [xpath::parse] function to parse the expression once,
/// and then using the [Xpath::find_elements] method on the parsed expression.
/// Parsing xpath expression takes effort, so it's best to do it once and then reuse the parsed expression.
///
/// # Example
///
/// ```rust
/// use skyscraper::html;
/// use skyscraper::xpath::{XpathItemTree, query::find_elements, grammar::data_model::ElementNode};
///
/// let html = html::parse("<html><body><div id=\"example\">Example 1</div></body></html>").unwrap();
/// let tree = XpathItemTree::from(&html);
///
/// let elements = find_elements(&tree, "//div").unwrap();
///
/// let element = elements[0];
///
/// assert_eq!(element.name, "div");
/// ```
pub fn find_elements<'tree>(
    tree: &'tree XpathItemTree,
    xpath: &str,
) -> Result<Vec<&'tree ElementNode>, ParseApplyError> {
    let xpath = xpath::parse(xpath)?;

    Ok(xpath.find_elements(tree)?)
}

#[cfg(test)]
mod tests {
    use crate::html;

    use super::*;

    #[test]
    fn find_attributes_should_find_attribute() {
        // arrange
        let html = r#"<html><body><div id="example">Example 1</div></body></html>"#;
        let tree = html::parse(html).unwrap();

        // act
        let attributes = find_attributes(&tree, "//div/@id").unwrap();

        // assert
        let attribute = attributes[0];

        assert_eq!(attribute.name, "id");
        assert_eq!(attribute.value, "example");
    }

    #[test]
    fn find_elements_should_find_element() {
        // arrange
        let html = r#"<html><body><div id="example">Example 1</div></body></html>"#;
        let tree = html::parse(html).unwrap();

        // act
        let elements = find_elements(&tree, "//div").unwrap();

        // assert
        let element = elements[0];

        assert_eq!(element.name, "div");
    }

    #[test]
    fn find_elements_from_item_should_find_element() {
        // arrange
        let html = r#"<html><body><div id="example">Example 1</div></body></html>"#;
        let tree = html::parse(html).unwrap();

        let items = find(&tree, "//body").unwrap();

        let xpath = xpath::parse("//div").unwrap();

        // act
        let elements = xpath
            .find_elements_from_item(&tree, items[0].clone())
            .unwrap();

        // assert
        let element = elements[0];

        assert_eq!(element.name, "div");
    }

    #[test]
    fn find_elements_from_element_should_find_element() {
        // arrange
        let html = r#"<html><body><div id="example">Example 1</div></body></html>"#;
        let tree = html::parse(html).unwrap();

        let first_elements = find_elements(&tree, "//body").unwrap();

        let xpath = xpath::parse("//div").unwrap();

        // act
        let elements = xpath
            .find_elements_from_element(&tree, first_elements[0])
            .unwrap();

        // assert
        let element = elements[0];

        assert_eq!(element.name, "div");
    }
}
