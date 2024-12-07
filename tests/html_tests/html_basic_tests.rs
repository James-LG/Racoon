use skyscraper::html::{self, grammar::document_builder::DocumentBuilder};

use crate::test_framework;

#[test]
fn parse_should_return_document() {
    // arrange
    let text = r###"
            <html>
                <body>
                    <div>
                        <p>1</p>
                        <p>2</p>
                        <p>3</p>
                    </div>
                    <div>
                        <p>4</p>
                        <p>5</p>
                        <p>6</p>
                    </div>
                </body>
            </html>"###;

    // act
    let document = html::parse(&text).unwrap();

    // assert
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("div", |div| {
                        div.add_element("p", |p| p.add_text("1"))
                            .add_element("p", |p| p.add_text("2"))
                            .add_element("p", |p| p.add_text("3"))
                    })
                    .add_element("div", |div| {
                        div.add_element("p", |p| p.add_text("4"))
                            .add_element("p", |p| p.add_text("5"))
                            .add_element("p", |p| p.add_text("6"))
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn text_should_include_text_before_between_and_after_child_element() {
    // arrange
    let text = r##"
        <div>
            hello
            <span>my</span>
            friend
        </div>"##;

    // act
    let document = html::parse(text).unwrap();

    // assert
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("div", |div| {
                        div.add_text("\n            hello\n            ")
                            .add_element("span", |span| span.add_text("my"))
                            .add_text("\n            friend\n        ")
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn sample1_should_parse() {
    // arrange
    let text = r#"<html><body><div id="example">Example 1</div></body></html>"#;

    // act
    let document = html::parse(text).unwrap();

    // assert
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("div", |div| {
                        div.add_text("Example 1").add_attribute_str("id", "example")
                    })
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, false));
}

#[test]
fn sample2_should_parse() {
    // arrange
    let text = r###"
        <html id="foo" class="bar" style="baz">
        </html>"###;

    // act
    let document = html::parse(text).unwrap();

    // assert
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| body)
                .add_attribute_str("id", "foo")
                .add_attribute_str("class", "bar")
                .add_attribute_str("style", "baz")
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, false));
}

#[test]
fn comment_should_parse() {
    // arrange
    let text = r###"
        <html>
            <!-- comment -->
        </html>"###;

    // act
    let document = html::parse(text).unwrap();

    // assert
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_comment(" comment ")
                .add_element("head", |head| head)
                .add_element("body", |body| body)
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, false));
}

#[test]
fn text_should_unescape_characters() {
    // arrange
    let text = r##"<div>&amp;&quot;&#39;&lt;&gt;&#96;</div>"##;

    // act
    let document = html::parse(text).unwrap();

    // assert
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("div", |div| div.add_text(r##"&"'<>`"##))
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn doctype_should_handle_regular_doctype() {
    // arrange
    let text = r##"
        <!DOCTYPE html>
        <div>hi</div>"##;

    // act
    let document = html::parse(text).unwrap();

    // assert
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("div", |div| div.add_text("hi"))
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn doctype_should_skip_verbose_doctype() {
    // arrange
    let text = r##"
        <!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Transitional//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd">
        <div>hi</div>"##;

    // act
    let document = html::parse(text).unwrap();

    // assert
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| head)
                .add_element("body", |body| {
                    body.add_element("div", |div| div.add_text("hi"))
                })
        })
        .build()
        .unwrap();

    assert!(test_framework::compare_documents(expected, document, true));
}

#[test]
fn script_should_close_properly() {
    // arrange
    let text = r###"
        <html>

        <head>
        <script></script>
        <script></script>
        </head>

        <body>
        </body>

        </html>
        "###;

    // act
    let document = html::parse(text).unwrap();

    // assert
    let expected = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("script", |script| script)
                    .add_element("script", |script| script)
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();

    println!("{}", expected.to_string());
    println!("{}", document.to_string());

    // assert!(test_framework::compare_documents(expected, document, true));
}
