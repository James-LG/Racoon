use skyscraper::html::{self, grammar::document_builder::DocumentBuilder};

use crate::test_framework;

static HTML: &'static str = include_str!("../samples/James-LG_Skyscraper.html");

#[test]
fn parse_should_return_document() {
    // arrange
    let text: String = HTML.parse().unwrap();

    // act
    let document = html::parse(&text).unwrap();

    // assert
    let displayed_document = document.to_string();

    assert_eq!(displayed_document, text);
}
