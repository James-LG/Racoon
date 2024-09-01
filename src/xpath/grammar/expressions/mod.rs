//! <https://www.w3.org/TR/2017/REC-xpath-31-20170321/#id-expressions>

use std::fmt::Display;

use nom::{character::complete::char, error::context, multi::many0, sequence::tuple};

use crate::xpath::{
    grammar::{
        expressions::{
            conditional_expressions::if_expr, for_expressions::for_expr, let_expressions::let_expr,
            logical_expressions::or_expr, quantified_expressions::quantified_expr,
        },
        recipes::max,
    },
    ExpressionApplyError, XpathExpressionContext, XpathItemSet, XpathItemTree,
};

use self::{
    conditional_expressions::IfExpr, for_expressions::ForExpr, let_expressions::LetExpr,
    logical_expressions::OrExpr, quantified_expressions::QuantifiedExpr,
};

use super::{
    data_model::{ElementNode, XpathItem},
    recipes::Res,
};

pub mod arithmetic_expressions;
pub mod arrow_operator;
pub mod common;
pub mod comparison_expressions;
pub mod conditional_expressions;
pub mod expressions_on_sequence_types;
pub mod for_expressions;
pub mod let_expressions;
pub mod logical_expressions;
pub mod maps_and_arrays;
pub mod path_expressions;
pub mod postfix_expressions;
pub mod primary_expressions;
pub mod quantified_expressions;
pub mod sequence_expressions;
pub mod simple_map_operator;
pub mod string_concat_expressions;

pub(crate) fn xpath(input: &str) -> Res<&str, Xpath> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#doc-xpath31-XPath

    context("xpath", expr)(input).map(|(next_input, res)| (next_input, Xpath(res)))
}

/// An XPath expression.
#[derive(PartialEq, Debug, Clone)]
pub struct Xpath(pub Expr);

impl Display for Xpath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Xpath {
    pub(crate) fn eval<'tree>(
        &self,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        self.0.eval(context)
    }

    /// Apply the XPath expression to the given item tree.
    ///
    /// # Arguments
    ///
    /// * `item_tree` - The item tree to apply the expression to.
    ///
    /// # Returns
    ///
    /// The result of applying the expression to the item tree.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use skyscraper::html;
    /// use skyscraper::xpath::{self, XpathItemTree, grammar::{XpathItemTreeNode, data_model::XpathItem}};
    /// use std::error::Error;
    ///
    /// fn main() -> Result<(), Box<dyn Error>> {
    ///     let html_text = r##"
    ///     <html>
    ///         <body>
    ///             <div>Hello world</div>
    ///         </body>
    ///     </html>"##;
    ///
    ///     let document = html::parse(html_text)?;
    ///     let xpath_item_tree = XpathItemTree::from(&document);
    ///     let xpath = xpath::parse("//div")?;
    ///    
    ///     let items = xpath.apply(&xpath_item_tree)?;
    ///    
    ///     assert_eq!(items.len(), 1);
    ///    
    ///     let mut items = items.into_iter();
    ///    
    ///     let item = items
    ///         .next()
    ///         .unwrap();
    ///
    ///     let element = item
    ///         .as_node()?
    ///         .as_element_node()?;
    ///
    ///     assert_eq!(element.name, "div");
    ///     Ok(())
    /// }
    /// ```
    pub fn apply<'tree>(
        &self,
        item_tree: &'tree XpathItemTree,
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        let context =
            XpathExpressionContext::new_single(item_tree, XpathItem::Node(item_tree.root()), true);
        let mut item_set = self.eval(&context)?;
        item_set.sort();
        Ok(item_set)
    }

    /// Apply the XPath expression to the given item.
    /// The expression will be evaluated relative to the given item.
    ///
    /// # Arguments
    ///
    /// * `item_tree` - The item tree.
    /// * `item` - The item to apply the expression to.
    ///
    /// # Returns
    ///
    /// The result of applying the expression to the item.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use skyscraper::html::{self, trim_internal_whitespace};
    /// use skyscraper::xpath::{self, XpathItemTree, grammar::{data_model::{XpathItem}}};
    /// use std::error::Error;
    ///
    /// fn main() -> Result<(), Box<dyn Error>> {
    ///     let html_text = r##"
    ///     <html>
    ///         <body>
    ///             <div id="1"><span>Hello</span></div>
    ///             <div id="2"><span>world</span></div>
    ///         </body>
    ///     </html>"##;
    ///
    ///     let document = html::parse(html_text)?;
    ///     let xpath_item_tree = XpathItemTree::from(&document);
    ///     let xpath = xpath::parse(r#"//div[@id="2"]"#)?;
    ///    
    ///     let items = xpath.apply(&xpath_item_tree)?;
    ///    
    ///     assert_eq!(items.len(), 1);
    ///
    ///     let relative_xpath = xpath::parse("/span")?;
    ///     let items = relative_xpath.apply_to_item(&xpath_item_tree, items[0].clone())?;
    ///     let mut items = items.into_iter();
    ///
    ///     let item = items.next().unwrap();
    ///     let element = item
    ///         .as_node()?
    ///         .as_element_node()?;
    ///
    ///     assert_eq!(element.name, "span");
    ///     assert_eq!(trim_internal_whitespace(&element.text(&xpath_item_tree).unwrap()), "world");
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn apply_to_item<'tree>(
        &self,
        item_tree: &'tree XpathItemTree,
        item: XpathItem<'tree>,
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        let context = XpathExpressionContext::new_single(item_tree, item, false);
        self.eval(&context)
    }

    /// Apply the XPath expression to the given element.
    /// The expression will be evaluated relative to the given element.
    ///
    /// # Arguments
    ///
    /// * `item_tree` - The item tree.
    /// * `element` - The element to apply the expression to.
    ///
    /// # Returns
    ///
    /// The result of applying the expression to the element.
    ///
    /// # Example
    ///
    /// ```rust
    /// use skyscraper::html::{self, trim_internal_whitespace};
    /// use skyscraper::xpath::{self, XpathItemTree, grammar::{XpathItemTreeNode, data_model::{XpathItem}}};
    /// use std::error::Error;
    ///
    /// fn main() -> Result<(), Box<dyn Error>> {
    ///     let html_text = r##"
    ///     <html>
    ///         <body>
    ///             <div id="1"><span>Hello</span></div>
    ///             <div id="2"><span>world</span></div>
    ///         </body>
    ///     </html>"##;
    ///
    ///     let document = html::parse(html_text)?;
    ///     let xpath_item_tree = XpathItemTree::from(&document);
    ///     let xpath = xpath::parse(r#"//div[@id="2"]"#)?;
    ///    
    ///     let items = xpath.find_elements(&xpath_item_tree)?;
    ///    
    ///     assert_eq!(items.len(), 1);
    ///
    ///     let relative_xpath = xpath::parse("/span")?;
    ///     let items = relative_xpath.apply_to_element(&xpath_item_tree, items[0])?;
    ///     let mut items = items.into_iter();
    ///
    ///     let item = items.next().unwrap();
    ///     let element = item
    ///         .as_node()?
    ///         .as_element_node()?;
    ///
    ///     assert_eq!(element.name, "span");
    ///     assert_eq!(trim_internal_whitespace(&element.text(&xpath_item_tree).unwrap()), "world");
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn apply_to_element<'tree>(
        &self,
        item_tree: &'tree XpathItemTree,
        element: &'tree ElementNode,
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        let item = element.to_item(item_tree);
        let context = XpathExpressionContext::new_single(item_tree, item, false);
        self.eval(&context)
    }
}

