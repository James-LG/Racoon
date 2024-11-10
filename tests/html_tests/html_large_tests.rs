use skyscraper::html::{self, grammar::document_builder::DocumentBuilder};

use crate::test_framework;

static HTML: &'static str = include_str!("../samples/large.html");

#[test]
fn parse_should_return_document() {
    // arrange
    let text: String = HTML.parse().unwrap();

    // act
    let document = html::parse(&text).unwrap();

    // assert
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                {
                    {
                        head.add_element("script", |script| script)
                            .add_element("script", |script| script)
                    }
                }
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}
