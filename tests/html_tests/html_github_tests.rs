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
                .add_element("link", |link| {
                    link.add_attributes_str(vec![
                        ("rel", "dns-prefetch"),
                        ("href", "https://avatars.githubusercontent.com/"),
                    ])
                })
                .add_element("link", |link| {
                    link.add_attributes_str(vec![
                        ("rel", "dns-prefetch"),
                        ("href", "https://github-cloud.s3.amazonaws.com/"),
                    ])
                })
                .add_element("link", |link| {
                    link.add_attributes_str(vec![
                        ("rel", "dns-prefetch"),
                        ("href", "https://user-images.githubusercontent.com/"),
                    ])
                })
                .add_element("link", |link| {
                    link.add_attributes_str(vec![
                        ("rel", "preconnect"),
                        ("href", "https://github.githubassets.com/"),
                        ("crossorigin", ""),
                    ])
                })
                .add_element("link", |link| {
                    link.add_attributes_str(vec![
                        ("rel", "preconnect"),
                        ("href", "https://avatars.githubusercontent.com/"),
                    ])
                })
            })
            .add_element("body", |body| {
                body.add_element("div", |div| {
                    {
                        div.add_element("p", |p| p.add_text("1"))
                            .add_element("p", |p| p.add_text("2"))
                            .add_element("p", |p| p.add_text("3"))
                    }
                    .add_element("div", |div| {
                        div.add_element("p", |p| p.add_text("4"))
                            .add_element("p", |p| p.add_text("5"))
                            .add_element("p", |p| p.add_text("6"))
                    })
                })
            })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}
