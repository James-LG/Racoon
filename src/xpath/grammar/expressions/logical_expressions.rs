//! https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-logical-expressions

use std::fmt::Display;

use nom::{bytes::complete::tag, multi::many0, sequence::tuple};

use crate::xpath::grammar::recipes::Res;

use super::comparison_expressions::{comparison_expr, ComparisonExpr};

pub fn or_expr(input: &str) -> Res<&str, OrExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-OrExpr
    tuple((and_expr, many0(tuple((tag("or"), and_expr)))))(input).map(|(next_input, res)| {
        let items = res.1.into_iter().map(|res| res.1).collect();
        (next_input, OrExpr { expr: res.0, items })
    })
}

#[derive(PartialEq, Debug)]
pub struct OrExpr {
    pub expr: AndExpr,
    pub items: Vec<AndExpr>,
}

impl Display for OrExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expr)?;
        for x in &self.items {
            write!(f, " or {}", x)?;
        }

        Ok(())
    }
}

fn and_expr(input: &str) -> Res<&str, AndExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-AndExpr

    tuple((comparison_expr, many0(tuple((tag("and"), comparison_expr)))))(input).map(
        |(next_input, res)| {
            let items = res.1.into_iter().map(|res| res.1).collect();
            (next_input, AndExpr { expr: res.0, items })
        },
    )
}

#[derive(PartialEq, Debug)]
pub struct AndExpr {
    pub expr: ComparisonExpr,
    pub items: Vec<ComparisonExpr>,
}

impl Display for AndExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expr)?;
        for x in &self.items {
            write!(f, " and {}", x)?;
        }

        Ok(())
    }
}