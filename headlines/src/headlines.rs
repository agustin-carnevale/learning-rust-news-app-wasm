use std::sync::mpsc::{Receiver, SyncSender};

use eframe::egui::{
    menu, Button, CentralPanel, Color32, CtxRef, FontDefinitions, FontFamily, Hyperlink, Key,
    Label, Layout, Separator, TextStyle, TopBottomPanel, Ui, Window,
};
use serde::{Deserialize, Serialize};

pub const PADDING: f32 = 5.0;
const WHITE: Color32 = Color32::from_rgb(255, 255, 255);
const BLACK: Color32 = Color32::from_rgb(0, 0, 0);
const CYAN: Color32 = Color32::from_rgb(0, 255, 255);
const RED: Color32 = Color32::from_rgb(255, 0, 0);

pub enum Msg {
    ApiKeySet(String),
    Refresh,
}

#[derive(Serialize, Deserialize)]
pub struct HeadlinesConfig {
    pub dark_mode: bool,
    pub api_key: String,
}

impl Default for HeadlinesConfig {
    fn default() -> Self {
        Self {
            dark_mode: Default::default(),
            api_key: String::new(),
        }
    }
}

pub struct NewsCardData {
    pub title: String,
    pub description: String,
    pub url: String,
}

pub struct Headlines {
    pub articles: Vec<NewsCardData>,
    pub config: HeadlinesConfig,
    pub api_key_initialized: bool,
    pub news_rx: Option<Receiver<NewsCardData>>,
    pub app_tx: Option<SyncSender<Msg>>,
}

impl Headlines {
    pub fn new() -> Headlines {
        Headlines {
            articles: vec![],
            api_key_initialized: Default::default(),
            config: Default::default(),
            news_rx: None,
            app_tx: None,
        }
    }

    pub fn configure_fonts(&self, ctx: &CtxRef) {
        let mut font_def = FontDefinitions::default();
        font_def.font_data.insert(
            "MesloLGS".to_string(),
            std::borrow::Cow::Borrowed(include_bytes!("../../MesloLGS_NF_Regular.ttf")),
        );
        font_def
            .family_and_size
            .insert(TextStyle::Heading, (FontFamily::Proportional, 30.0));
        font_def
            .family_and_size
            .insert(TextStyle::Body, (FontFamily::Proportional, 20.0));
        font_def
            .fonts_for_family
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "MesloLGS".to_string());
        ctx.set_fonts(font_def);
    }
    pub fn render_news_cards(&self, ui: &mut Ui) {
        for a in &self.articles {
            ui.add_space(PADDING);

            //render title
            let title = format!("▶ {}", a.title);
            if self.config.dark_mode {
                ui.colored_label(WHITE, title);
            } else {
                ui.colored_label(BLACK, title);
            }

            //render desc
            ui.add_space(PADDING);
            let desc = Label::new(&a.description).text_style(TextStyle::Button);
            ui.add(desc);

            //render hyperlink
            if self.config.dark_mode {
                ui.style_mut().visuals.hyperlink_color = CYAN;
            } else {
                ui.style_mut().visuals.hyperlink_color = RED;
            }

            ui.add_space(PADDING);
            ui.with_layout(Layout::right_to_left(), |ui| {
                ui.add(Hyperlink::new(&a.url).text("read more ⤴"));
            });
            ui.add_space(PADDING);
            ui.add(Separator::default());
        }
    }

    pub(crate) fn render_top_panel(&mut self, ctx: &CtxRef, frame: &mut eframe::epi::Frame<'_>) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(10.0);
            menu::bar(ui, |ui| {
                ui.with_layout(Layout::left_to_right(), |ui| {
                    ui.add(Label::new("📓").text_style(TextStyle::Heading));
                });
                ui.with_layout(Layout::right_to_left(), |ui| {
                    //conditional render
                    if !cfg!(target_arch = "wasm32") {
                        let close_btn = ui.add(Button::new("❌").text_style(TextStyle::Body));
                        if close_btn.clicked() {
                            frame.quit();
                        }
                    }

                    let refresh_btn = ui.add(Button::new("🔄").text_style(TextStyle::Body));
                    if refresh_btn.clicked() {
                        self.articles.clear();
                        if let Some(tx) = &self.app_tx {
                            tx.send(Msg::Refresh);
                        }
                    }

                    let theme_btn = ui.add(
                        Button::new({
                            if self.config.dark_mode {
                                "🌞"
                            } else {
                                "🌙"
                            }
                        })
                        .text_style(TextStyle::Body),
                    );
                    if theme_btn.clicked() {
                        self.config.dark_mode = !self.config.dark_mode;
                    }
                })
            });
            ui.add_space(10.0);
        });
    }

    pub fn preload_articles(&mut self) {
        if let Some(rx) = &self.news_rx {
            match rx.try_recv() {
                Ok(news_data) => {
                    self.articles.push(news_data);
                }
                Err(e) => {
                    tracing::warn!("Error receiving news data: {}", e);
                }
            }
        }
    }

    pub fn render_config(&mut self, ctx: &CtxRef) {
        CentralPanel::default().show(ctx, |ui| {
            Window::new("Configuration").show(ctx, |ui| {
                ui.label("Enter your API_KEY for newsapi.org");
                let text_input = ui.text_edit_singleline(&mut self.config.api_key);
                if text_input.lost_focus() && ui.input().key_pressed(Key::Enter) {
                    self.api_key_initialized = true;
                    if let Some(tx) = &self.app_tx {
                        tx.send(Msg::ApiKeySet(self.config.api_key.to_string()));
                    }
                    // tracing::error!("api key set");
                }
                // tracing::error!("{}", &self.config.api_key);
                ui.label("If you haven't registered for the API_KEY, head over to");
                ui.hyperlink("https://newsapi.org");
            });
        });
    }
}
