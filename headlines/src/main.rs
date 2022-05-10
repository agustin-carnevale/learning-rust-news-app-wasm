use eframe::{NativeOptions, egui::Vec2, run_native};
use headlines::Headlines;

fn main() {
    tracing_subscriber::fmt::init();

    let headlines = Headlines::new();
    let mut win_options = NativeOptions::default();
    win_options.initial_window_size = Some(Vec2::new(640.0, 860.0));
    run_native("Headlines", win_options, Box::new(|cc| Box::new(headlines.init(cc))));
}
