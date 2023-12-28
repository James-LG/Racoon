use skyscraper::{
    html,
    xpath::{self, grammar::XpathItemTreeNodeData},
};

#[test]
fn class_equals_predicate_should_select_nodes_with_that_match() {
    // arrange
    let text = r###"
        <html>
            <div>
                bad
            </div>
            <div class="here">
                good
            </div>
        </html>"###;

    let document = html::parse(&text).unwrap();
    let xpath_item_tree = xpath::XpathItemTree::from(&document);
    let xpath = xpath::parse("/html/div[@class='here']").unwrap();

    // act
    let nodes = xpath.apply(&xpath_item_tree).unwrap();

    // assert
    assert_eq!(nodes.len(), 1);
    let mut nodes = nodes.into_iter();

    // assert node
    {
        let tree_node = nodes
            .next()
            .unwrap()
            .extract_into_node()
            .extract_into_tree_node();

        match tree_node.data {
            XpathItemTreeNodeData::ElementNode(e) => {
                assert_eq!(e.name, "div")
            }
            _ => panic!("expected element, got {:?}", tree_node.data),
        }

        assert_eq!(tree_node.text(&xpath_item_tree).trim(), "good");
    }
}

#[test]
fn predicate_on_double_leading_slash_should_select_nodes_with_that_match() {
    // arrange
    let text = r###"
        <html>
            <div>
                bad
            </div>
            <div class="here">
                good
            </div>
        </html>"###;

    let document = html::parse(&text).unwrap();
    let xpath_item_tree = xpath::XpathItemTree::from(&document);
    let xpath = xpath::parse("//div[@class='here']").unwrap();

    // act
    let nodes = xpath.apply(&xpath_item_tree).unwrap();

    // assert
    assert_eq!(nodes.len(), 1);
    let mut nodes = nodes.into_iter();

    // assert node
    {
        let tree_node = nodes
            .next()
            .unwrap()
            .extract_into_node()
            .extract_into_tree_node();

        match tree_node.data {
            XpathItemTreeNodeData::ElementNode(e) => {
                assert_eq!(e.name, "div")
            }
            _ => panic!("expected element, got {:?}", tree_node.data),
        }

        assert_eq!(tree_node.text(&xpath_item_tree).trim(), "good");
    }
}

#[test]
fn index_should_select_indexed_child_for_all_selected_parents() {
    // arrange
    let text = r###"
        <html>
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
        </html>"###;

    let document = html::parse(&text).unwrap();
    let xpath_item_tree = xpath::XpathItemTree::from(&document);
    let xpath = xpath::parse("//div/p[2]").unwrap();

    // act
    let nodes = xpath.apply(&xpath_item_tree).unwrap();

    // assert
    assert_eq!(nodes.len(), 2);
    let mut nodes = nodes.into_iter();

    // assert node
    {
        let tree_node = nodes
            .next()
            .unwrap()
            .extract_into_node()
            .extract_into_tree_node();

        match tree_node.data {
            XpathItemTreeNodeData::ElementNode(e) => {
                assert_eq!(e.name, "p")
            }
            _ => panic!("expected element, got {:?}", tree_node.data),
        }

        assert_eq!(tree_node.text(&xpath_item_tree).trim(), "2");
    }

    // assert node
    {
        let tree_node = nodes
            .next()
            .unwrap()
            .extract_into_node()
            .extract_into_tree_node();

        match tree_node.data {
            XpathItemTreeNodeData::ElementNode(e) => {
                assert_eq!(e.name, "p")
            }
            _ => panic!("expected element, got {:?}", tree_node.data),
        }

        assert_eq!(tree_node.text(&xpath_item_tree).trim(), "5");
    }
}
