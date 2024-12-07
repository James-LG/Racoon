use indextree::NodeId;

use crate::{
    html::grammar::{tokenizer::TokenizerState, NodeOrMarker, SPECIAL_ELEMENTS, SVG_NAMESPACE},
    xpath::grammar::{
        data_model::{AttributeNode, ElementNode},
        XpathItemTreeNode,
    },
};

use super::{
    super::tokenizer::{HtmlToken, Parser, TagToken, TagTokenType},
    chars, Acknowledgement, HtmlParseError, HtmlParser, HtmlParserError, InsertionMode,
    HTML_NAMESPACE,
};

impl HtmlParser {
    /// <https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inbody>
    pub(crate) fn in_body_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        fn ensure_open_elements_has_valid_element(
            parser: &HtmlParser,
        ) -> Result<(), HtmlParseError> {
            let valid_elements = vec![
                "dd", "dt", "li", "optgroup", "option", "p", "rb", "rp", "rt", "rtc", "tbody",
                "td", "tfoot", "th", "thead", "tr", "body", "html",
            ];

            if !parser
                .open_elements
                .iter()
                .map(|node_id| parser.arena.get(*node_id).unwrap().get())
                .filter_map(|node| node.as_element_node().ok())
                .any(|node| valid_elements.contains(&node.name.as_str()))
            {
                return parser.handle_error(HtmlParserError::MinorError(String::from(
                    "open elements has no valid element",
                )));
            }

            Ok(())
        }

        match token {
            HtmlToken::Character(chars::NULL) => {
                todo!()
            }
            HtmlToken::Character(c)
                if [
                    chars::CHARACTER_TABULATION,
                    chars::LINE_FEED,
                    chars::FORM_FEED,
                    chars::CARRIAGE_RETURN,
                    chars::SPACE,
                ]
                .contains(&c) =>
            {
                self.reconstruct_the_active_formatting_elements()?;

                self.insert_character(vec![c])?;
            }
            HtmlToken::Character(c) => {
                self.reconstruct_the_active_formatting_elements()?;

                self.insert_character(vec![c])?;

                self.frameset_ok = false;
            }
            HtmlToken::Comment(comment) => {
                self.insert_a_comment(comment, None)?;
            }
            HtmlToken::DocType(_) => {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "html" => {
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "html start tag inside body",
                )))?;

                // if there is a template element on the stack, ignore the token
                let elements: Vec<&ElementNode> = self
                    .open_elements_as_nodes()
                    .iter()
                    .filter_map(|node| node.as_element_node().ok())
                    .collect();

                if elements.iter().any(|node| node.name == "template") {
                    return Ok(Acknowledgement::no());
                }

                // for each attribute, check if the attribute is already present on top element of the stack
                let top_element_res = self.top_node().unwrap().as_element_node();

                let top_element = match top_element_res {
                    Ok(node) => node,
                    Err(_) => {
                        self.handle_error(HtmlParserError::MinorError(String::from(
                            "top element is not an element node",
                        )))?;
                        return Ok(Acknowledgement::no());
                    }
                };

                let top_element_attrs = top_element
                    .attributes_arena(&self.arena)
                    .into_iter()
                    .map(|attr| attr.name.to_string())
                    .collect::<Vec<String>>();

                for attribute in token.attributes.into_iter() {
                    // if the element doesn't already have the attribute, add it
                    if !top_element_attrs.contains(&attribute.name) {
                        let top_node_id = *self.open_elements.first().unwrap();

                        let attr_node_id = self.new_node(XpathItemTreeNode::AttributeNode(
                            AttributeNode::new(attribute.name, attribute.value),
                        ));
                        top_node_id.append(attr_node_id, &mut self.arena);
                    }
                }
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if [
                    "base", "basefont", "bgsound", "link", "meta", "noframes", "script", "style",
                    "template", "title",
                ]
                .contains(&token.tag_name.as_str()) =>
            {
                self.using_the_rules_for(
                    HtmlToken::TagToken(TagTokenType::StartTag(token)),
                    InsertionMode::InHead,
                )?;
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "template" => {
                self.using_the_rules_for(
                    HtmlToken::TagToken(TagTokenType::EndTag(token)),
                    InsertionMode::InHead,
                )?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "body" => {
                if !self.has_an_element_in_scope("body") {
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "open elements has no body element in scope",
                    )))?;
                } else {
                    ensure_open_elements_has_valid_element(&self)?;
                }

