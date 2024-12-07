use skyscraper::html::trim_internal_whitespace;
use skyscraper::xpath::grammar::data_model::TextNode;
use skyscraper::xpath::grammar::XpathItemTreeNode;
use skyscraper::{html, xpath};

static HTML: &'static str = include_str!("../samples/James-LG_Skyscraper.html");

#[test]
fn xpath_github_sample1() {
    // arrange
    let text: String = HTML.parse().unwrap();

    let document = html::parse(&text).unwrap();
    let xpath = xpath::parse("//main").unwrap();

    // act
    let nodes = xpath.apply(&document).unwrap();

    // assert
    assert_eq!(nodes.len(), 1);
    let mut nodes = nodes.into_iter();

    let tree_node = nodes.next().unwrap().extract_into_node();

    let element = tree_node.extract_as_element_node();
    assert_eq!(element.name, "main")
}

#[test]
fn xpath_github_sample2() {
    // arrange
    let text: String = HTML.parse().unwrap();

    let document = html::parse(&text).unwrap();
    let xpath = xpath::parse("//a[@class='Link--secondary']").unwrap();

    // act
    let nodes = xpath.apply(&document).unwrap();

    // assert
    assert_eq!(nodes.len(), 5);
    let mut nodes = nodes.into_iter();

    let tree_node = nodes.next().unwrap().extract_into_node();

    let element = tree_node.extract_as_element_node();
    assert_eq!(element.name, "a");

    let children: Vec<&TextNode> = tree_node
        .children(&document)
        .into_iter()
        .filter_map(|x| x.as_text_node().ok())
        .collect();

    assert_eq!(1, children.len());
    let mut children = children.into_iter();

    let text = children.next().unwrap();
    assert_eq!(text.content, "refactor: Reorganize into workspace")
}

#[test]
fn xpath_github_sample3() {
    // arrange
    let text: String = HTML.parse().unwrap();

    let document = html::parse(&text).unwrap();
    let xpath =
        xpath::parse("//div[@class='BorderGrid-cell']/div[@class=' text-small']/a").unwrap();

    // act
    let nodes = xpath.apply(&document).unwrap();

    // assert
    assert_eq!(nodes.len(), 1);
    let mut nodes = nodes.into_iter();

    let tree_node = nodes.next().unwrap().extract_into_node();

    let element = tree_node.extract_as_element_node();
    assert_eq!(element.name, "a");

    let children: Vec<&XpathItemTreeNode> = tree_node.children(&document);
    assert_eq!(children.len(), 2);
    let mut children = children.into_iter();

    let attribute = children.next().unwrap().extract_as_attribute_node();
    assert_eq!(attribute.name, "href");
    assert_eq!(
        attribute.value,
        "https://github.com/James-LG/Skyscraper/releases/new"
    );

    let text = children.next().unwrap().extract_as_text_node();
    assert_eq!(text.content, "Create a new release");
}

#[test]
fn xpath_github_get_text_sample() {
    // arrange
    let text: String = HTML.parse().unwrap();

    let document = html::parse(&text).unwrap();
    let xpath = xpath::parse("//div[@class='flex-auto min-width-0 width-fit mr-3']").unwrap();

    // act
    let nodes = xpath.apply(&document).unwrap();

    // assert
    assert_eq!(nodes.len(), 1);
    let mut nodes = nodes.into_iter();

    let element = nodes.next().unwrap().extract_into_node();

    let text = element.text_content(&document).trim().to_string();
    let trimmed_text = trim_internal_whitespace(&text);

    assert_eq!(trimmed_text, "James-LG / Skyscraper Public");
}

#[test]
fn xpath_github_parent_axis() {
    // arrange
    let text: String = HTML.parse().unwrap();

    let document = html::parse(&text).unwrap();
    let xpath = xpath::parse("//div[@role='gridcell']/parent::div").unwrap();

    // act
    let nodes = xpath.apply(&document).unwrap();

    // assert
    assert_eq!(nodes.len(), 5);
}

