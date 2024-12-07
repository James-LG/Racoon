use skyscraper::{html, xpath};

#[test]
fn root_should_select_root_node() {
    // arrange
    let text = r###"
        <html>
            <body>
            </body>
        </html>"###;

    let document = html::parse(&text).unwrap();
    let xpath = xpath::parse("/html/body/fn:root()").unwrap();

    // act
    let nodes = xpath.apply(&document).unwrap();

    // assert
    assert_eq!(nodes.len(), 1);
    let mut nodes = nodes.into_iter();

    // assert node
    {
        let tree_node = nodes.next().unwrap().extract_into_node();

        tree_node.extract_as_document_node();
    }
}
