use skyscraper::{
    html::{self, trim_internal_whitespace},
    xpath,
};

#[test]
fn contains_should_select_text() {
    // arrange
    let text = r###"
        <html>
            <div>
                hello world
            </div>
            <div>
                select me
            </div>
        </html>"###;

    let document = html::parse(&text).unwrap();
    let xpath = xpath::parse("//div[contains(text(),'select')]").unwrap();

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
            "select me"
        );
    }
}

#[test]
fn contains_should_select_attribute() {
    // arrange
    let text = r###"
        <html>
            <div class="nah">
                hello world
            </div>
            <div class="select me">
                select me
            </div>
        </html>"###;

    let document = html::parse(&text).unwrap();
    let xpath = xpath::parse("//div[contains(@class,'select')]").unwrap();

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
            "select me"
        );
    }
}

#[test]
fn contains_should_select_on_expression() {
    // arrange
    let text = r###"
        <html>
            <div>
                other
            </div>
            <div>
                hello world
            </div>
            <div class="hello" id="select"></div>
        
        </html>"###;

    let document = html::parse(&text).unwrap();

    // this expression first selects the class attribute of the div with id 'select',
    // then uses that result to find a div with text containing that class.
    let xpath = xpath::parse("//div[contains(text(),//div[@id='select']/@class)]").unwrap();

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
            "hello world"
        );
    }
}
