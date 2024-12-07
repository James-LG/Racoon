use std::{
    collections::HashMap,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

use itertools::Itertools;
use serde::Deserialize;
use skyscraper::{
    html::{self, HtmlNode},
    xpath::{self, xpath_item_set::XpathItemSet, XpathItemTree},
};

#[derive(Deserialize, Debug, PartialEq)]
struct LxmlElement {
    pub tag: String,
    pub text: Option<String>,
    pub text_content: String,
    pub attrib: HashMap<String, String>,
    pub itertext: Vec<String>,
}

fn get_lxml_output(xpath: &str, html_text: String, count_only: bool) -> std::process::Output {
    let mut lxml_python_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    lxml_python_path.push("tests/lxml_tests/xpath.py");

    let mut cmd = Command::new("python3");
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .arg(
            lxml_python_path
                .clone()
                .into_os_string()
                .into_string()
                .unwrap(),
        )
        .arg(xpath);

    if count_only {
        cmd.arg("--count-only");
    }

    let mut process = cmd.spawn().expect("failed to spawn process");

    let mut stdin = process.stdin.take().expect("Failed to open stdin");
    std::thread::spawn(move || {
        stdin
            .write_all(html_text.as_bytes())
            .expect("Failed to write to stdin");
    });

    let output = process
        .wait_with_output()
        .expect("failed to execute stack overflow tests");

    output
}

fn get_lxml_elements(xpath: &str, html_text: String) -> Vec<LxmlElement> {
    let output = get_lxml_output(xpath, html_text, false);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "{}\n{}",
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );

    let lxml_elements: Vec<LxmlElement> = serde_json::from_str(&stdout).unwrap();
    return lxml_elements;
}

fn skyscraper_to_lxml_elements(
    xpath_tree: &XpathItemTree,
    item_set: XpathItemSet,
) -> Vec<LxmlElement> {
    let mut lxml_elements = Vec::new();
    for item in item_set.into_iter() {
        let node = item.extract_into_node();
        let element = node.extract_as_element_node();
        let text = element.text(&xpath_tree);
        let text_content = element.text_content(&xpath_tree);
        let itertext = element.itertext(&xpath_tree).collect();

        lxml_elements.push(LxmlElement {
            tag: element.name.to_string(),
            text,
            text_content,
            attrib: element
                .attributes(&xpath_tree)
                .iter()
                .map(|x| (x.name.clone(), x.value.clone()))
                .collect(),
            itertext,
        });
    }
    return lxml_elements;
}

fn compare_skyscraper_to_lxml(
    lxml_elements: Vec<LxmlElement>,
    converted_skyscraper_elems: Vec<LxmlElement>,
) {
    for (i, eb) in lxml_elements
        .iter()
        .zip_longest(converted_skyscraper_elems.iter())
        .enumerate()
    {
        let (lxml_elem, skyscraper_elem) = eb.left_and_right();

        if let (Some(lxml_elem), Some(skyscraper_elem)) = (lxml_elem, skyscraper_elem) {
            assert_eq!(
                lxml_elem.tag, skyscraper_elem.tag,
                "Tag mismatch at index {}",
                i
            );
            assert_eq!(
                lxml_elem.text, skyscraper_elem.text,
                "Text mismatch at index {}",
                i
            );
            assert_eq!(
                lxml_elem.attrib, skyscraper_elem.attrib,
                "Attribute mismatch at index {}",
                i
            );
            compare_itertext(&lxml_elem.itertext, &skyscraper_elem.itertext);
        } else {
            assert_eq!(
                lxml_elem, skyscraper_elem,
                "Element mismatch at index {}",
                i
            );
        }
    }
    assert_eq!(converted_skyscraper_elems.len(), lxml_elements.len());
}

fn compare_itertext(first: &Vec<String>, second: &Vec<String>) {
    println!("First itertext:");
    for (i, x) in first.iter().enumerate() {
        println!("{}: {:?}", i, x);
    }

    println!("Second itertext:");
    for (i, x) in second.iter().enumerate() {
        println!("{}: {:?}", i, x);
    }

    for (i, eb) in first.iter().zip_longest(second.iter()).enumerate() {
        let (first, second) = eb.left_and_right();
        assert_eq!(first, second, "Itertext mismatch at index {}", i);
    }
}

