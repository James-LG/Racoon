# generates rust code for named character references
# https://html.spec.whatwg.org/multipage/named-characters.html#named-character-references

# download the named character references from the whatwg spec
# https://html.spec.whatwg.org/entities.json
# and save it as entities.json

import json

def to_unicode_escape_sequence(codepoint):
    return f"\\u{{{codepoint:04x}}}"

def main():
    test_chars = "\uD835\uDD04"
    print(''.join([to_unicode_escape_sequence(ord(x)) for x in test_chars]))

    with open("entities.json") as f:
        entities = json.load(f)
    
    max_length = max([len(entity) for entity in entities])

    print('&Afr;', entities['&Afr;'], entities['&Afr;']["characters"].encode('unicode_escape').decode('utf-8'))

    with open("../src/html/grammar/tokenizer/named_character_references.rs", "w") as f:
        f.write("use std::collections::HashMap;\n")
        f.write("use once_cell::sync::Lazy;\n")
        f.write("\n")
        f.write("pub(crate) static NAMED_CHARACTER_REFS_MAX_LENGTH: usize = {};\n".format(max_length))
        f.write("\n")
        f.write("pub(crate) static NAMED_CHARACTER_REFS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {\n")
        f.write("    let mut m = HashMap::new();\n")
        for entity in entities:
            escaped_characters = ''.join([to_unicode_escape_sequence(ord(x)) for x in entities[entity]["characters"]])
            f.write("    m.insert(\"{}\", \"{}\");\n".format(entity, escaped_characters))
        f.write("    m\n")
        f.write("});\n")

if __name__ == "__main__":
    main()