//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-types>

use std::fmt::Display;

use indexmap::IndexSet;
use nom::{
    branch::alt, bytes::complete::tag, character::complete::char, combinator::opt, error::context,
    sequence::tuple,
};

use crate::xpath::{
    grammar::{
        terminal_symbols::{string_literal, uri_qualified_name},
        whitespace_recipes::ws,
        xml_names::qname,
    },
    xpath_item_set::XpathItemSet,
    ExpressionApplyError,
};

use self::{
    attribute_test::AttributeTest,
    common::{attribute_name, type_name, AttributeName, TypeName},
    element_test::ElementTest,
    schema_element_test::SchemaElementTest,
};

use super::{
    data_model::XpathItem,
    recipes::Res,
    terminal_symbols::UriQualifiedName,
    xml_names::{nc_name, QName},
    XpathItemTreeNode,
};

pub mod array_test;
pub mod attribute_test;
pub mod common;
pub mod element_test;
pub mod function_test;
pub mod map_test;
pub mod schema_element_test;
pub mod sequence_type;

pub fn kind_test(input: &str) -> Res<&str, KindTest> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-KindTest

    fn any_kind_test(input: &str) -> Res<&str, KindTest> {
        // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-AnyKindTest

        ws((tag("node"), char('('), char(')')))(input)
            .map(|(next_input, _res)| (next_input, KindTest::AnyKindTest))
    }

    fn text_test(input: &str) -> Res<&str, KindTest> {
        // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-TextTest

        ws((tag("text"), char('('), char(')')))(input)
            .map(|(next_input, _res)| (next_input, KindTest::TextTest))
    }

    fn comment_test(input: &str) -> Res<&str, KindTest> {
        // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-CommentTest

        ws((tag("comment"), char('('), char(')')))(input)
            .map(|(next_input, _res)| (next_input, KindTest::CommentTest))
    }

    fn namespace_node_test(input: &str) -> Res<&str, KindTest> {
        // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-NamespaceNodeTest

        ws((tag("namespace-node"), char('('), char(')')))(input)
            .map(|(next_input, _res)| (next_input, KindTest::NamespaceNodeTest))
    }

    fn document_test_map(input: &str) -> Res<&str, KindTest> {
        document_test(input).map(|(next_input, res)| (next_input, KindTest::DocumentTest(res)))
    }

    fn element_test_map(input: &str) -> Res<&str, KindTest> {
        element_test::element_test(input)
            .map(|(next_input, res)| (next_input, KindTest::ElementTest(res)))
    }

    fn attribute_test_map(input: &str) -> Res<&str, KindTest> {
        attribute_test::attribute_test(input)
            .map(|(next_input, res)| (next_input, KindTest::AttributeTest(res)))
    }

    fn schema_element_test_map(input: &str) -> Res<&str, KindTest> {
        schema_element_test::schema_element_test(input)
            .map(|(next_input, res)| (next_input, KindTest::SchemaElementTest(res)))
    }

    fn schema_attribute_test_map(input: &str) -> Res<&str, KindTest> {
        schema_attribute_test(input)
            .map(|(next_input, res)| (next_input, KindTest::SchemaAttributeTest(res)))
    }

    fn pi_test_map(input: &str) -> Res<&str, KindTest> {
        pi_test(input).map(|(next_input, res)| (next_input, KindTest::PITest(res)))
    }

    context(
        "kind_test",
        alt((
            document_test_map,
            element_test_map,
            attribute_test_map,
            schema_element_test_map,
            schema_attribute_test_map,
            pi_test_map,
            comment_test,
            text_test,
            namespace_node_test,
            any_kind_test,
        )),
    )(input)
}

#[derive(PartialEq, Debug, Clone)]
pub enum KindTest {
    AnyKindTest,
    TextTest,
    CommentTest,
    NamespaceNodeTest,
    DocumentTest(DocumentTest),
    ElementTest(ElementTest),
    AttributeTest(AttributeTest),
    SchemaElementTest(SchemaElementTest),
    SchemaAttributeTest(SchemaAttributeTest),
    PITest(PITest),
}

