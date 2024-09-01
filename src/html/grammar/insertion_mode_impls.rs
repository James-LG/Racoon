use std::vec;

use crate::{
    html::grammar::{tokenizer::TokenizerState, NodeOrMarker, SPECIAL_ELEMENTS},
    xpath::grammar::{
        data_model::{AttributeNode, ElementNode},
        XpathItemTreeNode,
    },
};

use super::{
    chars,
    tokenizer::{HtmlToken, Parser, TagToken, TagTokenType},
    Acknowledgement, HtmlParseError, HtmlParser, HtmlParserError, InsertionMode, HTML_NAMESPACE,
};

impl HtmlParser {
    /// <https://html.spec.whatwg.org/multipage/parsing.html#the-initial-insertion-mode>
    pub(super) fn initial_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        match token {
            HtmlToken::Character(
                chars::CHARACTER_TABULATION
                | chars::LINE_FEED
                | chars::FORM_FEED
                | chars::CARRIAGE_RETURN
                | chars::SPACE,
            ) => {
                // ignore
            }
            HtmlToken::Comment(_) => todo!(),
            HtmlToken::DocType(_) => {
                // TODO: Implement this section. No-op is good enough for now, but there's lots to do here.
                self.insertion_mode = InsertionMode::BeforeHtml;
            }
            _ => {
                // TODO: If the document is not an iframe srcdoc document, then this is a parse error;
                //       if the parser cannot change the mode flag is false, set the Document to quirks mode.

                self.insertion_mode = InsertionMode::BeforeHtml;
                self.token_emitted(token)?;
            }
        }

        Ok(Acknowledgement::no())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#the-before-html-insertion-mode>
    pub(super) fn before_html_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        fn anything_else(parser: &mut HtmlParser, token: HtmlToken) -> Result<(), HtmlParseError> {
            let result = parser.create_element(String::from("html"), HTML_NAMESPACE, None, None)?;

            // append the node to the document
            let node_id = parser.new_node(XpathItemTreeNode::ElementNode(result));
            parser
                .root_node
                .expect("root node is None")
                .append(node_id, &mut parser.arena);

            parser.open_elements.push(node_id);

            parser.insertion_mode = InsertionMode::BeforeHead;
            parser.token_emitted(token)?;

            Ok(())
        }