static GITHUB_HTML: &'static str = include_str!("../samples/James-LG_Skyscraper.html");

/// This test is a sanity check of the lxml output.
#[test]
fn test_lxml_output() {
    // arrange
    let html_text = GITHUB_HTML.to_string();
    let xpath = r#"//a[@rel='author']"#;

    // act
    let lxml_elements = get_lxml_elements(xpath, html_text);

    // assert
    assert_eq!(lxml_elements.len(), 1);

    let mut lxml_elements = lxml_elements.into_iter();
    let lxml_element = lxml_elements.next().unwrap();
    assert_eq!(lxml_element.tag, "a");
    assert_eq!(lxml_element.text, Some("James-LG".to_string()));
    assert_eq!(lxml_element.attrib["rel"], "author");
}

/// Selects a large block of text and checks that Skyscraper handles text the same as lxml.
#[test]
fn test_text_handling() {
    // arrange
    let html_text = GITHUB_HTML.to_string();
    let xpath = r#"//div[@role='tabpanel']"#;

    let html_document = html::parse(&html_text).unwrap();
    let xpath_expr = xpath::parse(xpath).unwrap();

    // act
    let lxml_elements = get_lxml_elements(xpath, html_text);
    let skyscraper_elements = xpath_expr.apply(&html_document).unwrap();

    // assert
    let converted_skyscraper_elems =
        skyscraper_to_lxml_elements(&html_document, skyscraper_elements);

    compare_skyscraper_to_lxml(lxml_elements, converted_skyscraper_elems);
}

#[test]
fn test_text_handling2() {
    // arrange
    let html_text = GITHUB_HTML.to_string();
    let xpath = r#"//h2"#;

    let html_document = html::parse(&html_text).unwrap();
    let xpath_expr = xpath::parse(xpath).unwrap();

    // act
    let lxml_elements = get_lxml_elements(xpath, html_text);
    let skyscraper_elements = xpath_expr.apply(&html_document).unwrap();

    // assert
    let converted_skyscraper_elems =
        skyscraper_to_lxml_elements(&html_document, skyscraper_elements);

    compare_skyscraper_to_lxml(lxml_elements, converted_skyscraper_elems);
}

#[test]
fn test_text_handling3() {
    // arrange
    let html_text = GITHUB_HTML.to_string();
    let xpath = r#"//div"#;

    let html_document = html::parse(&html_text).unwrap();
    let xpath_expr = xpath::parse(xpath).unwrap();

    // act
    let lxml_elements = get_lxml_elements(xpath, html_text);
    let skyscraper_elements = xpath_expr.apply(&html_document).unwrap();

    // assert
    let converted_skyscraper_elems =
        skyscraper_to_lxml_elements(&html_document, skyscraper_elements);

    compare_skyscraper_to_lxml(lxml_elements, converted_skyscraper_elems);
}

#[test]
fn test_item_count1() {
    // arrange
    let html_text = GITHUB_HTML.to_string();
    let xpath = "//div[@class='flex-auto min-width-0 width-fit mr-3']";

    let html_document = html::parse(&html_text).unwrap();
    let xpath_expr = xpath::parse(xpath).unwrap();

    // act
    let lxml_output = get_lxml_output(xpath, html_text, true);
    let skyscraper_elements = xpath_expr.apply(&html_document).unwrap();

    // assert
    let output = String::from_utf8_lossy(&lxml_output.stdout);
    let lxml_count = output.trim().parse::<usize>().unwrap();
    assert_eq!(lxml_count, skyscraper_elements.len());
}

#[allow(dead_code)]
fn debug_xpath_tree(xpath_item_tree: &XpathItemTree) {
    let xpath_iter = xpath_item_tree.iter();
    for node in xpath_iter {
        if let Ok(element) = node.as_element_node() {
            if element.name == "h2" {
                println!(
                    "{:?}",
                    element.itertext(&xpath_item_tree).collect::<Vec<_>>()
                );
            }
        }
    }
}

#[allow(dead_code)]
fn debug_html(html_document: html::HtmlDocument) {
    let html_iter = html_document.iter();
    for node in html_iter {
        let html_node = html_document.get_html_node(&node).unwrap();
        if let HtmlNode::Tag(tag) = html_node {
            if tag.name == "h2" {
                println!("{:?}", tag.get_all_text(&node, &html_document));
            }
        }
    }
}
