//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#node-tests>

use std::fmt::Display;

use nom::{
    branch::alt, bytes::complete::tag, character::complete::char, error::context, sequence::tuple,
};

use crate::{
    xpath::{
        grammar::{
            data_model::XpathItem,
            recipes::Res,
            terminal_symbols::braced_uri_literal,
            types::{eq_name, kind_test, EQName, KindTest},
            xml_names::{nc_name, QName},
            XpathItemTreeNode,
        },
        ExpressionApplyError, XpathExpressionContext,
    },
    xpath_item_set,
};

use super::axes::{forward_axis::ForwardAxis, reverse_axis::ReverseAxis};

pub fn node_test(input: &str) -> Res<&str, NodeTest> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-NodeTest

    fn kind_test_map(input: &str) -> Res<&str, NodeTest> {
        kind_test(input).map(|(next_input, res)| (next_input, NodeTest::KindTest(res)))
    }

    fn name_test_map(input: &str) -> Res<&str, NodeTest> {
        name_test(input).map(|(next_input, res)| (next_input, NodeTest::NameTest(res)))
    }

    context("node_test", alt((kind_test_map, name_test_map)))(input)
}

#[derive(PartialEq, Debug, Clone)]
pub enum NodeTest {
    KindTest(KindTest),
    NameTest(NameTest),
}

impl Display for NodeTest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeTest::KindTest(x) => write!(f, "{}", x),
            NodeTest::NameTest(x) => write!(f, "{}", x),
        }
    }
}

impl NodeTest {
    pub(crate) fn eval<'tree>(
        &self,
        axis: BiDirectionalAxis,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<Option<&'tree XpathItemTreeNode>, ExpressionApplyError> {
        match self {
            NodeTest::KindTest(test) => {
                let filtered_nodes = test.filter(&xpath_item_set![context.item.clone()])?;

                if !filtered_nodes.is_empty() {
                    let node: &'tree XpathItemTreeNode = filtered_nodes.into_iter().next().unwrap();
                    Ok(Some(node))
                } else {
                    Ok(None)
                }
            }
            NodeTest::NameTest(test) => test.eval(axis, context),
        }
    }
}

fn name_test(input: &str) -> Res<&str, NameTest> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-NameTest

    fn eq_name_map(input: &str) -> Res<&str, NameTest> {
        eq_name(input).map(|(next_input, res)| (next_input, NameTest::Name(res)))
    }

    fn wildcard_map(input: &str) -> Res<&str, NameTest> {
        wildcard(input).map(|(next_input, res)| (next_input, NameTest::Wildcard(res)))
    }

    context("name_test", alt((eq_name_map, wildcard_map)))(input)
}

#[derive(PartialEq, Debug, Clone)]
pub enum NameTest {
    Name(EQName),
    Wildcard(Wildcard),
}

impl Display for NameTest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NameTest::Name(x) => write!(f, "{}", x),
            NameTest::Wildcard(x) => write!(f, "{}", x),
        }
    }
}

pub(crate) enum BiDirectionalAxis {
    ForwardAxis(ForwardAxis),
    ReverseAxis(ReverseAxis),
}

impl NameTest {
    pub(crate) fn eval<'tree>(
        &self,
        axis: BiDirectionalAxis,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<Option<&'tree XpathItemTreeNode>, ExpressionApplyError> {
        let node = if let XpathItem::Node(node) = &context.item {
            node
        } else {
            todo!("NameTest::eval non-node");
        };

        let is_match = match self {
            NameTest::Name(expected_name) => {
                // Get the name of the node, if available for the node type.
                let node_name = match node {
                    XpathItemTreeNode::DocumentNode(_) => None,
                    XpathItemTreeNode::ElementNode(e) => Some(&e.name),
                    XpathItemTreeNode::PINode(_) => None,
                    XpathItemTreeNode::CommentNode(_) => None,
                    XpathItemTreeNode::TextNode(_) => None, // Text nodes do not have a name.
                    XpathItemTreeNode::AttributeNode(a) => Some(&a.name),
                };

                match node_name {
                    Some(node_name) => match expected_name {
                        EQName::QName(qname) => match qname {
                            QName::PrefixedName(_) => todo!("NameTest::is_match PrefixedName"),
                            QName::UnprefixedName(unprefixed_name) => unprefixed_name == node_name,
                        },
                        EQName::UriQualifiedName(_) => todo!("NameTest::is_match UriQualifiedName"),
                    },

                    // Name tests need a name to match.
                    // If the node does not have a name, it cannot match.
                    None => false,
                }
            }
            NameTest::Wildcard(wildcard) => wildcard.is_match(axis, node),
        };

