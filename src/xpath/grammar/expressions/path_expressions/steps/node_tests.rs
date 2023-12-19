//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#node-tests

use std::fmt::Display;

use nom::{
    branch::alt, bytes::complete::tag, character::complete::char, error::context, sequence::tuple,
};

use crate::xpath::grammar::{
    data_model::{Node, XpathItem},
    recipes::Res,
    terminal_symbols::braced_uri_literal,
    types::{eq_name, kind_test, EQName, KindTest},
    xml_names::{nc_name, QName},
    XpathItemTreeNodeData,
};

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
    pub(crate) fn is_match(&self, node: &Node) -> bool {
        match self {
            NodeTest::KindTest(test) => test.is_match(&XpathItem::Node(node.clone())),
            NodeTest::NameTest(test) => test.is_match(node),
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

impl NameTest {
    pub(crate) fn is_match(&self, node: &Node) -> bool {
        // Name test only works on element nodes
        let element = if let Node::TreeNode(tree_node) = node {
            if let XpathItemTreeNodeData::ElementNode(element) = &tree_node.data {
                element
            } else {
                return false;
            }
        } else {
            return false;
        };

        match self {
            NameTest::Name(name) => match name {
                EQName::QName(qname) => match qname {
                    QName::PrefixedName(_) => todo!("NameTest::is_match PrefixedName"),
                    QName::UnprefixedName(unprefixed_name) => unprefixed_name == &element.name,
                },
                EQName::UriQualifiedName(_) => todo!("NameTest::is_match UriQualifiedName"),
            },
            NameTest::Wildcard(wildcard) => wildcard.is_match(node),
        }
    }
}

fn wildcard(input: &str) -> Res<&str, Wildcard> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-Wildcard

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
        char('*')(input).map(|(next_input, _res)| (next_input, Wildcard::Simple))
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
            Wildcard::BracedUri(x) => write!(f, "{}*", x),
        }
    }
}

impl Wildcard {
    pub(crate) fn is_match(&self, node: &Node) -> bool {
        match self {
            Wildcard::Simple => true,
            Wildcard::PrefixedName(_) => todo!("Wildcard::is_match PrefixedName"),
            Wildcard::SuffixedName(_) => todo!("Wildcard::is_match SuffixedName"),
            Wildcard::BracedUri(_) => todo!("Wildcard::is_match BracedUri"),
        }
    }
}
