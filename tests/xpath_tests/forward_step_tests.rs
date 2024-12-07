use skyscraper::{
    html,
    xpath::{self, grammar::data_model::AttributeNode},
};

#[test]
fn abbrev_attribute_step_should_match_given_attribute() {
    // arrange
    let text = r###"
        <html id="foo" class="bar" style="baz">
        </html>"###;

    let document = html::parse(&text).unwrap();
    let xpath = xpath::parse("/html/@class").unwrap();

    // act
    let nodes = xpath.apply(&document).unwrap();

    // assert
    assert_eq!(nodes.len(), 1);
    let attributes: Vec<&AttributeNode> = nodes
        .into_iter()
        .map(|item| item.extract_into_node().extract_as_attribute_node())
        .collect();

    // assert attribute
    {
        let attribute = attributes
            .iter()
            .find(|x| x.name == "class")
            .expect("missing id attribute");

        assert_eq!(attribute.value, "bar")
    }
}