impl KindTest {
    pub(crate) fn filter<'tree>(
        &self,
        item_set: &XpathItemSet<'tree>,
    ) -> Result<IndexSet<&'tree XpathItemTreeNode>, ExpressionApplyError> {
        match self {
            KindTest::AnyKindTest => {
                // AnyKindTest is `node()`.
                // Select all node types.
                let filtered_nodes = item_set.iter().filter_map(|item| {
                    if let XpathItem::Node(node) = item {
                        if let XpathItemTreeNode::AttributeNode(_) = node {
                            None
                        } else {
                            Some(*node)
                        }
                    } else {
                        None
                    }
                });

                Ok(filtered_nodes.collect())
            }
            KindTest::TextTest => {
                // TextTest is `text()`.
                // Select all text nodes.
                let filtered_nodes = item_set.iter().filter_map(|item| {
                    if let XpathItem::Node(node) = item {
                        if node.is_text_node() {
                            return Some(*node);
                        }
                    }

                    None
                });

                Ok(filtered_nodes.collect())
            }
            KindTest::CommentTest => todo!("KindTest::CommentTest::is_match"),
            KindTest::NamespaceNodeTest => todo!("KindTest::NamespaceNodeTest::is_match"),
            KindTest::DocumentTest(x) => x.filter(item_set),
            KindTest::ElementTest(_) => todo!("KindTest::ElementTest::is_match"),
            KindTest::AttributeTest(x) => {
                let mut filtered_nodes = IndexSet::new();

                for item in item_set {
                    if let XpathItem::Node(node) = item {
                        if x.is_match(node)? {
                            filtered_nodes.insert(*node);
                        }
                    }
                }

                Ok(filtered_nodes)
            }
            KindTest::SchemaElementTest(_) => todo!("KindTest::SchemaElementTest::is_match"),
            KindTest::SchemaAttributeTest(_) => todo!("KindTest::SchemaAttributeTest::is_match"),
            KindTest::PITest(_) => todo!("KindTest::PITest::is_match"),
        }
    }
}

impl Display for KindTest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KindTest::AnyKindTest => write!(f, "node()"),
            KindTest::TextTest => write!(f, "text()"),
            KindTest::CommentTest => write!(f, "comment()"),
            KindTest::NamespaceNodeTest => write!(f, "namespace-node()"),
            KindTest::DocumentTest(x) => write!(f, "{}", x),
            KindTest::ElementTest(x) => write!(f, "{}", x),
            KindTest::AttributeTest(x) => write!(f, "{}", x),
            KindTest::SchemaElementTest(x) => write!(f, "{}", x),
            KindTest::SchemaAttributeTest(x) => write!(f, "{}", x),
            KindTest::PITest(x) => write!(f, "{}", x),
        }
    }
}

pub fn document_test(input: &str) -> Res<&str, DocumentTest> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-DocumentTest

    fn element_test_map(input: &str) -> Res<&str, DocumentTestValue> {
        element_test::element_test(input)
            .map(|(next_input, res)| (next_input, DocumentTestValue::ElementTest(res)))
    }

    fn schema_element_test_map(input: &str) -> Res<&str, DocumentTestValue> {
        schema_element_test::schema_element_test(input)
            .map(|(next_input, res)| (next_input, DocumentTestValue::SchemaElementTest(res)))
    }

    context(
        "document_test",
        tuple((
            tag("document-node"),
            char('('),
            opt(alt((element_test_map, schema_element_test_map))),
            char(')'),
        )),
    )(input)
    .map(|(next_input, res)| (next_input, DocumentTest { value: res.2 }))
}

#[derive(PartialEq, Debug, Clone)]
pub struct DocumentTest {
    pub value: Option<DocumentTestValue>,
}

impl Display for DocumentTest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "document-node(")?;
        if let Some(x) = &self.value {
            write!(f, "{}", x)?;
        }
        write!(f, ")")
    }
}

impl DocumentTest {
    pub(crate) fn filter<'tree>(
        &self,
        item_set: &XpathItemSet<'tree>,
    ) -> Result<IndexSet<&'tree XpathItemTreeNode>, ExpressionApplyError> {
        match &self.value {
            // document-node() matches any document node.
            None => {
                let mut filtered_nodes = IndexSet::new();

                for item in item_set {
                    if let XpathItem::Node(node) = item {
                        if matches!(node, XpathItemTreeNode::DocumentNode(_),) {
                            filtered_nodes.insert(*node);
                        }
                    }
                }

                Ok(filtered_nodes)
            }
            // document-node( E ) matches any document node that contains exactly one element node,
            // optionally accompanied by one or more comment and processing instruction nodes,
            // if E is an ElementTest or SchemaElementTest that matches the element node.
            Some(_) => todo!("DocumentTest::is_match value"),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum DocumentTestValue {
    ElementTest(ElementTest),
    SchemaElementTest(SchemaElementTest),
}

impl Display for DocumentTestValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentTestValue::ElementTest(x) => write!(f, "{}", x),
            DocumentTestValue::SchemaElementTest(x) => write!(f, "{}", x),
        }
    }
}

