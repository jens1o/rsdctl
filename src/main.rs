use anyhow::Result;
use parse_wiki_text::{Configuration, Node};
use reqwest::blocking as reqwest;
use serde_json::Value;

#[derive(Debug)]
enum Token {
    Word(String),
    NonWord(String),
}

#[derive(Debug)]
enum Section {
    Heading(usize, Vec<Token>),
    Paragraph(Vec<Token>),
    UnorderedList(Vec<Vec<Section>>),
    OrderedList(Vec<Vec<Section>>),
}

fn chop_into_tokens(input: &str) -> Vec<Token> {

    let mut result: Vec<Token> = Vec::new();
    let mut current: Vec<char> = Vec::new();

    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    // Check for a possible NonWord at the beginning.
    while i < chars.len() && !chars[i].is_alphanumeric() {
        current.push(chars[i]);
        i += 1;
    }

    if current.len() > 0 {
        result.push(Token::NonWord(current.into_iter().collect()));
    }

    if i == chars.len() { return result; }

    loop {
        // Now, we alternative between Words and NonWords until the end.

        let mut current: Vec<char> = Vec::new();

        while i < chars.len() && chars[i].is_alphanumeric() {
            current.push(chars[i]);
            i += 1;
        }
        assert!(current.len() > 0);
        result.push(Token::Word(current.into_iter().collect()));

        if i == chars.len() { break; }

        let mut current: Vec<char> = Vec::new();

        while i < chars.len() && !chars[i].is_alphanumeric() {
            current.push(chars[i]);
            i += 1;
        }
        assert!(current.len() > 0);
        result.push(Token::NonWord(current.into_iter().collect()));

        if i == chars.len() { break; }
    }

    result
}

fn get_template_text(template: &Node) -> String {
    let Node::Template { name, parameters, .. } = template else { panic!("Argument must be template") };

    let name = get_inline_text(name);

    match name.as_str() {
        "lang" => {
            parameters
                .get(1)
                .map(|param| { get_inline_text(&param.value) })
                .unwrap_or(String::from(""))
        }

        "abbr" => {
            let short_form =
                parameters
                .get(0)
                .map(|param| { get_inline_text(&param.value) });

            let long_form =
                parameters
                .get(1)
                .map(|param| { get_inline_text(&param.value) });

            match (long_form, short_form) {
                (None, None) => String::from(""),

                (Some(long), None) => String::from(long),

                (None, Some(short)) => String::from(short),

                (Some(long), Some(short)) => format!("{} ({})", long, short)
            }
        }

        _ => {
            println!("Discarding template named {}: {:?}", name, template);
            String::from("")
        }
    }
}

fn get_inline_text(nodes: &Vec<Node>) -> String {
    let mut result: String = String::new();

    for node in nodes {
        match node {
            Node::CharacterEntity { character, .. } => {
                result.push(*character);
            }

            Node::Link { text: nodes, ..} => {
                let link_text = get_inline_text(nodes);
                result.push_str(&link_text);
            }

            Node::Template { .. } => {
                let template_text = get_template_text(node);
                result.push_str(&template_text);
            }

            Node::Text { value, .. } => {
                result.push_str(value);
            }

            _ => {
                println!("Discarding in inline text: {:?}", node);
            }
        }
    }

    result
}

fn get_sections(nodes: &Vec<Node>) -> Vec<Section> {

    let mut result: Vec<Section> = Vec::new();
    let mut current_para = String::new();

    for node in nodes {
        // These nodes end the previous paragraph
        match node {
            Node::Heading { .. } |
            Node::HorizontalDivider { .. } |
            Node::OrderedList { .. } |
            Node::ParagraphBreak { .. } |
            Node::UnorderedList { .. } => {
                if !current_para.trim().is_empty() {
                    result.push(Section::Paragraph(chop_into_tokens(current_para.trim())));
                    current_para = String::new();
                }
            }

            _ => { }
        }

        match node {
            Node::CharacterEntity { character, .. } => {
                current_para.push(*character);
            }

            Node::Heading { level, nodes, .. } => {
                let heading_text = get_inline_text(nodes);
                result.push(Section::Heading(*level as usize, chop_into_tokens(&heading_text)));
            }

            Node::Link { text: nodes, ..} => {
                let link_text = get_inline_text(nodes);
                current_para.push_str(&link_text);
            }

            Node::OrderedList { items, .. } => {
                let mut sections: Vec<Vec<Section>> = Vec::new();

                for item in items {
                    sections.push(get_sections(&item.nodes));
                }

                result.push(Section::OrderedList(sections));
            }

            Node::Template { .. } => {
                let template_text = get_template_text(node);
                current_para.push_str(&template_text);
            }

            Node::Text { value, .. } => {
                current_para.push_str(value);
            }

            Node::UnorderedList { items, .. } => {
                let mut sections: Vec<Vec<Section>> = Vec::new();

                for item in items {
                    sections.push(get_sections(&item.nodes));
                }

                result.push(Section::UnorderedList(sections));
            }

            _ => {
                // discard other nodes
            }
        }
    }

    if !current_para.trim().is_empty() {
        result.push(Section::Paragraph(chop_into_tokens(current_para.trim())));
    }

    result
}

fn get_wikipedia_article(language: &str, title: &str) -> Result<Vec<Section>> {

    let query = format!("https://{}.wikipedia.org/w/api.php?action=parse&page={}&prop=wikitext&formatversion=2&format=json", language, title);

    let content = reqwest::get(query)?;
    let content = content.text()?;
    let content: Value = serde_json::from_str(&content).expect("JSON response from wiki server malformed.");

    let wikitext = content["parse"]["wikitext"].as_str().expect("JSON response did not contain wikitext.");
    let output = Configuration::default().parse(wikitext);

    if !output.warnings.is_empty() {
        println!("Warnings while parsing wiki text:");
        for warn in &output.warnings {
            println!("{} - {}: {}", warn.start, warn.end, warn.message.message());
        }
    }

    let mut result = get_sections(&output.nodes);

    let page_title = content["parse"]["title"].as_str().expect("JSON response did not contain wikitext.");
    result.insert(0, Section::Heading(0, chop_into_tokens(page_title)));

    Ok(result)
}

fn main() -> Result<()> {

    // let wikitext = get_wikipedia_article("en", "Z22 (computer)")?;
    let wikitext = get_wikipedia_article("en", "English language")?;

    println!("-------------------------------------");
    for section in &wikitext {
        match section {
            Section::Paragraph(tokens) => {
                for token in tokens {
                    match token {
                        Token::Word(w) => {
                            let blanked: String = std::iter::repeat('_').take(w.len()).collect();
                            print!("{}", blanked);
                            // print!("{}", w);
                        }
                        Token::NonWord(w) => {
                            print!("{}", w);
                        }
                    }
                }
                print!("\n\n");
            }

            _ => {
                println!("{:?}", section);
            }
        }
    }

    Ok(())
}
