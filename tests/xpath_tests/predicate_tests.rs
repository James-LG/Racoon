use skyscraper::{
    html::{self, trim_internal_whitespace},
    xpath,
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
    let xpath = xpath::parse("/html/div[@class='here']").unwrap();

    // act
    let nodes = xpath.apply(&document).unwrap();

    // assert
    assert_eq!(nodes.len(), 1);
    let mut nodes = nodes.into_iter();

    // assert node
    {
        let tree_node = nodes.next().unwrap().extract_into_node();

        let element = tree_node.extract_as_element_node();
        assert_eq!(element.name, "div");

        assert_eq!(
            trim_internal_whitespace(&tree_node.text(&document).unwrap()),
            "good"
        );
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
    let xpath = xpath::parse("//div[@class='here']").unwrap();

    // act
    let nodes = xpath.apply(&document).unwrap();

    // assert
    assert_eq!(nodes.len(), 1);
    let mut nodes = nodes.into_iter();

    // assert node
    {
        let tree_node = nodes.next().unwrap().extract_into_node();

        let element = tree_node.extract_as_element_node();
        assert_eq!(element.name, "div");

        assert_eq!(
            trim_internal_whitespace(&tree_node.text(&document).unwrap()),
            "good"
        );
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
    let xpath = xpath::parse("//div/p[2]").unwrap();

    // act
    let nodes = xpath.apply(&document).unwrap();

    // assert
    assert_eq!(nodes.len(), 2);
    let mut nodes = nodes.into_iter();

    // assert node
    {
        let tree_node = nodes.next().unwrap().extract_into_node();

        let element = tree_node.extract_as_element_node();
        assert_eq!(element.name, "p");

        assert_eq!(
            trim_internal_whitespace(&tree_node.text(&document).unwrap()),
            "2"
        );
    }

    // assert node
    {
        let tree_node = nodes.next().unwrap().extract_into_node();

        let element = tree_node.extract_as_element_node();
        assert_eq!(element.name, "p");

        assert_eq!(
            trim_internal_whitespace(&tree_node.text(&document).unwrap()),
            "5"
        );
    }
}

/// The index being out of bounds for one parent should not affect the selection of the indexed node for some other parent.
#[test]
fn index_out_of_bounds_should_select_nothing_for_parent() {
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
            </div>
        </html>"###;

    let document = html::parse(&text).unwrap();
    let xpath = xpath::parse("//div/p[2]").unwrap();

    // act
    let nodes = xpath.apply(&document).unwrap();

    // assert
    assert_eq!(nodes.len(), 2);
    let mut nodes = nodes.into_iter();

    // assert node
    {
        let tree_node = nodes.next().unwrap().extract_into_node();

        let element = tree_node.extract_as_element_node();
        assert_eq!(element.name, "p");

        assert_eq!(
            trim_internal_whitespace(&tree_node.text(&document).unwrap()),
            "2"
        );
    }
}
