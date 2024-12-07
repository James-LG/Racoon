use skyscraper::html::grammar::document_builder::DocumentBuilder;

use crate::test_framework;

#[test]
fn different_document_should_not_be_equal() {
    // arrange
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

    let actual = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("script", |script| {
                    script
                        .add_element("script", |script| script)
                        .add_element("body", |body| body)
                })
            })
        })
        .build()
        .unwrap();

    println!("Expected document:\n{}", expected);
    println!("Actual document:\n{}", actual);

    // act
    let result = test_framework::compare_documents(expected, actual, false);

    // assert
    assert!(!result);
}

#[test]
fn same_document_should_be_equal() {
    // arrange
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

    let actual = DocumentBuilder::new()
        .add_element("html", |html| {
            html.add_element("head", |head| {
                head.add_element("script", |script| script)
                    .add_element("script", |script| script)
            })
            .add_element("body", |body| body)
        })
        .build()
        .unwrap();

    println!("Expected document:\n{}", expected);
    println!("Actual document:\n{}", actual);

    // act
    let result = test_framework::compare_documents(expected, actual, false);

    // assert
    assert!(result);
}