        if is_match {
            Ok(Some(*node))
        } else {
            Ok(None)
        }
    }
}

fn wildcard(input: &str) -> Res<&str, Wildcard> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-Wildcard
    // ws: explicit

    fn prefixed_name_map(input: &str) -> Res<&str, Wildcard> {
        tuple((tag("*:"), nc_name))(input)
            .map(|(next_input, res)| (next_input, Wildcard::PrefixedName(res.1.to_string())))
    }

    fn suffixed_name_map(input: &str) -> Res<&str, Wildcard> {
        tuple((nc_name, tag(":*")))(input)
            .map(|(next_input, res)| (next_input, Wildcard::SuffixedName(res.0.to_string())))
    }

    fn braced_uri_map(input: &str) -> Res<&str, Wildcard> {
        tuple((braced_uri_literal, char('*')))(input)
            .map(|(next_input, res)| (next_input, Wildcard::BracedUri(res.0.to_string())))
    }

    fn simple_map(input: &str) -> Res<&str, Wildcard> {
        tuple((char('*'),))(input).map(|(next_input, _res)| (next_input, Wildcard::Simple))
    }

    context(
        "wildcard",
        alt((
            prefixed_name_map,
            suffixed_name_map,
            braced_uri_map,
            simple_map,
        )),
    )(input)
}

#[derive(PartialEq, Debug, Clone)]
pub enum Wildcard {
    Simple,
    PrefixedName(String),
    SuffixedName(String),
    BracedUri(String),
}

impl Display for Wildcard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Wildcard::Simple => write!(f, "*"),
            Wildcard::PrefixedName(x) => write!(f, "*:{}", x),
            Wildcard::SuffixedName(x) => write!(f, "{}:*", x),
            Wildcard::BracedUri(x) => write!(f, "Q{{{}}}*", x),
        }
    }
}

impl Wildcard {
    pub(crate) fn is_match<'tree>(
        &self,
        axis: BiDirectionalAxis,
        node: &'tree XpathItemTreeNode,
    ) -> bool {
        // Wildcards only match context items that are the axis' principal node kind.
        // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#dt-principal-node-kind
        let is_principal_node_kind = match axis {
            BiDirectionalAxis::ForwardAxis(ForwardAxis::Attribute) => {
                // For the attribute axis, the principal node kind is attribute.
                matches!(node, XpathItemTreeNode::AttributeNode(_))
            }
            _ => {
                // For all other axes, the principal node kind is element.
                matches!(node, XpathItemTreeNode::ElementNode(_),)
            }
        };

        if !is_principal_node_kind {
            return false;
        }

        match self {
            Wildcard::Simple => true,
            Wildcard::PrefixedName(_) => todo!("Wildcard::is_match PrefixedName"),
            Wildcard::SuffixedName(_) => todo!("Wildcard::is_match SuffixedName"),
            Wildcard::BracedUri(_) => todo!("Wildcard::is_match BracedUri"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wildcard_should_parse_simple() {
        // arrange
        let input = "*";

        // act
        let (next_input, res) = wildcard(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }

    #[test]
    fn wildcard_should_parse_prefixed_name() {
        // arrange
        let input = "*:foo";

        // act
        let (next_input, res) = wildcard(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "*:foo");
    }

    #[test]
    fn wildcard_should_parse_suffixed_name() {
        // arrange
        let input = "foo:*";

        // act
        let (next_input, res) = wildcard(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), "foo:*");
    }

    #[test]
    fn wildcard_should_parse_braced_uri() {
        // arrange
        let input = "Q{http://example.com/ns}*";

        // act
        let (next_input, res) = wildcard(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }
}
