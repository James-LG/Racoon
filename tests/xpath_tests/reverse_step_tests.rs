use skyscraper::{html, xpath};

#[test]
fn parent_axis_should_select_parent_node() {
    // arrange
    let text = r###"
        <html>
            <body>
            </body>
        </html>"###;

    let document = html::parse(&text).unwrap();
    let xpath = xpath::parse("//body/parent::html").unwrap();

    // act
    let nodes = xpath.apply(&document).unwrap();

    // assert
    assert_eq!(nodes.len(), 1);
    let mut nodes = nodes.into_iter();

    // assert node
    {
        let tree_node = nodes.next().unwrap().extract_into_node();

        let element = tree_node.extract_as_element_node();
        assert_eq!(element.name, "html");
    }
}

#[test]
fn parent_axis_should_select_parents_of_all_selected_nodes() {
    // arrange

    // `not-p` is not selected by the initial `//p` query, so its parent `div` should not be selected by the parent axis.
    let text = r###"
        <html>
            <body>
                <div id="1">
                    <p>1</p>
                </div>
                <div id="2">
                    <p>2</p>
                </div>
                <div>
                    <not-p>3</not-p>
                </div>
            </body>
        </html>"###;

    let document = html::parse(&text).unwrap();
    let xpath = xpath::parse("//p/parent::div").unwrap();

    // act
    let nodes = xpath.apply(&document).unwrap();

    // assert
    assert_eq!(nodes.len(), 2);
    let mut nodes = nodes.into_iter();

    // assert node
    {
        let tree_node = nodes.next().unwrap().extract_into_node();

        let element = tree_node.extract_as_element_node();
        assert_eq!(element.name, "div");
        assert_eq!(element.get_attribute(&document, "id"), Some("1"));
    }

    // assert node
    {
        let tree_node = nodes.next().unwrap().extract_into_node();

        let element = tree_node.extract_as_element_node();
        assert_eq!(element.name, "div");
        assert_eq!(element.get_attribute(&document, "id"), Some("2"));
    }
}

/// The node test (ex. `div`) is applied to the parent of the context node.
#[test]
fn parent_axis_should_respect_node_test() {
    // arrange

    // `not-div` does not match the node test of `div`, so it should not be selected.
    let text = r###"
        <html>
            <body>
                <div id="1">
                    <p>1</p>
                </div>
                <div id="2">
                    <p>2</p>
                </div>
                <not-div>
                    <p>3</p>
                </not-div>
            </body>
        </html>"###;

    let document = html::parse(&text).unwrap();
    let xpath = xpath::parse("//p/parent::div").unwrap();

    // act
    let nodes = xpath.apply(&document).unwrap();

    // assert
    assert_eq!(nodes.len(), 2);
    let mut nodes = nodes.into_iter();

    // assert node
    {
        let tree_node = nodes.next().unwrap().extract_into_node();

        let element = tree_node.extract_as_element_node();
        assert_eq!(element.name, "div");
        assert_eq!(element.get_attribute(&document, "id"), Some("1"));
    }

    // assert node
    {
        let tree_node = nodes.next().unwrap().extract_into_node();

        let element = tree_node.extract_as_element_node();
        assert_eq!(element.name, "div");
        assert_eq!(element.get_attribute(&document, "id"), Some("2"));
    }
}
