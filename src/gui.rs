use eframe::egui;
use eframe::egui::widgets::*;
use egui_notify::{Toasts};

use crate::wiki;
use crate::wiki::{Token, Section};

struct App {
    selected_language: String,
    article_title: String,
    wiki_article: Option<Vec<Section>>,
    guesses: Vec<String>,

    toasts: Toasts,
    next_guess: String,
    focus_on_guess: bool,
}

impl App {
    fn load_article(&mut self) {
        let maybe_article = wiki::load_article(self.selected_language.as_str(),
                                               self.article_title.as_str());

        match maybe_article {
            Ok(sections) => {
                self.wiki_article = Some(sections);
                self.guesses.clear();
                self.next_guess.clear();

                self.article_title.clear();
            }

            Err(e) => {
                self.toasts.error(format!("{}", e));
            }
        }
    }

    fn get_word(&self, word: &str) -> String {
        if self.guesses.contains(&word.to_lowercase()) {
            String::from(word)
        } else {
            std::iter::repeat('_').take(word.len()).collect()
        }
    }

    fn concat_tokens(&self, tokens: &Vec<Token>) -> String {
        let mut result = String::new();

        for token in tokens {
            match token {
                Token::Word(w) => {
                    // result.push_str(w);
                    result.push_str(self.get_word(w).as_str());
                }
                Token::NonWord(w) => {
                    result.push_str(w);
                }
            }
        }

        result
    }

    fn show_top_bar(&mut self, ui: &mut egui::Ui) {
            ui.horizontal(|ui| {
                ui.label("Language code:");

                let language_code = TextEdit::singleline(&mut self.selected_language)
                    .desired_width(30.0);
                ui.add(language_code);

                ui.label("Article:");

                let article_title = TextEdit::singleline(&mut self.article_title);
                let resp = ui.add(article_title);
                if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.load_article();
                }

                let load_btn = ui.button("load");

                if load_btn.clicked() {
                    self.load_article();
                }
            });
    }

    fn show_sections(&self, ui: &mut egui::Ui, sections: &Vec<Section>) {
        for section in sections {
            match section {
                Section::Heading(_level, tokens) => {
                    let text = self.concat_tokens(&tokens);
                    ui.add_space(30.0);
                    ui.heading(text);
                    ui.add_space(10.0);
                }

                Section::Paragraph(tokens) => {
                    let text = self.concat_tokens(&tokens);
                    ui.label(text);
                    ui.add_space(10.0);
                }

                Section::UnorderedList(list_items) => {
                    for item in list_items {
                        ui.horizontal_top(|ui| {
                            ui.label("•");

                            ui.vertical(|ui| {
                                self.show_sections(ui, item);
                            });
                        });
                    }
                }

                Section::OrderedList(list_items) => {
                    for (i, item) in list_items.iter().enumerate() {
                        ui.horizontal_top(|ui| {
                            ui.label(format!("{}.", i + 1));

                            ui.vertical(|ui| {
                                self.show_sections(ui, item);
                            });
                        });
                    }
                }
            }
        }
    }

    fn show_article(&self, ui: &mut egui::Ui) {
        if let Some(wiki_article) = &self.wiki_article {
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.show_sections(ui, wiki_article);
            });
        }
    }

    fn count_word_in_tokens(word: &str, tokens: &Vec<Token>) -> usize {
        let mut result = 0;

        for token in tokens {
            if let Token::Word(w) = token {
                if w.to_lowercase() == word.to_lowercase() {
                    result += 1;
                }
            }
        }
        result
    }

    fn count_word_in_sections(word: &str, sections: &Vec<Section>) -> usize {
        let mut result = 0;

        for section in sections {
            match section {
                Section::Heading(_level, tokens) => {
                    result += Self::count_word_in_tokens(word, tokens);
                }

                Section::Paragraph(tokens) => {
                    result += Self::count_word_in_tokens(word, tokens);
                }

                Section::UnorderedList(list_items) => {
                    for item in list_items {
                        result += Self::count_word_in_sections(word, item);
                    }
                }

                Section::OrderedList(list_items) => {
                    for item in list_items {
                        result += Self::count_word_in_sections(word, item);
                    }
                }
            }
        }
        result
    }

    fn show_guesses(&mut self, ui: &mut egui::Ui) {
        let Some(wiki_article) = &self.wiki_article else {
            return;
        };

        egui::Grid::new("guesses_grid")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                for guess in &self.guesses {
                    let occurs = Self::count_word_in_sections(guess.as_str(), &wiki_article);

                    ui.label(format!("{}", occurs));
                    ui.label(guess);
                    ui.end_row();
                }

                ui.label("");

                let next_guess_edit = TextEdit::singleline(&mut self.next_guess);
                let resp = ui.add(next_guess_edit);

                if self.focus_on_guess {
                    resp.request_focus();
                    self.focus_on_guess = false;
                }

                ui.end_row();

                if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.guesses.push(self.next_guess.to_lowercase());
                    self.next_guess.clear();

                    self.focus_on_guess = true;
                }
            });
    }

    fn show_gui(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.show_top_bar(ui);
        });

        if let Some(_) = self.wiki_article {
            egui::SidePanel::right("right_panel")
                .min_width(200.0)
                .resizable(true)
                .show_separator_line(true)
                .show(ctx, |ui| {
                    self.show_guesses(ui);
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_article(ui);
        });

        self.toasts.show(ctx);
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            selected_language: String::from("en"),
            article_title: String::from(""),
            wiki_article: None,
            guesses: Vec::new(),

            toasts: Toasts::new(),
            next_guess: String::from(""),
            focus_on_guess: false,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.show_gui(ctx, frame);
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