pub fn schema_attribute_test(input: &str) -> Res<&str, SchemaAttributeTest> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-SchemaAttributeTest

    context(
        "schema_attribute_test",
        tuple((
            tag("schema-attribute"),
            char('('),
            attribute_declaration,
            char(')'),
        )),
    )(input)
    .map(|(next_input, res)| (next_input, SchemaAttributeTest(res.2)))
}

#[derive(PartialEq, Debug, Clone)]
pub struct SchemaAttributeTest(pub AttributeDeclaration);

impl Display for SchemaAttributeTest {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("fmt SchemaAttributeTest")
    }
}

pub fn attribute_declaration(input: &str) -> Res<&str, AttributeDeclaration> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-AttributeDeclaration

    context("attribute_declaration", attribute_name)(input)
        .map(|(next_input, res)| (next_input, AttributeDeclaration(res)))
}

#[derive(PartialEq, Debug, Clone)]
pub struct AttributeDeclaration(pub AttributeName);

pub fn pi_test(input: &str) -> Res<&str, PITest> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-PITest

    fn nc_name_map(input: &str) -> Res<&str, PITestValue> {
        nc_name(input).map(|(next_input, res)| (next_input, PITestValue::NCName(res.to_string())))
    }

    fn string_literal_map(input: &str) -> Res<&str, PITestValue> {
        string_literal(input)
            .map(|(next_input, res)| (next_input, PITestValue::StringLiteral(res.to_string())))
    }

    context(
        "pi_test",
        tuple((
            tag("processing-instruction"),
            char('('),
            opt(alt((nc_name_map, string_literal_map))),
            char(')'),
        )),
    )(input)
    .map(|(next_input, res)| (next_input, PITest { val: res.2 }))
}

#[derive(PartialEq, Debug, Clone)]
pub struct PITest {
    pub val: Option<PITestValue>,
}

impl Display for PITest {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("fmt PITest")
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum PITestValue {
    NCName(String),
    StringLiteral(String),
}

#[derive(PartialEq, Debug, Clone)]
pub struct AtomicOrUnionType(EQName);

impl Display for AtomicOrUnionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn simple_type_name(input: &str) -> Res<&str, SimpleTypeName> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-SimpleTypeName
    context("simple_type_name", type_name)(input)
        .map(|(next_input, res)| (next_input, SimpleTypeName(res)))
}

#[derive(PartialEq, Debug, Clone)]
pub struct SimpleTypeName(TypeName);

impl Display for SimpleTypeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn eq_name(input: &str) -> Res<&str, EQName> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-EQName

    fn qname_map(input: &str) -> Res<&str, EQName> {
        qname(input).map(|(next_input, res)| (next_input, EQName::QName(res)))
    }

    fn uri_qualified_name_map(input: &str) -> Res<&str, EQName> {
        uri_qualified_name(input)
            .map(|(next_input, res)| (next_input, EQName::UriQualifiedName(res)))
    }

    context("eq_name", alt((uri_qualified_name_map, qname_map)))(input)
}

#[derive(PartialEq, Debug, Clone)]
pub enum EQName {
    QName(QName),
    UriQualifiedName(UriQualifiedName),
}

impl Display for EQName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EQName::QName(x) => write!(f, "{}", x),
            EQName::UriQualifiedName(x) => write!(f, "{}", x),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::xpath::grammar::xml_names::PrefixedName;

    use super::*;

    #[test]
    fn eq_name_example1() {
        // arrange
        let input = "pi";

        // act
        let res = eq_name(input);

        // assert
        assert_eq!(
            res,
            Ok(("", EQName::QName(QName::UnprefixedName(String::from("pi")))))
        )
    }

    #[test]
    fn eq_name_example2() {
        // arrange
        let input = "math:pi";

        // act
        let res = eq_name(input);

        // assert
        assert_eq!(
            res,
            Ok((
                "",
                EQName::QName(QName::PrefixedName(PrefixedName {
                    prefix: String::from("math"),
                    local_part: String::from("pi")
                }))
            ))
        )
    }

    #[test]
    fn eq_name_example3() {
        // arrange
        let input = "Q{http://www.w3.org/2005/xpath-functions/math}pi";

        // act
        let res = eq_name(input);

        // assert
        assert_eq!(
            res,
            Ok((
                "",
                EQName::UriQualifiedName(UriQualifiedName {
                    uri: String::from("http://www.w3.org/2005/xpath-functions/math"),
                    name: String::from("pi")
                })
            ))
        )
    }
}
