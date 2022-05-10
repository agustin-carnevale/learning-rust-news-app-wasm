use std::{
    sync::mpsc::{channel, sync_channel, Receiver, Sender, SyncSender},
};

#[cfg(not(target_arch = "wasm32"))]
use std::thread;

use eframe::{
    egui::{
        menu, Button, CentralPanel, Color32, Context, FontData, FontDefinitions, FontFamily,
        Hyperlink, Key, Label, Layout, RichText, Separator, TextStyle, TopBottomPanel, Ui, Window,
    },
    CreationContext, emath::Align,
};
use newsapi::NewsAPI;
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
    pub fn init(mut self, cc: &CreationContext) -> Self {
        if let Some(storage) = cc.storage {
            self.config = eframe::get_value(storage, "headlines").unwrap_or_default();
            self.api_key_initialized = !self.config.api_key.is_empty();
        }

        let api_key = self.config.api_key.to_string();

        let (news_tx, news_rx) = channel();
        let (app_tx, app_rx) = sync_channel(1);

        self.app_tx = Some(app_tx);
        self.news_rx = Some(news_rx);


        #[cfg(target_arch = "wasm32")]
        let api_key_web = api_key.clone();

        #[cfg(target_arch = "wasm32")]
        let news_tx_web = news_tx.clone();

        #[cfg(not(target_arch = "wasm32"))]
        thread::spawn(move || {
            if !api_key.is_empty() {
                fetch_news(&api_key, &news_tx);
            } else {
                loop {
                    match app_rx.recv() {
                        Ok(Msg::ApiKeySet(api_key)) => {
                            fetch_news(&api_key, &news_tx);
                        }
                        Ok(Msg::Refresh) => {
                            fetch_news(&api_key, &news_tx);
                        }
                        Err(e) => {
                            tracing::error!("failed receiving msg: {}", e);
                        }
                    }
                }
            }
        });

        #[cfg(target_arch = "wasm32")]
        gloo_timers::callback::Timeout::new(10, move || {
            wasm_bindgen_futures::spawn_local(async {
                fetch_web(api_key_web, news_tx_web).await;
            });
        })
        .forget();

        #[cfg(target_arch = "wasm32")]
        gloo_timers::callback::Interval::new(500, move || match app_rx.try_recv() {
            Ok(Msg::ApiKeySet(api_key)) => {
                wasm_bindgen_futures::spawn_local(fetch_web(api_key.clone(), news_tx.clone()));
            }
            Ok(Msg::Refresh) => {
                let api_key = api_key.clone();
                wasm_bindgen_futures::spawn_local(fetch_web(api_key, news_tx.clone()));
            }
            Err(e) => {
                tracing::error!("failed receiving msg: {}", e);
            }
        })
        .forget();

        self.configure_fonts(&cc.egui_ctx);

        self
    }

    pub fn configure_fonts(&self, ctx: &Context) {
        let mut font_def = FontDefinitions::default();
        font_def.font_data.insert(
            "MesloLGS".to_string(),
            FontData::from_static(include_bytes!("../../MesloLGS_NF_Regular.ttf")),
        );
        font_def
            .families
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
            let desc = Label::new(RichText::new(&a.description).text_style(TextStyle::Button));
            ui.add(desc);

            //render hyperlink
            if self.config.dark_mode {
                ui.style_mut().visuals.hyperlink_color = CYAN;
            } else {
                ui.style_mut().visuals.hyperlink_color = RED;
            }

            ui.add_space(PADDING);
            ui.with_layout(Layout::right_to_left().with_cross_align(Align::Min), |ui| {
                ui.add(Hyperlink::from_label_and_url("read more ⤴", &a.url));
            });
            ui.add_space(PADDING);
            ui.add(Separator::default());
        }
    }

    pub(crate) fn render_top_panel(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(10.0);
            menu::bar(ui, |ui| {
                ui.with_layout(Layout::left_to_right(), |ui| {
                    ui.add(Label::new(
                        RichText::new("📓").text_style(TextStyle::Heading),
                    ));
                });
                ui.with_layout(Layout::right_to_left(), |ui| {
                    //conditional render
                    if !cfg!(target_arch = "wasm32") {
                        let close_btn =
                            ui.add(Button::new(RichText::new("❌").text_style(TextStyle::Body)));
                        if close_btn.clicked() {
                            frame.quit();
                        }
                    }

                    let refresh_btn =
                        ui.add(Button::new(RichText::new("🔄").text_style(TextStyle::Body)));
                    if refresh_btn.clicked() {
                        self.articles.clear();
                        if let Some(tx) = &self.app_tx {
                            tx.send(Msg::Refresh).expect("Failed sending refresh event");
                        }
                    }

                    let theme_btn = ui.add(Button::new(
                        RichText::new({
                            if self.config.dark_mode {
                                "🌞"
                            } else {
                                "🌙"
                            }
                        })
                        .text_style(TextStyle::Body),
                    ));
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
                Err(_e) => {
                  // tracing::warn!("Error receiving news data: {}", e);
                }
            }
        }
    }

    pub fn render_config(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |_ui| {
            Window::new("Configuration").show(ctx, |ui| {
                ui.label("Enter your API_KEY for newsapi.org");
                let text_input = ui.text_edit_singleline(&mut self.config.api_key);
                if text_input.lost_focus() && ui.input().key_pressed(Key::Enter) {
                    self.api_key_initialized = true;
                    if let Some(tx) = &self.app_tx {
                        tx.send(Msg::ApiKeySet(self.config.api_key.to_string())).expect("Failed sending ApiKeySet event");
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

#[cfg(not(target_arch = "wasm32"))]
fn fetch_news(api_key: &str, news_tx: &Sender<NewsCardData>) {
    if let Ok(response) = NewsAPI::new(&api_key).fetch() {
        let response_articles = response.articles();
        for a in response_articles.iter() {
            let news = NewsCardData {
                title: a.title().to_string(),
                url: a.url().to_string(),
                description: "default description".to_string(),
            };

            if let Err(e) = news_tx.send(news) {
                tracing::error!("Error sending news data: {}", e);
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
async fn fetch_web(api_key: String, news_tx: Sender<NewsCardData>) {
    if let Ok(response) = NewsAPI::new(&api_key).fetch_web().await {
        let resp_articles = response.articles();
        for a in resp_articles.iter() {
            let news = NewsCardData {
                title: a.title().to_string(),
                url: a.url().to_string(),
                description: "default description".to_string(),
            };
            if let Err(e) = news_tx.send(news) {
                tracing::error!("Error sending news data: {}", e);
            }
        }
    } else {
        tracing::error!("failed fetching news");
    }
}
