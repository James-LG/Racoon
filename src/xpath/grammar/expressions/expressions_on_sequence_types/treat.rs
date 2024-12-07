//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-treat>

use std::fmt::Display;

use nom::{bytes::complete::tag, combinator::opt, error::context};

use crate::xpath::{
    grammar::{
        recipes::Res,
        types::sequence_type::{sequence_type, SequenceType},
        whitespace_recipes::sep,
    },
    xpath_item_set::XpathItemSet,
    ExpressionApplyError, XpathExpressionContext,
};

use super::castable::{castable_expr, CastableExpr};

pub fn treat_expr(input: &str) -> Res<&str, TreatExpr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-TreatExpr

    context(
        "treat_expr",
        sep((
            castable_expr,
            opt(sep((tag("treat"), tag("as"), sequence_type))),
        )),
    )(input)
    .map(|(next_input, res)| {
        let treat_type = res.1.map(|res| res.2);
        (
            next_input,
            TreatExpr {
                expr: res.0,
                treat_type,
            },
        )
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct TreatExpr {
    pub expr: CastableExpr,
    pub treat_type: Option<SequenceType>,
}

impl Display for TreatExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expr)?;
        if let Some(x) = &self.treat_type {
            write!(f, " treat as {}", x)?;
        }

        Ok(())
    }
}

impl TreatExpr {
    pub(crate) fn eval<'tree>(
        &self,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        // Evaluate the first expression.
        let result = self.expr.eval(context)?;

        let treat_type = match &self.treat_type {
            // If there's only one parameter, return it's eval.
            None => return Ok(result),
            // Otherwise, do the operation.
            Some(treat_type) => treat_type,
        };

        if !treat_type.is_match(&result)? {
            return Err(ExpressionApplyError {
                msg: format!(
                    "err:XPDY0050 Cannot treat {:?} as {}",
                    result,
                    treat_type.to_string()
                ),
            });
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn treat_expr_should_parse() {
        // arrange
        let input = "fn:root() treat as integer";

        // act
        let (next_input, res) = treat_expr(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }
}