                self.insertion_mode = InsertionMode::AfterBody;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "frameset" => {
                todo!()
            }
            HtmlToken::EndOfFile => {
                if !self.template_insertion_modes.is_empty() {
                    self.using_the_rules_for(token, InsertionMode::InTemplate)?;
                } else {
                    ensure_open_elements_has_valid_element(&self)?;
                    self.stop_parsing()?;
                }
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "body" => {
                if !self.has_an_element_in_scope("body") {
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "open elements has body element in scope",
                    )))?;
                } else {
                    ensure_open_elements_has_valid_element(&self)?;
                }

                self.insertion_mode = InsertionMode::AfterBody;
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "html" => {
                if !self.has_an_element_in_scope("body") {
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "open elements has body element in scope",
                    )))?;
                } else {
                    ensure_open_elements_has_valid_element(&self)?;
                }

                self.insertion_mode = InsertionMode::AfterBody;

                self.token_emitted(HtmlToken::TagToken(TagTokenType::EndTag(token)))?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if [
                    "address",
                    "article",
                    "aside",
                    "blockquote",
                    "center",
                    "details",
                    "dialog",
                    "dir",
                    "div",
                    "dl",
                    "fieldset",
                    "figcaption",
                    "figure",
                    "footer",
                    "header",
                    "hgroup",
                    "main",
                    "menu",
                    "nav",
                    "ol",
                    "p",
                    "search",
                    "section",
                    "summary",
                    "ul",
                ]
                .contains(&token.tag_name.as_str()) =>
            {
                if self.has_an_element_in_button_scope("p") {
                    self.close_a_p_element()?;
                }

                self.insert_an_html_element(token)?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["h1", "h2", "h3", "h4", "h5", "h6"].contains(&token.tag_name.as_str()) =>
            {
                if self.has_an_element_in_button_scope("p") {
                    self.close_a_p_element()?;
                }

                if let Some(element) = self.current_node_as_element() {
                    if ["h1", "h2", "h3", "h4", "h5", "h6"].contains(&element.name.as_str()) {
                        self.handle_error(HtmlParserError::MinorError(String::from(
                            "current node is h1, h2, h3, h4, h5, or h6",
                        )))?;
                        self.open_elements.pop();
                    }
                }

                self.insert_an_html_element(token)?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["pre", "listing"].contains(&token.tag_name.as_str()) =>
            {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "form" => {
                if self.form_element_pointer.is_some()
                    && !self.open_elements_has_element("template")
                {
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "form element pointer is not none and there is no template element",
                    )))?;
                } else {
                    if self.has_an_element_in_button_scope("p") {
                        self.close_a_p_element()?;
                    }

                    let element_id = self.insert_an_html_element(token)?;

                    if !self.open_elements_has_element("template") {
                        self.form_element_pointer = Some(element_id);
                    }
                }
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "li" => {
                fn step_3_loop(
                    parser: &mut HtmlParser,
                    element: &ElementNode,
                    token: TagToken,
                ) -> Result<(), HtmlParseError> {
                    if element.name == "li" {
                        parser.generate_implied_end_tags(Some("li"))?;

                        if parser.current_node_as_element_result()?.name != "li" {
                            parser.handle_error(HtmlParserError::MinorError(String::from(
                                "current node is not li",
                            )))?;
                        }

                        parser.pop_until_tag_name("li")?;
                    }

                    if SPECIAL_ELEMENTS.contains(&element.name.as_str())
                        && !["address", "div", "p"].contains(&element.name.as_str())
                    {
                        step_6_done(parser, token)?;
                    } else {
                        let current_element_index = parser
                            .open_elements
                            .iter()
                            .position(|node_id| node_id == &element.id())
                            .expect("current element is not in open elements");

                        let previous_element_id = parser
                            .open_elements
                            .get(current_element_index - 1)
                            .expect("previous element is not in open elements");

                        let previous_element = parser
                            .arena
                            .get(*previous_element_id)
                            .unwrap()
                            .get()
                            .as_element_node()
                            .map_err(|_| {
                                HtmlParserError::MinorError(String::from(
                                    "previous element is not an element node",
                                ))
                            });

                        match previous_element {
                            Err(_) => {
                                parser.handle_error(HtmlParserError::MinorError(String::from(
                                    "previous element is not an element node",
                                )))?;
                            }
                            Ok(previous_element) => {
                                let previous_element = previous_element.clone();
                                return step_3_loop(parser, &previous_element, token);
                            }
                        }
                    }

                    Ok(())
                }

                fn step_6_done(
                    parser: &mut HtmlParser,
                    token: TagToken,
                ) -> Result<(), HtmlParseError> {
                    if parser.has_an_element_in_button_scope("p") {
                        parser.close_a_p_element()?;
                    }

                    parser.insert_an_html_element(token)?;

                    Ok(())
                }

                self.frameset_ok = false;

                let node = self.current_node_as_element_result()?.clone();
                step_3_loop(self, &node, token)?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["dd", "dt"].contains(&token.tag_name.as_str()) =>
            {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "plaintext" => {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "button" => {
                if self.has_an_element_in_scope("button") {
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "open elements has button element in scope",
                    )))?;

                    self.generate_implied_end_tags(None)?;

                    self.pop_until_tag_name("button")?;
                }

                self.reconstruct_the_active_formatting_elements()?;
                self.insert_an_html_element(token)?;
                self.frameset_ok = false;
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token))
                if [
                    "address",
                    "article",
                    "aside",
                    "blockquote",
                    "button",
                    "center",
                    "details",
                    "dialog",
                    "dir",
                    "div",
                    "dl",
                    "fieldset",
                    "figcaption",
                    "figure",
                    "footer",
                    "header",
                    "hgroup",
                    "listing",
                    "main",
                    "menu",
                    "nav",
                    "ol",
                    "pre",
                    "section",
                    "summary",
                    "ul",
                ]
                .contains(&token.tag_name.as_str()) =>
            {
                if !self.has_an_element_in_scope(&token.tag_name) {
                    self.handle_error(HtmlParserError::MinorError(String::from(format!(
                        "open elements has {} element in scope",
                        token.tag_name
                    ))))?;
                } else {
                    self.generate_implied_end_tags(None)?;

                    if self.current_node_as_element().unwrap().name != token.tag_name {
                        self.handle_error(HtmlParserError::MinorError(String::from(
                            "current node is not the same as the token tag name",
                        )))?;
                    }

                    self.pop_until_tag_name(&token.tag_name)?;
                }
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "form" => {
                if !self.open_elements_has_element("template") {
                    let node = self.form_element_pointer;
                    self.form_element_pointer = None;

                    match node {
                        Some(node) => {
                            let element = self
                                .arena
                                .get(node)
                                .expect("form element pointer is none")
                                .get()
                                .as_element_node()
                                .expect("form element pointer is not an element node")
                                .clone();

                            if !self.has_an_element_in_scope(&element.name) {
                                self.handle_error(HtmlParserError::MinorError(String::from(
                                    "open elements has no form element in scope",
                                )))?;
                            } else {
                                self.generate_implied_end_tags(None)?;

                                if self.current_node_id_result()? != node {
                                    self.handle_error(HtmlParserError::MinorError(String::from(
                                        "current node is not the same as the form element",
                                    )))?;
                                }

                                // remove node from open elements
                                self.open_elements.retain(|node_id| node_id != &node);
                            }
                        }
                        None => {
                            self.handle_error(HtmlParserError::MinorError(String::from(
                                "form element pointer is none",
                            )))?;

                            return Ok(Acknowledgement::no());
                        }
                    }
                } else {
                    if !self.has_an_element_in_scope("form") {
                        self.handle_error(HtmlParserError::MinorError(String::from(
                            "open elements has no form element in scope",
                        )))?;
                        return Ok(Acknowledgement::no());
                    } else {
                        self.generate_implied_end_tags(None)?;

                        if self.current_node_as_element_result()?.name != "form" {
                            self.handle_error(HtmlParserError::MinorError(String::from(
                                "current node is not form",
                            )))?;
                        }

                        self.pop_until_tag_name("form")?;
                    }
                }
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "p" => {
                if !self.has_an_element_in_button_scope("p") {
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "open elements has no p element in button scope",
                    )))?;

                    self.insert_an_html_element(TagToken::new(String::from("p")))?;
                }

                self.close_a_p_element()?;
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "li" => {
                if !self.has_an_element_in_list_item_scope("li") {
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "open elements has no li element in list item scope",
                    )))?;
                } else {
                    self.generate_implied_end_tags(Some("li"))?;

                    if self.current_node_as_element().unwrap().name != "li" {
                        self.handle_error(HtmlParserError::MinorError(String::from(
                            "current node is not li",
                        )))?;
                    }

                    self.pop_until_tag_name("li")?;
                }
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token))
                if ["dd", "dt"].contains(&token.tag_name.as_str()) =>
            {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token))
                if ["h1", "h2", "h3", "h4", "h5", "h6"].contains(&token.tag_name.as_str()) =>
            {
                if !self
                    .has_an_element_in_scope_by_tag_names(vec!["h1", "h2", "h3", "h4", "h5", "h6"])
                {
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "open elements has no h1, h2, h3, h4, h5, or h6 element in scope",
                    )))?;

                    return Ok(Acknowledgement::no());
                }

                self.generate_implied_end_tags(None)?;
                if self.current_node_as_element_result()?.name != token.tag_name {
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "current node is not the same as the token tag name",
                    )))?;
                }

                self.pop_until_tag_name_one_of(vec!["h1", "h2", "h3", "h4", "h5", "h6"])?;
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "sarcasm" => {
                // "Take a deep breath, then act as described in the 'any other end tag' entry below." lol
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "a" => {
                // if the active formatting element list contains an `a` element between the end of the list and the last marker
                // then this is a parse error
                if self
                    .active_formatting_elements
                    .iter()
                    .rev()
                    .map_while(|node| {
                        if let NodeOrMarker::Node(node) = node {
                            Some(node)
                        } else {
                            None
                        }
                    })
                    .any(|entry| {
                        if let XpathItemTreeNode::ElementNode(element) =
                            self.arena.get(entry.node_id).unwrap().get()
                        {
                            element.name == "a"
                        } else {
                            false
                        }
                    })
                {
                    self.handle_error(HtmlParserError::MinorError(String::from(
                        "active formatting elements contains an a element",
                    )))?;
                    self.adoption_agency_algorithm(&token)?;
                }

                self.reconstruct_the_active_formatting_elements()?;

                let element_id = self.insert_an_html_element(token.clone())?;
                self.push_onto_the_list_of_active_formatting_elements(element_id, token)?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if [
                    "a", "b", "big", "code", "em", "font", "i", "s", "small", "strike", "strong",
                    "tt", "u",
                ]
                .contains(&token.tag_name.as_str()) =>
            {
                self.adoption_agency_algorithm(&token)?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "nobr" => {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token))
                if [
                    "a", "b", "big", "code", "em", "font", "i", "nobr", "s", "small", "strike",
                    "strong", "tt", "u",
                ]
                .contains(&token.tag_name.as_str()) =>
            {
                self.adoption_agency_algorithm(&token)?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["applet", "marquee", "object"].contains(&token.tag_name.as_str()) =>
            {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token))
                if ["applet", "marquee", "object"].contains(&token.tag_name.as_str()) =>
            {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "table" => {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "br" => {
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "br end tag in body",
                )))?;

                // drop attributes
                let token = TagToken {
                    tag_name: String::from("br"),
                    attributes: vec![],
                    self_closing: token.self_closing,
                };

                // act as if it was a start tag
                self.reconstruct_the_active_formatting_elements()?;

                let self_closing = token.self_closing;
                self.insert_an_html_element(token)?;
                self.open_elements.pop();

                self.frameset_ok = false;

                if self_closing {
                    return Ok(Acknowledgement::yes());
                }
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["area", "br", "embed", "img", "keygen", "wbr"]
                    .contains(&token.tag_name.as_str()) =>
            {
                self.reconstruct_the_active_formatting_elements()?;

                let self_closing = token.self_closing;
                self.insert_an_html_element(token)?;
                self.open_elements.pop();

                self.frameset_ok = false;

                if self_closing {
                    return Ok(Acknowledgement::yes());
                }
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "input" => {
                self.reconstruct_the_active_formatting_elements()?;

                let self_closing = token.self_closing;
                self.insert_an_html_element(token)?;
                self.open_elements.pop();

                self.frameset_ok = false;
                if self_closing {
                    return Ok(Acknowledgement::yes());
                }
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["param", "source", "track"].contains(&token.tag_name.as_str()) =>
            {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "hr" => {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "image" => {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "textarea" => {
                self.insert_an_html_element(token)?;

                // TODO: if next token is line feed character token, ignore it

                self.original_insertion_mode = Some(self.insertion_mode);
                self.insertion_mode = InsertionMode::Text;
                self.frameset_ok = false;

                return Ok(Acknowledgement {
                    self_closed: false,
                    tokenizer_state: Some(TokenizerState::RCDATA),
                });
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "xmp" => {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "iframe" => {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["noembed", "noscript"].contains(&token.tag_name.as_str()) =>
            {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "select" => {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["optgroup", "option"].contains(&token.tag_name.as_str()) =>
            {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["rb", "rtc"].contains(&token.tag_name.as_str()) =>
            {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["rp", "rt"].contains(&token.tag_name.as_str()) =>
            {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "math" => {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "svg" => {
                self.reconstruct_the_active_formatting_elements()?;

                // TODO: adjust SVG attribtues
                // TODO: adjust foreign attributes

                let self_closing = token.self_closing;
                self.insert_foreign_element(token, SVG_NAMESPACE, false)?;

                if self_closing {
                    self.open_elements.pop();
                    return Ok(Acknowledgement::yes());
                }
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if [
                    "caption", "col", "colgroup", "frame", "head", "tbody", "td", "tfoot", "th",
                    "thead", "tr",
                ]
                .contains(&token.tag_name.as_str()) =>
            {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) => {
                self.reconstruct_the_active_formatting_elements()?;

                self.insert_an_html_element(token)?;
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) => {
                self.other_end_tag(&token)?;
            }
        }

        Ok(Acknowledgement::no())
    }

    fn other_end_tag(&mut self, token: &TagToken) -> Result<(), HtmlParseError> {
        let node = self.current_node_as_element_result()?.clone();

        self.in_body_other_end_tag_loop(0, &node, token)?;

        Ok(())
    }

    fn in_body_other_end_tag_loop(
        &mut self,
        node_index: usize,
        node: &ElementNode,
        token: &TagToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        if node.name == token.tag_name {
            self.generate_implied_end_tags(Some(&token.tag_name))?;

            if node != self.current_node_as_element().unwrap() {
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "node is not the same as the current node",
                )))?;
            }

            // pop all nodes from the current node up to node
            while node != self.current_node_as_element_result()? {
                self.open_elements.pop();
            }

            // should now be the same as node, pop it as well
            self.open_elements.pop();

            // stop these steps
            return Ok(Acknowledgement::no());
        }
        // if node is in special category, parse error and ignore token
        else if SPECIAL_ELEMENTS.contains(&node.name.as_str()) {
            self.handle_error(HtmlParserError::MinorError(String::from(
                "node is in special category",
            )))?;
            return Ok(Acknowledgement::no());
        }

        // set node to the previous entry
        let node = self
            .open_elements
            .iter()
            .rev()
            .skip(node_index)
            .next()
            .map(|node_id| {
                self.arena
                    .get(*node_id)
                    .expect("node not found")
                    .get()
                    .as_element_node()
                    .expect("node is not an element node")
                    .clone()
            })
            .expect("node not found");

        self.in_body_other_end_tag_loop(node_index + 1, &node, token)?;

        Ok(Acknowledgement::no())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#adoption-agency-algorithm>
    pub(crate) fn adoption_agency_algorithm(
        &mut self,
        token: &TagToken,
    ) -> Result<(), HtmlParseError> {
        let subject = token.tag_name.clone();

        if let Some(XpathItemTreeNode::ElementNode(element)) = self.current_node() {
            if element.name == subject {
                self.open_elements.pop();
                return Ok(());
            }
        }

        let mut outer_loop_counter = 0;

        loop {
            if outer_loop_counter >= 8 {
                return Ok(()); // abort the adoption agency algorithm
            }

            outer_loop_counter += 1;

            let mut active_formatting_elements_until_marker =
                self.active_formatting_elements_until_marker();

            let formatting_element =
                match active_formatting_elements_until_marker.find(|node| node.name == subject) {
                    Some(element) => element.clone(),
                    None => {
                        drop(active_formatting_elements_until_marker);
                        // act as the "any other end tag" and return
                        return self.other_end_tag(token);
                    }
                };
            drop(active_formatting_elements_until_marker);

            // if formatting element is not in the open elements
            if !self.open_elements.contains(&formatting_element.id()) {
                // remove the formatting element from the list of active formatting elements
                self.remove_from_active_formatting_elements(formatting_element.id())?;
                return self.handle_error(HtmlParserError::MinorError(String::from(
                    "formatting element is not in open elements",
                )));
            }

            // if formatting element is not in scope
            if !self.has_an_element_in_scope(&formatting_element.name) {
                return self.handle_error(HtmlParserError::MinorError(String::from(
                    "formatting element is not in scope",
                )));
            }

            // if formatting element is not the current node
            if formatting_element.id() != self.current_node_id_result()? {
                // parse error, but do _not_ return
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "formatting element is not the current node",
                )))?;
            }

            let formatting_element_index_in_open_elements = self
                .open_elements
                .iter()
                .position(|node_id| node_id == &formatting_element.id())
                .unwrap();

            let lower_than_formatting_element = self
                .open_elements
                .iter()
                .skip(formatting_element_index_in_open_elements + 1)
                .map(|node_id| *node_id)
                .collect::<Vec<NodeId>>();

            let furthest_block = lower_than_formatting_element
                .iter()
                .find(|node_id| {
                    if let XpathItemTreeNode::ElementNode(element) =
                        self.arena.get(**node_id).unwrap().get()
                    {
                        SPECIAL_ELEMENTS.contains(&element.name.as_str())
                    } else {
                        false
                    }
                })
                .map(|node_id| *node_id);

            let current_node_id = self.current_node_id_result()?;
            let furthest_block = match furthest_block {
                Some(node_id) => node_id,
                None => {
                    // pop all nodes from the current node up to and including the formatting element
                    while current_node_id != formatting_element.id() {
                        self.open_elements.pop();
                    }

                    self.open_elements.pop();

                    // remove the formatting element from the list of active formatting elements
                    self.remove_from_active_formatting_elements(formatting_element.id())?;
                    return Ok(());
                }
            };

            let common_ancestor = self
                .open_elements
                .get(formatting_element_index_in_open_elements - 1);

            // TODO: bookmark?

            todo!()
        }
    }
}
