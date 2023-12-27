//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-context-item-expression

use std::fmt::Display;

use nom::{error::context, sequence::tuple};

use crate::{
    xpath::{
        grammar::{
            data_model::{Node, XpathItem},
            expressions::common::{argument_list, ArgumentList},
            recipes::Res,
            types::{eq_name, EQName},
            xml_names::QName,
        },
        xpath_item_set::XpathItemSet,
        ExpressionApplyError, XpathExpressionContext,
    },
    xpath_item_set,
};

pub fn function_call(input: &str) -> Res<&str, FunctionCall> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-FunctionCall

    context("function_call", tuple((eq_name, argument_list)))(input).map(|(next_input, res)| {
        (
            next_input,
            FunctionCall {
                name: res.0,
                argument_list: res.1,
            },
        )
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct FunctionCall {
    pub name: EQName,
    pub argument_list: ArgumentList,
}

impl Display for FunctionCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.name, self.argument_list)
    }
}

impl FunctionCall {
    pub(crate) fn eval<'tree>(
        &self,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        match &self.name {
            EQName::QName(qname) => match qname {
                QName::PrefixedName(prefixed_name) => {
                    if prefixed_name.prefix == "fn" {
                        // Root function selects the root node of the tree.
                        if prefixed_name.local_part == "root" {
                            return Ok(xpath_item_set![XpathItem::Node(Node::TreeNode(
                                context.item_tree.root(),
                            ))]);
                        }
                    }

                    Err(ExpressionApplyError {
                        msg: format!("Unknown function {}", self.name.to_string()),
                    })
                }
                QName::UnprefixedName(_) => todo!("FunctionCall::eval UnprefixedName"),
            },
            EQName::UriQualifiedName(_) => todo!("FunctionCall::eval UriQualifiedName"),
        }
    }
}
