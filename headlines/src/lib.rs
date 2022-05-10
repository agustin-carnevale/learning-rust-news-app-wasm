mod headlines;

use eframe::{
    egui::{
        CentralPanel, Context, Hyperlink, Label, RichText, ScrollArea, Separator, TextStyle,
        TopBottomPanel, Ui, Visuals,
    },
    App,
};
pub use headlines::{Headlines, Msg, NewsCardData, PADDING};

impl App for Headlines {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        ctx.request_repaint();
        // ctx.set_debug_on_hover(true);

        if self.config.dark_mode {
            ctx.set_visuals(Visuals::dark());
        } else {
            ctx.set_visuals(Visuals::light());
        }

        if !self.api_key_initialized {
            self.render_config(ctx);
        } else {
            self.preload_articles();

            self.render_top_panel(ctx, frame);
            render_footer(ctx);

            CentralPanel::default().show(ctx, |ui| {
                if self.articles.is_empty() {
                    ui.vertical_centered_justified(|ui| {
                        ui.heading("Loading ⌛");
                    });
                } else {
                    render_header(ui);
                    ScrollArea::vertical().show(ui, |ui| {
                        self.render_news_cards(ui);
                    });
                }
            });
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "headlines", &self.config);
    }
}

fn render_header(ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        ui.heading("Headlines");
    });
    ui.add_space(PADDING);
    let sep = Separator::default().spacing(20.0);
    ui.add(sep);
}

fn render_footer(ctx: &Context) {
    TopBottomPanel::bottom("footer").show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(10.0);
            ui.add(Label::new(
                RichText::new("API source: newsapi.org").monospace(),
            ));
            ui.add(Hyperlink::from_label_and_url(
                RichText::new("Made with egui").text_style(TextStyle::Monospace),
                "https://github.com/emilk/egui",
            ));
            ui.add_space(10.0);
        })
    });
}

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn main_web(canvas_id: &str) {
    let headlines = Headlines::new();
    tracing_wasm::set_as_global_default();
    eframe::start_web(canvas_id, Box::new(|cc| Box::new(headlines.init(cc))))
        .expect("Failed to launch app");
}
