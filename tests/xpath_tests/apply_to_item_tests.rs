use skyscraper::{html, xpath};

#[test]
fn apply_to_item_should_select_given_node() {
    /* Stage 0: Document setup. */
    // arrange
    let text = r###"
        <html>
            <body>
            </body>
        </html>"###;

    let document = html::parse(&text).unwrap();

    /* Stage 1: Select a node. */
    // arrange
    let xpath = xpath::parse("/html/body").unwrap();

    // act
    let nodes = xpath.apply(&document).unwrap();

    // assert
    assert_eq!(nodes.len(), 1);
    let first_expr_item = nodes.into_iter().next().unwrap();

    /* Stage 2: Apply context expression to selected node. */
    // arrange
    let xpath = xpath::parse(".").unwrap();

    // act
    let nodes = xpath
        .apply_to_item(&document, first_expr_item.clone())
        .unwrap();

    // assert
    assert_eq!(nodes.len(), 1);
    let second_expr_item = nodes.into_iter().next().unwrap();

    assert_eq!(first_expr_item, second_expr_item);

    // assert node
    let tree_node = second_expr_item.extract_into_node();

    let element = tree_node.extract_as_element_node();
    assert_eq!(element.name, "body")
}

#[test]
fn apply_to_item_slash_should_select_children() {
    /* Stage 0: Document setup. */
    // arrange
    let text = r###"
        <html>
            <body>
                <span>1</span>
                <span>2</span>
                <span>3</span>
            </body>
        </html>"###;

    let document = html::parse(&text).unwrap();

    /* Stage 1: Select a node. */
    // arrange
    let xpath = xpath::parse("/html/body").unwrap();

    // act
    let nodes = xpath.apply(&document).unwrap();

    // assert
    assert_eq!(nodes.len(), 1);
    let first_expr_item = nodes.into_iter().next().unwrap();

    /* Stage 2: Apply expression to selected node. */
    // arrange
    let xpath = xpath::parse("/span").unwrap();

    // act
    let nodes = xpath
        .apply_to_item(&document, first_expr_item.clone())
        .unwrap();

    // assert
    assert_eq!(nodes.len(), 3);
    let mut nodes = nodes.into_iter();

    // assert node
    {
        let tree_node = nodes.next().unwrap().extract_into_node();

        let element = tree_node.extract_as_element_node();
        assert_eq!(element.name, "span");

        assert_eq!(tree_node.text(&document), Some("1".to_string()));
    }

    // assert node
    {
        let tree_node = nodes.next().unwrap().extract_into_node();

        let element = tree_node.extract_as_element_node();
        assert_eq!(element.name, "span");

        assert_eq!(tree_node.text(&document), Some("2".to_string()));
    }

    // assert node
    {
        let tree_node = nodes.next().unwrap().extract_into_node();

        let element = tree_node.extract_as_element_node();
        assert_eq!(element.name, "span");

        assert_eq!(tree_node.text(&document), Some("3".to_string()));
    }
}

#[test]
fn apply_to_item_double_slash_should_select_self_or_descendents() {
    /* Stage 0: Document setup. */
    // arrange
    let text = r###"
        <html>
            <body>
                <span>1</span>
                <span>2</span>
                <span>3</span>
            </body>
        </html>"###;

    let document = html::parse(&text).unwrap();

    /* Stage 1: Select a node. */
    // arrange
    let xpath = xpath::parse("/html").unwrap();

    // act
    let nodes = xpath.apply(&document).unwrap();

    // assert
    assert_eq!(nodes.len(), 1);
    let first_expr_item = nodes.into_iter().next().unwrap();

    /* Stage 2: Apply expression to selected node. */
    // arrange
    let xpath = xpath::parse("//span").unwrap();

    // act
    let nodes = xpath
        .apply_to_item(&document, first_expr_item.clone())
        .unwrap();

    // assert
    assert_eq!(nodes.len(), 3);
    let mut nodes = nodes.into_iter();

    // assert node
    {
        let tree_node = nodes.next().unwrap().extract_into_node();

        let element = tree_node.extract_as_element_node();
        assert_eq!(element.name, "span");

        assert_eq!(tree_node.text(&document), Some("1".to_string()));
    }

    // assert node
    {
        let tree_node = nodes.next().unwrap().extract_into_node();

        let element = tree_node.extract_as_element_node();
        assert_eq!(element.name, "span");

        assert_eq!(tree_node.text(&document), Some("2".to_string()));
    }

    // assert node
    {
        let tree_node = nodes.next().unwrap().extract_into_node();

        let element = tree_node.extract_as_element_node();
        assert_eq!(element.name, "span");

        assert_eq!(tree_node.text(&document), Some("3".to_string()));
    }
}
