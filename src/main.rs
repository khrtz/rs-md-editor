use eframe::egui;
use egui::{ScrollArea, TextEdit, TextStyle};
use std::fs::{File, OpenOptions};
use std::io::BufReader;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use pulldown_cmark::{Parser, Event, Tag};

#[derive(Serialize, Deserialize)]
struct MarkdownEditor {
    text: String,
    file_path: PathBuf,
    scroll_offset: f32,
    show_preview: bool,
}

impl Default for MarkdownEditor {
    fn default() -> Self {
        let mut desktop_path = dirs::desktop_dir().expect("Could not find desktop directory");
        desktop_path.push("markdown_editor_content.json");
        Self {
            text: "# Welcome to Markdown Editor\n\nStart typing your markdown here!".to_owned(),
            file_path: desktop_path,
            scroll_offset: 0.0,
            show_preview: true,
        }
    }
}

impl MarkdownEditor {

    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut editor = Self::default();
        editor.load_content();
        editor.show_preview = true;
        editor
    }

    fn save_changes(&self) {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.file_path)
            .expect("Unable to open file");

        serde_json::to_writer(file, self).expect("Unable to write to file");
    }

    fn load_content(&mut self) {
        if let Ok(file) = File::open(&self.file_path) {
            let reader = BufReader::new(file);
            if let Ok(content) = serde_json::from_reader(reader) {
                *self = content;
            }
        }
    }
}

fn highlight_markdown(ui: &mut egui::Ui, text: &str) {
    let parser = Parser::new(text);
    let mut color = egui::Color32::WHITE;
    let mut font_size = 14.0;
    let mut italics = false;
    let mut bold = false;
    let mut list_level = 0;
    let mut in_item = false;

    for event in parser {
        match event {
            Event::Start(Tag::Heading(level, _, _)) => {
                font_size = 24.0 - (level as u8 as f32 * 2.0);
                color = egui::Color32::LIGHT_BLUE;
                in_item = false;
                ui.end_row();
            },
            Event::Start(Tag::Paragraph) => {
                if !in_item {
                    font_size = 14.0;
                    color = egui::Color32::WHITE;
                }
            },
            Event::Start(Tag::List(_)) => {
                list_level += 1;
            },
            Event::End(Tag::List(_)) => {
                list_level -= 1;
                in_item = false;
            },
            Event::Start(Tag::Item) => {
                in_item = true;
            },
            Event::Text(text) => {
                let mut rich_text = egui::RichText::new(text.to_string())
                    .color(color)
                    .size(font_size);
            
                if italics {
                    rich_text = rich_text.italics();
                }
                if bold {
                    rich_text = rich_text.strong();
                }
            
                if in_item {
                    ui.horizontal(|ui| {
                        ui.add_space((list_level - 1) as f32 * 20.0);
                        ui.label("• ");
                        ui.label(rich_text);
                    });
                } else {
                    ui.label(rich_text);
                }
            },
            
            Event::End(Tag::Item) => {
                in_item = false;
            },
            Event::SoftBreak | Event::HardBreak => {
                ui.end_row();
            },
            Event::End(_) => {
                color = egui::Color32::WHITE;
                font_size = 14.0;
                italics = false;
                bold = false;
            },
            _ => {}
        }
    }
}

impl eframe::App for MarkdownEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let available_size = ui.available_size();
            let half_width = available_size.x / 2.0;

            ui.horizontal(|ui| {
                // 左側：エディター
                ui.vertical(|ui| {
                    ui.set_width(half_width);
                    ui.horizontal(|ui| {
                        // 行番号
                        let line_count = self.text.lines().count().max(1);
                        ui.vertical(|ui| {
                            ui.set_width(30.0);
                            ui.style_mut().spacing.item_spacing.y = 0.0;
                            for i in 1..=line_count {
                                ui.label(egui::RichText::new(format!("{:3}", i)).monospace());
                            }
                        });

                        // テキストエディタ
                        let editor_width = half_width - 40.0;
                        let response = ui.add_sized(
                            [editor_width, available_size.y - 20.0],
                            TextEdit::multiline(&mut self.text)
                                .font(TextStyle::Monospace)
                                .frame(false)
                        );

                        if response.changed() {
                            self.save_changes();
                        }
                    });
                });

                // 右側：プレビュー
                ui.vertical(|ui| {
                    ui.set_width(half_width);
                    ui.label("Preview:");
                    ScrollArea::vertical()
                        .id_source("preview_scroll")
                        .show(ui, |ui| {
                            highlight_markdown(ui, &self.text);
                        });
                });
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Markdown Editor",
        options,
        Box::new(|cc| Ok(Box::new(MarkdownEditor::new(cc))))
    )
}