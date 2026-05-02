use eframe::egui;

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let board_path = if args.len() > 1 {
        Some(args[1].clone())
    } else {
        None
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([600.0, 400.0])
            .with_title("Recall Sketch Notes"),
        ..Default::default()
    };

    eframe::run_native(
        "Recall Sketch Notes",
        options,
        Box::new(|_cc| Ok(Box::new(recall_sketch_notes::app::RecallApp::new(board_path)))),
    )
}
