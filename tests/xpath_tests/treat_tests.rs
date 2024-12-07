use skyscraper::{html, xpath};

#[test]
fn treat_correct_type_should_not_fail() {
    // arrange
    let text = r###"
        <html>
            <body>
            </body>
        </html>"###;

    let document = html::parse(&text).unwrap();
    let xpath = xpath::parse("/html treat as node()").unwrap();

    // act
    let nodes = xpath.apply(&document).unwrap();

    // assert
    assert_eq!(nodes.len(), 1);
}

#[test]
fn treat_incorrect_type_should_fail() {
    // arrange
    let text = r###"
        <html>
            <body>
            </body>
        </html>"###;

    let document = html::parse(&text).unwrap();
    let xpath = xpath::parse("/html treat as document-node()").unwrap();

    // act
    let err = xpath.apply(&document).unwrap_err();

    // assert
    assert_eq!(
        err.to_string(),
        "Error applying expression err:XPDY0050 Cannot treat XpathItemSet { index_set: {Node(ElementNode(ElementNode { name: \"html\" }))} } as document-node()"
    );
}
