use skyscraper::{
    html,
    xpath::{
        self,
        grammar::{data_model::AttributeNode, XpathItemTreeNode},
    },
};

#[test]
fn text_test_should_match_all_text() {
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

    let document = html::parse(&text).unwrap();
    let xpath = xpath::parse("//text()").unwrap();

    // act
    let nodes = xpath.apply(&document).unwrap();

    // assert
    // filter out whitespace nodes before testing since they vary by HTML parser.
    let nodes: Vec<_> = nodes
        .into_iter()
        .filter(|n| match n.extract_as_node() {
            XpathItemTreeNode::TextNode(text) => !text.is_whitespace(),
            _ => true,
        })
        .collect();

    assert_eq!(nodes.len(), 6);
    let mut nodes = nodes.into_iter();

    // assert node
    {
        let tree_node = nodes.next().unwrap().extract_into_node();

        let text_node = tree_node.extract_as_text_node();
        assert_eq!(text_node.content, "1");
    }

    // assert node
    {
        let tree_node = nodes.next().unwrap().extract_into_node();

        let text_node = tree_node.extract_as_text_node();
        assert_eq!(text_node.content, "2");
    }

    // assert node
    {
        let tree_node = nodes.next().unwrap().extract_into_node();

        let text_node = tree_node.extract_as_text_node();
        assert_eq!(text_node.content, "3");
    }

    // assert node
    {
        let tree_node = nodes.next().unwrap().extract_into_node();

        let text_node = tree_node.extract_as_text_node();
        assert_eq!(text_node.content, "4");
    }

    // assert node
    {
        let tree_node = nodes.next().unwrap().extract_into_node();

        let text_node = tree_node.extract_as_text_node();
        assert_eq!(text_node.content, "5");
    }

    // assert node
    {
        let tree_node = nodes.next().unwrap().extract_into_node();

        let text_node = tree_node.extract_as_text_node();
        assert_eq!(text_node.content, "6");
    }
}

#[test]
fn attribute_test_should_match_all_attributes() {
    // arrange
    let text = r###"
        <html id="foo" class="bar" style="baz">
        </html>"###;

    let document = html::parse(&text).unwrap();
    let xpath = xpath::parse("/html/attribute::*").unwrap();

    // act
    let nodes = xpath.apply(&document).unwrap();

    // assert
    assert_eq!(nodes.len(), 3);
    let attributes: Vec<&AttributeNode> = nodes
        .into_iter()
        .filter_map(|x| x.extract_into_node().as_attribute_node().ok())
        .collect();

    // assert attribute
    {
        let attribute = attributes
            .iter()
            .find(|x| x.name == "id")
            .expect("missing id attribute");

        assert_eq!(attribute.value, "foo")
    }

    // assert attribute
    {
        let attribute = attributes
            .iter()
            .find(|x| x.name == "class")
            .expect("missing id attribute");

        assert_eq!(attribute.value, "bar")
    }

    // assert attribute
    {
        let attribute = attributes
            .iter()
            .find(|x| x.name == "style")
            .expect("missing id attribute");

        assert_eq!(attribute.value, "baz")
    }
}