        match token {
            HtmlToken::DocType(_) => todo!(),
            HtmlToken::Comment(token) => {
                let parent = self
                    .root_node
                    .ok_or(HtmlParseError::new("root node is None"))?;

                self.insert_a_comment(token, Some(parent))?;
            }
            HtmlToken::Character(
                chars::CHARACTER_TABULATION
                | chars::LINE_FEED
                | chars::FORM_FEED
                | chars::CARRIAGE_RETURN
                | chars::SPACE,
            ) => {
                // ignore
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "html" => {
                let result = self.create_an_element_for_the_token(token, HTML_NAMESPACE)?;

                // insert the result
                let node_id = self.insert_create_an_element_for_the_token_result(result)?;

                // append it to the document
                self.root_node
                    .expect("root node is None")
                    .append(node_id, &mut self.arena);

                self.insertion_mode = InsertionMode::BeforeHead;
            }
            HtmlToken::TagToken(TagTokenType::EndTag(TagToken { tag_name, .. }))
                if ["head", "body", "html", "br"].contains(&tag_name.as_ref()) =>
            {
                anything_else(
                    self,
                    HtmlToken::TagToken(TagTokenType::EndTag(TagToken::new(tag_name))),
                )?;
            }
            HtmlToken::TagToken(TagTokenType::EndTag(_)) => {
                todo!()
            }
            _ => {
                anything_else(self, token)?;
            }
        }

        Ok(Acknowledgement::no())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#the-before-head-insertion-mode>
    pub(super) fn before_head_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        fn anything_else(parser: &mut HtmlParser, token: HtmlToken) -> Result<(), HtmlParseError> {
            let node_id = parser.insert_an_html_element(TagToken::new(String::from("head")))?;

            parser.head_element_pointer = Some(node_id);

            parser.insertion_mode = InsertionMode::InHead;
            parser.token_emitted(token)?;

            Ok(())
        }

        match token {
            HtmlToken::Character(
                chars::CHARACTER_TABULATION
                | chars::LINE_FEED
                | chars::FORM_FEED
                | chars::CARRIAGE_RETURN
                | chars::SPACE,
            ) => {
                // ignore
            }
            HtmlToken::Comment(_) => todo!(),
            HtmlToken::DocType(_) => todo!(),
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "html" => {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "head" => {
                let node_id = self.insert_an_html_element(token)?;

                self.head_element_pointer = Some(node_id);

                self.insertion_mode = InsertionMode::InHead;
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token))
                if ["head", "body", "html", "br"].contains(&token.tag_name.as_ref()) =>
            {
                anything_else(self, HtmlToken::TagToken(TagTokenType::EndTag(token)))?;
            }
            HtmlToken::TagToken(TagTokenType::EndTag(_)) => {
                todo!()
            }
            _ => anything_else(self, token)?,
        }

        Ok(Acknowledgement::no())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inhead>
    pub(super) fn in_head_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        fn anything_else(parser: &mut HtmlParser, token: HtmlToken) -> Result<(), HtmlParseError> {
            parser.open_elements.pop().expect("open elements is empty");

            parser.insertion_mode = InsertionMode::AfterHead;

            parser.token_emitted(token)?;

            Ok(())
        }
        match token {
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
                self.insert_character(vec![c])?;
            }
            HtmlToken::Comment(comment) => {
                self.insert_a_comment(comment, None)?;
            }
            HtmlToken::DocType(_) => todo!(),
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "html" => {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["base", "basefont", "bgsound", "link"].contains(&token.tag_name.as_str()) =>
            {
                self.insert_an_html_element(token)?;

                self.open_elements.pop().expect("open elements is empty");

                // acknowledge the self closing tag
                return Ok(Acknowledgement::yes());
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "meta" => {
                self.insert_an_html_element(token)?;

                self.open_elements.pop().expect("open elements is empty");

                // TODO: some encoding stuff

                // acknowledge the self closing tag
                return Ok(Acknowledgement::yes());
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "title" => {
                return self.generic_rcdata_element_parsing_algorithm(token);
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["noframes", "style"].contains(&token.tag_name.as_str()) =>
            {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "noscript" => {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "script" => {
                let node = self.insert_an_html_element(token)?;

                // TODO: lots of script and template stuff

                self.open_elements.push(node);

                self.original_insertion_mode = Some(self.insertion_mode);
                self.insertion_mode = InsertionMode::Text;

                // set tokenizer state to script data state
                return Ok(Acknowledgement {
                    self_closed: false,
                    tokenizer_state: Some(TokenizerState::ScriptData),
                });
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "head" => {
                self.open_elements.pop().expect("open elements is empty");

                self.insertion_mode = InsertionMode::AfterHead;
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token))
                if ["body", "html", "br"].contains(&token.tag_name.as_str()) =>
            {
                anything_else(self, HtmlToken::TagToken(TagTokenType::EndTag(token)))?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "template" => {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "template" => {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "head" => {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::EndTag(_)) => {
                todo!()
            }
            _ => {
                anything_else(self, token)?;
            }
        }

        Ok(Acknowledgement::no())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#the-after-head-insertion-mode>
    pub(super) fn after_head_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        fn anything_else(parser: &mut HtmlParser, token: HtmlToken) -> Result<(), HtmlParseError> {
            parser.insert_an_html_element(TagToken::new(String::from("body")))?;

            parser.insertion_mode = InsertionMode::InBody;

            parser.token_emitted(token)?;

            Ok(())
        }
        match token {
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
                self.insert_character(vec![c])?;
            }
            HtmlToken::Comment(_) => todo!(),
            HtmlToken::DocType(_) => todo!(),
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "html" => {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "body" => {
                self.insert_an_html_element(token)?;

                self.frameset_ok = false;

                self.insertion_mode = InsertionMode::InBody;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "frameset" => {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if [
                    "base", "basefont", "bgsound", "link", "meta", "noframes", "script", "style",
                    "template", "title",
                ]
                .contains(&token.tag_name.as_str()) =>
            {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "template" => {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token))
                if ["body", "html", "br"].contains(&token.tag_name.as_str()) =>
            {
                anything_else(self, HtmlToken::TagToken(TagTokenType::EndTag(token)))?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "head" => {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::EndTag(_)) => {
                todo!()
            }
            _ => {
                anything_else(self, token)?;
            }
        }

        Ok(Acknowledgement::no())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inbody>
    pub(super) fn in_body_insertion_mode(
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
            HtmlToken::Comment(_) => {
                todo!()
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
                if !self.stack_of_template_insertion_modes.is_empty() {
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
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["pre", "listing"].contains(&token.tag_name.as_str()) =>
            {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "form" => {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "li" => {
                todo!()
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
                todo!()
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
                todo!()
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
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token))
                if ["dd", "dt"].contains(&token.tag_name.as_str()) =>
            {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token))
                if ["h1", "h2", "h3", "h4", "h5", "h6"].contains(&token.tag_name.as_str()) =>
            {
                todo!()
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
                    .any(|node| {
                        if let XpathItemTreeNode::ElementNode(element) =
                            self.arena.get(*node).unwrap().get()
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
                    todo!("adoption agency algorithm")
                }

                self.reconstruct_the_active_formatting_elements()?;

                let element_id = self.insert_an_html_element(token)?;
                self.push_onto_the_list_of_active_formatting_elements(element_id)?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if [
                    "b", "big", "code", "em", "font", "i", "s", "small", "strike", "strong", "tt",
                    "u",
                ]
                .contains(&token.tag_name.as_str()) =>
            {
                todo!()
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
                todo!()
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
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token))
                if ["area", "br", "embed", "img", "keygen", "wbr"]
                    .contains(&token.tag_name.as_str()) =>
            {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "input" => {
                todo!()
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
                todo!()
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
                todo!()
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
                let node = self.current_node_as_element_error()?.clone();

                self.in_body_other_end_tag_loop(&node, token)?;
            }
        }

        Ok(Acknowledgement::no())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-incdata>
    pub(super) fn text_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        match token {
            HtmlToken::Character(c) => {
                self.insert_character(vec![c])?;
            }
            HtmlToken::EndOfFile => {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "script" => {
                let script = self.current_node_as_element_error()?;

                self.open_elements.pop().expect("open elements is empty");

                self.insertion_mode = self
                    .original_insertion_mode
                    .expect("original insertion mode is None");

                // lots of unsupported scripting logic would go here
                // it is intentionally not included
            }
            HtmlToken::TagToken(TagTokenType::EndTag(_token)) => {
                self.open_elements.pop().expect("open elements is empty");

                self.insertion_mode = self.original_insertion_mode.unwrap();
            }
            _ => {
                // ignore
            }
        }

        Ok(Acknowledgement::no())
    }

    fn in_body_other_end_tag_loop(
        &mut self,
        node: &ElementNode,
        token: TagToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        while node.name == token.tag_name {
            self.generate_implied_end_tags(Some(&token.tag_name))?;

            if node != self.current_node_as_element().unwrap() {
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "node is not the same as the current node",
                )))?;
            }

            // pop all nodes from the current node up to node
            while node != self.current_node_as_element_error()? {
                self.open_elements.pop();
            }

            // should now be the same as node, pop it as well
            self.open_elements.pop();

            // stop these steps
            return Ok(Acknowledgement::no());
        }

        // if node is in special category, parse error and ignore token
        if SPECIAL_ELEMENTS.contains(&node.name.as_str()) {
            self.handle_error(HtmlParserError::MinorError(String::from(
                "node is in special category",
            )))?;
            return Ok(Acknowledgement::no());
        }

        let node = self.current_node_as_element_error()?.clone();
        self.in_body_other_end_tag_loop(&node, token)?;

        Ok(Acknowledgement::no())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-afterbody>
    pub(super) fn after_body_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        match token {
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
                self.using_the_rules_for(token, InsertionMode::InBody)?;
            }
            HtmlToken::Comment(_) => {
                todo!()
            }
            HtmlToken::DocType(_) => {
                todo!()
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "html" => {
                self.using_the_rules_for(
                    HtmlToken::TagToken(TagTokenType::StartTag(token)),
                    InsertionMode::InBody,
                )?;
            }
            HtmlToken::TagToken(TagTokenType::EndTag(token)) if token.tag_name == "html" => {
                // TODO: If parser was created as part of the HTML fragment parsing algorithm...

                self.insertion_mode = InsertionMode::AfterAfterBody;
            }
            HtmlToken::EndOfFile => {
                self.stop_parsing()?;
            }
            _ => {
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "unexpected token after body",
                )))?;

                self.insertion_mode = InsertionMode::InBody;
                self.token_emitted(token)?;
            }
        }

        Ok(Acknowledgement::no())
    }

    /// <https://html.spec.whatwg.org/multipage/parsing.html#the-after-after-body-insertion-mode>
    pub(super) fn after_after_body_insertion_mode(
        &mut self,
        token: HtmlToken,
    ) -> Result<Acknowledgement, HtmlParseError> {
        match token {
            HtmlToken::Comment(_) => {
                todo!()
            }
            HtmlToken::DocType(_)
            | HtmlToken::Character(
                chars::CHARACTER_TABULATION
                | chars::LINE_FEED
                | chars::FORM_FEED
                | chars::CARRIAGE_RETURN
                | chars::SPACE,
            ) => {
                self.using_the_rules_for(token, InsertionMode::InBody)?;
            }
            HtmlToken::TagToken(TagTokenType::StartTag(token)) if token.tag_name == "html" => {
                self.using_the_rules_for(
                    HtmlToken::TagToken(TagTokenType::StartTag(token)),
                    InsertionMode::InBody,
                )?;
            }
            HtmlToken::EndOfFile => {
                self.stop_parsing()?;
            }
            _ => {
                self.handle_error(HtmlParserError::MinorError(String::from(
                    "unexpected token after after body",
                )))?;

                self.insertion_mode = InsertionMode::InBody;
                self.token_emitted(token)?;
            }
        }

        Ok(Acknowledgement::no())
    }
}