pub fn expr(input: &str) -> Res<&str, Expr> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-Expr

    context(
        "expr",
        tuple((expr_single, many0(tuple((char(','), expr_single))))),
    )(input)
    .map(|(next_input, res)| {
        let items = res.1.into_iter().map(|res| res.1).collect();
        (next_input, Expr { expr: res.0, items })
    })
}

#[derive(PartialEq, Debug, Clone)]
pub struct Expr {
    pub expr: ExprSingle,
    pub items: Vec<ExprSingle>,
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expr)?;
        for x in &self.items {
            write!(f, ", {}", x)?;
        }

        Ok(())
    }
}

impl Expr {
    pub(crate) fn eval<'tree>(
        &self,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        /// Add the result of an ExprSingle to the items vector.
        ///
        /// # Arguments
        ///
        /// * `context` - The context to evaluate the expression in.
        /// * `items` - The vector to add the result to.
        /// * `expr_single` - The expression to evaluate.
        fn add_expr_single_item<'tree>(
            context: &XpathExpressionContext<'tree>,
            items: &mut XpathItemSet<'tree>,
            expr_single: &ExprSingle,
        ) -> Result<(), ExpressionApplyError> {
            // Evaluate the expression.
            let result: XpathItemSet<'tree> = expr_single.eval(context)?;

            // Add the result to the items vector.
            items.extend(result);

            Ok(())
        }

        // If there's only one parameter, return it's eval.
        if self.items.is_empty() {
            return self.expr.eval(context);
        }

        // Otherwise concatenate the results of all the expressions.
        // Expr items are separated by the comma operator, which concatenates results into a sequence.
        let mut items: XpathItemSet = XpathItemSet::new();

        // Get first item
        add_expr_single_item(context, &mut items, &self.expr)?;

        // Get remaining items
        for item in self.items.iter() {
            add_expr_single_item(context, &mut items, item)?;
        }

        Ok(items)
    }
}

