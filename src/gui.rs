use eframe::egui;

use crate::wiki;
use crate::wiki::{Token, Section};

struct App {
    wiki_article: Option<Vec<Section>>,
    guesses: Vec<String>,
}

impl App {
    fn load_clicked(&mut self) {
        let maybe_article = wiki::load_article("en", "English language");

        match maybe_article {
            Ok(sections) => {
                self.wiki_article = Some(sections);
            }

            Err(_) => {
                self.wiki_article =
                    None;
            }
        }
    }

    fn concat_tokens(&self, tokens: &Vec<Token>) -> String {
        let mut result = String::new();

        for token in tokens {
            match token {
                Token::Word(w) => {
                    let blanked: String = std::iter::repeat('_').take(w.len()).collect();
                    result.push_str(w);
                    // ui.label(blanked);
                }
                Token::NonWord(w) => {
                    result.push_str(w);
                }
            }
        }

        println!("Paragraph text: {}", result);
        result
    }

    fn show_article(&self, ui: &mut egui::Ui) {
        if let Some(wiki_article) = &self.wiki_article {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for section in wiki_article {
                    match section {
                        Section::Paragraph(tokens) => {
                            let text = self.concat_tokens(&tokens);
                            ui.label(text);
                        }

                        _ => {

                        }
                    }
                }
            });
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            wiki_article: None,
            guesses: Vec::new(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("my_panel").show(ctx, |ui| {
            let load_btn = ui.button("load");

            if load_btn.clicked() {
                self.load_clicked();
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_article(ui);
        });
    }
}

pub fn launch() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };
    eframe::run_native(
        "rsdctl",
        options,
        Box::new(|_cc| Box::<App>::default()),
    )
}
