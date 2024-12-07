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
        .add_comment(" saved from url=(0038)https://github.com/James-LG/Skyscraper ")
        .add_element("html", |html| {
            html.add_attributes_str(vec![
                ("lang", "en"),
                ("data-color-mode", "dark"),
                ("data-light-theme", "light"),
                ("data-dark-theme", "dark"),
            ])
            .add_element("head", |head| {
                {
                    {
                        head.add_element("meta", |meta| {
                            meta.add_attributes_str(vec![
                                ("http-equiv", "Content-Type"),
                                ("content", "text/html; charset=UTF-8"),
                            ])
                        })
                        .add_element("link", |link| {
                            link.add_attributes_str(vec![
                                ("rel", "dns-prefetch"),
                                ("href", "https://github.githubassets.com/"),
                            ])
                        })
                        .add_element("script", |script| {
                            script.add_attributes_str(vec![
                                ("crossorigin", "anonymous"),
                                ("defer", "defer"),
                                ("type", "application/javascript"),
                                (
                                    "src",
                                    "./James-LG_Skyscraper_files/environment-2bf92300.js.download",
                                ),
                            ])
                        })
                        .add_element("title", |title| title.add_text("James-LG/Skyscraper"))
                    }
                }
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}