pub fn expr_single(input: &str) -> Res<&str, ExprSingle> {
    // https://www.w3.org/TR/2017/REC-xpath-31-20170321/#prod-xpath31-ExprSingle

    fn for_expr_map(input: &str) -> Res<&str, ExprSingle> {
        for_expr(input).map(|(next_input, res)| (next_input, ExprSingle::ForExpr(Box::new(res))))
    }

    fn let_expr_map(input: &str) -> Res<&str, ExprSingle> {
        let_expr(input).map(|(next_input, res)| (next_input, ExprSingle::LetExpr(Box::new(res))))
    }

    fn quantified_expr_map(input: &str) -> Res<&str, ExprSingle> {
        quantified_expr(input)
            .map(|(next_input, res)| (next_input, ExprSingle::QuantifiedExpr(Box::new(res))))
    }

    fn if_expr_map(input: &str) -> Res<&str, ExprSingle> {
        if_expr(input).map(|(next_input, res)| (next_input, ExprSingle::IfExpr(Box::new(res))))
    }

    fn or_expr_map(input: &str) -> Res<&str, ExprSingle> {
        or_expr(input).map(|(next_input, res)| (next_input, ExprSingle::OrExpr(Box::new(res))))
    }

    context(
        "expr_single",
        max((
            for_expr_map,
            let_expr_map,
            quantified_expr_map,
            if_expr_map,
            or_expr_map,
        )),
    )(input)
}

#[derive(PartialEq, Debug, Clone)]
pub enum ExprSingle {
    ForExpr(Box<ForExpr>),
    LetExpr(Box<LetExpr>),
    QuantifiedExpr(Box<QuantifiedExpr>),
    IfExpr(Box<IfExpr>),
    OrExpr(Box<OrExpr>),
}

impl Display for ExprSingle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExprSingle::ForExpr(x) => write!(f, "{}", x),
            ExprSingle::LetExpr(x) => write!(f, "{}", x),
            ExprSingle::QuantifiedExpr(x) => write!(f, "{}", x),
            ExprSingle::IfExpr(x) => write!(f, "{}", x),
            ExprSingle::OrExpr(x) => write!(f, "{}", x),
        }
    }
}

impl ExprSingle {
    pub(crate) fn eval<'tree>(
        &self,
        context: &XpathExpressionContext<'tree>,
    ) -> Result<XpathItemSet<'tree>, ExpressionApplyError> {
        match self {
            ExprSingle::ForExpr(_) => todo!("ExprSingle::ForExpr"),
            ExprSingle::LetExpr(_) => todo!("ExprSingle::LetExpr"),
            ExprSingle::QuantifiedExpr(_) => todo!("ExprSingle::QuantifiedExpr"),
            ExprSingle::IfExpr(_) => todo!("ExprSingle::IfExpr"),
            ExprSingle::OrExpr(e) => e.eval(context),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expr_should_parse1() {
        // arrange
        let input = "/(chapter|appendix)";

        // act
        let (next_input, res) = expr(input).unwrap();

        // assert
        assert_eq!(res.to_string(), input);
        assert_eq!(next_input, "");
    }

    #[test]
    fn expr_should_parse2() {
        // arrange
        let input = "((book/author[.=$a])[1], book[author=$a]/title)";

        // act
        let (next_input, res) = expr(input).unwrap();

        // assert
        assert_eq!(res.to_string(), input);
        assert_eq!(next_input, "");
    }

    #[test]
    fn xpath_should_parse1() {
        // arrange
        let input = "//div[@class='BorderGrid-cell']/div[@class=' text-small']/a";

        // act
        let (next_input, res) = xpath(input).unwrap();

        // assert
        assert_eq!(res.to_string(), input);
        assert_eq!(next_input, "");
    }

    #[test]
    fn xpath_should_parse2() {
        // arrange
        let input = r#"fn:doc("bib.xml")/books/book[fn:count(./author)>1]"#;

        // act
        let (next_input, res) = xpath(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }

    #[test]
    fn xpath_should_parse3() {
        // arrange
        let input = "book/(chapter|appendix)/section";

        // act
        let (next_input, res) = xpath(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }

    #[test]
    fn xpath_should_parse4() {
        // arrange
        let input = "$products[price gt 100]";

        // act
        let (next_input, res) = xpath(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }

    #[test]
    fn xpath_should_parse5() {
        // arrange
        let input =
            r#"(fn:root(self::node()) treat as document-node())/descendant-or-self::node()"#;

        // act
        let (next_input, res) = xpath(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }

    #[test]
    fn xpath_should_parse6() {
        // arrange
        let input = r#"$emp/bonus>0.25 * $emp/salary"#;

        // act
        let (next_input, res) = expr_single(input).unwrap();

        // assert
        assert_eq!(next_input, "");
        assert_eq!(res.to_string(), input);
    }
}
