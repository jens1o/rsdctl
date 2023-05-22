#![feature(iter_intersperse)]

use anyhow::Result;

mod gui;
mod wikipedia_api;
mod article_parser;

use crate::article_parser::{Section, Token};

#[allow(dead_code)]
fn print_tokens(tokens: &Vec<Token>) {
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
}

#[allow(dead_code)]
fn print_sections(sections: &Vec<Section>) {
    for section in sections {
        match section {
            Section::Paragraph(tokens) => {
                print_tokens(tokens);
                print!("\n\n");
            }

            Section::Heading(level, tokens) => {
                let mut heading_marker = String::from(" ");
                for _ in 0..*level { heading_marker += "="; }
                heading_marker += " ";

                print!("{}", heading_marker);
                print_tokens(tokens);
                print!("{}\n\n", heading_marker);
            }

            Section::OrderedList(items) => {
                for (i, item) in items.iter().enumerate() {
                    print!("{}. ", i);
                    print_sections(item);
                }
            }

            Section::UnorderedList(items) => {
                for item in items {
                    print!(" * ");
                    print_sections(item);
                }
            }
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    gui::launch()
}