#[test]
fn xpath_github_parent_axis_recursive() {
    // arrange
    let text: String = HTML.parse().unwrap();

    let document = html::parse(&text).unwrap();
    let xpath = xpath::parse("//div[@role='gridcell']//parent::div").unwrap();

    // act
    let nodes = xpath.apply(&document).unwrap();

    // assert
    assert_eq!(nodes.len(), 20);
}

#[test]
fn xpath_github_dashed_attribute() {
    // arrange
    let text: String = HTML.parse().unwrap();

    let document = html::parse(&text).unwrap();
    let xpath = xpath::parse("//span[@data-view-component='true']").unwrap();

    // act
    let nodes = xpath.apply(&document).unwrap();

    // assert
    assert_eq!(nodes.len(), 19);
}

#[test]
fn xpath_github_get_attributes_sample() {
    // arrange
    let text: String = HTML.parse().unwrap();

    let document = html::parse(&text).unwrap();
    let xpath = xpath::parse("//div[@class='flex-auto min-width-0 width-fit mr-3']").unwrap();

    // act
    let nodes = xpath.apply(&document).unwrap();

    // assert
    assert_eq!(nodes.len(), 1);
    let mut nodes = nodes.into_iter();

    let tree_node = nodes.next().unwrap().extract_into_node();
    let elem = tree_node.extract_as_element_node();

    assert_eq!(
        elem.get_attribute(&document, "class").unwrap(),
        "flex-auto min-width-0 width-fit mr-3"
    );
}

#[test]
fn xpath_github_root_search() {
    // arrange
    let text: String = HTML.parse().unwrap();

    let document = html::parse(&text).unwrap();
    let xpath = xpath::parse("/html").unwrap();

    // act
    let nodes = xpath.apply(&document).unwrap();

    // assert
    assert_eq!(nodes.len(), 1);
    let mut nodes = nodes.into_iter();

    let tree_node = nodes.next().unwrap().extract_into_node();

    let element = tree_node.extract_as_element_node();
    assert_eq!(element.name, "html");
}

#[test]
fn xpath_github_root_search_all() {
    // arrange
    let text: String = HTML.parse().unwrap();

    let document = html::parse(&text).unwrap();
    let xpath = xpath::parse("//html").unwrap();

    // act
    let nodes = xpath.apply(&document).unwrap();

    // assert
    assert_eq!(nodes.len(), 1);
    let mut nodes = nodes.into_iter();

    let tree_node = nodes.next().unwrap().extract_into_node();

    let element = tree_node.extract_as_element_node();
    assert_eq!(element.name, "html")
}

#[test]
fn xpath_github_root_wildcard() {
    // arrange
    let text: String = HTML.parse().unwrap();

    let document = html::parse(&text).unwrap();
    let xpath = xpath::parse("//body/*").unwrap();

    // act
    let nodes = xpath.apply(&document).unwrap();

    // assert
    assert_eq!(nodes.len(), 16);

    // assert first node
    let tree_node = &nodes[0].extract_as_node();
    let elem = tree_node.extract_as_element_node();

    assert_eq!(elem.name, "div");

    assert_eq!(
        elem.get_attribute(&document, "class").unwrap(),
        "position-relative js-header-wrapper "
    );

    // assert random node 4
    let tree_node = &nodes[3].extract_as_node();
    let elem = tree_node.extract_as_element_node();

    assert_eq!(elem.name, "include-fragment");

    // assert last node
    let tree_node = &nodes[15].extract_as_node();
    let elem = tree_node.extract_as_element_node();

    assert_eq!(elem.name, "div");

    assert_eq!(elem.get_attribute(&document, "class").unwrap(), "sr-only");
    assert_eq!(
        elem.get_attribute(&document, "aria-live").unwrap(),
        "polite"
    )
}
