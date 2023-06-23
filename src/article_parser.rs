use parse_wiki_text::{Configuration, Node};

#[derive(Debug)]
pub enum Token {
    Word(String),
    NonWord(String),
}

#[derive(Debug)]
pub enum Section {
    Heading(usize, Vec<Token>),
    Paragraph(Vec<Token>),
    UnorderedList(Vec<Vec<Section>>),
    OrderedList(Vec<Vec<Section>>),
}

#[derive(Debug)]
pub struct WikiArticle {
    pub title: Vec<Token>,
    pub content: Vec<Section>,
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

fn get_parameter_by_name(parameters: &Vec<parse_wiki_text::Parameter>, name: &str) -> Option<String> {
    for param in parameters.iter() {
        if let Some(name_nodes) = &param.name {
            let n = get_inline_text(&name_nodes);
            if n == name {
                return Some(get_inline_text(&param.value));
            }
        }
    }
    None
}

fn get_template_text(template: &Node) -> String {
    let Node::Template { name, parameters, .. } = template else { panic!("Argument must be template") };

    let name = get_inline_text(name);

    match name.to_lowercase().as_str() {
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

	"blockquote" => {
            let quote =
                parameters
                .get(0)
                .map(|param| { get_inline_text(&param.value) });

            let source =
                parameters
                .get(1)
                .map(|param| { get_inline_text(&param.value) });

	    let mut result = String::new();

	    if let Some(q) = quote {
		result.push_str("“");
		result.push_str(&q);
		result.push_str("”");

		if let Some(s) = source {
		    result.push_str(" – ");
		    result.push_str(&s);
		}
	    }

	    result
	}

        "cite encyclopedia" => {
            get_parameter_by_name(parameters, &"encyclopedia")
                .unwrap_or(String::from(""))
        }

        "cite book" | "cite journal" | "cite web" | "cite news" | "cite report" | "cite periodical" => {
            get_parameter_by_name(parameters, &"title")
                .unwrap_or(String::from(""))
        }

	"cvt" | "convert" => {
	    let number =
                parameters
                .get(0)
                .map(|param| { get_inline_text(&param.value) });

	    let unit =
                parameters
                .get(1)
                .map(|param| { get_inline_text(&param.value) });

	    match (number, unit) {
		(None, None) => String::from(""),

		(Some(n), None) => format!("{}", n),

		(None, Some(u)) => format!("??? {}", u),

		(Some(n), Some(u)) => format!("{} {}", n, u)
	    }
	}

	"endash" => {
	    String::from("–")
	}

        _ => {
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

            Node::ExternalLink { nodes, ..} => {
                let link_text = get_inline_text(nodes);
                let without_url: String = link_text
                    .split_whitespace()
                    .skip(1)
                    .intersperse(" ")
                    .collect();
                current_para.push_str(&without_url);
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
            }
        }
    }

    if !current_para.trim().is_empty() {
        result.push(Section::Paragraph(chop_into_tokens(current_para.trim())));
    }

    result
}

pub fn parse(title: &str, content: &str) -> WikiArticle {

    let parsed = Configuration::default().parse(content);

    if !parsed.warnings.is_empty() {
        println!("Warnings while parsing wiki text:");
        for warn in &parsed.warnings {
            println!("{} - {}: {}", warn.start, warn.end, warn.message.message());
        }
    }

    let content = get_sections(&parsed.nodes);

    let title_tokens = chop_into_tokens(title);

    WikiArticle{
        title: title_tokens,
        content: content,
    }
}
