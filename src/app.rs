use eframe::egui;
use egui::{CornerRadius, Color32, Frame, Margin, Panel, Pos2, Stroke, Vec2};

use crate::model::{Board, DrawingStroke, TextNote};
use crate::storage;

#[derive(PartialEq)]
enum ToolMode {
    Pen,
    Text,
}

pub struct RecallApp {
    board: Board,
    board_path: Option<String>,
    mode: ToolMode,
    current_stroke: Option<Vec<[f32; 2]>>,
    next_note_id: u64,
    status: String,
    editing_note: Option<u64>,
    edit_buffer: String,
    drag_note_id: Option<u64>,
    drag_offset: Vec2,
    dirty: bool,
}

impl Default for RecallApp {
    fn default() -> Self {
        Self {
            board: Board::new("Untitled Board"),
            board_path: None,
            mode: ToolMode::Pen,
            current_stroke: None,
            next_note_id: 1,
            status: "Ready".to_string(),
            editing_note: None,
            edit_buffer: String::new(),
            drag_note_id: None,
            drag_offset: Vec2::ZERO,
            dirty: false,
        }
    }
}

impl RecallApp {
    pub fn new(board_path: Option<String>) -> Self {
        let mut app = Self::default();
        if let Some(path) = &board_path {
            match storage::load_board(path) {
                Ok(board) => {
                    app.board = board;
                    app.board_path = Some(path.clone());
                    app.next_note_id = app
                        .board
                        .text_notes
                        .iter()
                        .map(|n| n.id)
                        .max()
                        .unwrap_or(0)
                        + 1;
                    app.status = format!("Loaded: {path}");
                }
                Err(e) => {
                    app.status = format!("Load failed: {e}");
                }
            }
        }
        app
    }

    fn save_current(&mut self) {
        let path = self.board_path.clone().unwrap_or_else(|| {
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            format!("./data/board_{ts}.json")
        });
        self.board_path = Some(path.clone());
        match storage::save_board(&self.board, &path) {
            Ok(()) => {
                self.status = format!("Saved: {path}");
                self.dirty = false;
            }
            Err(e) => {
                self.status = format!("Save failed: {e}");
            }
        }
    }

    fn load(&mut self, path: &str) {
        match storage::load_board(path) {
            Ok(board) => {
                self.board = board;
                self.board_path = Some(path.to_string());
                self.next_note_id = self
                    .board
                    .text_notes
                    .iter()
                    .map(|n| n.id)
                    .max()
                    .unwrap_or(0)
                    + 1;
                self.dirty = false;
                self.status = format!("Loaded: {path}");
            }
            Err(e) => {
                self.status = format!("Load failed: {e}");
            }
        }
    }

    fn undo_last_stroke(&mut self) {
        self.board.strokes.pop();
        self.dirty = true;
        self.status = "Undo last stroke".to_string();
    }

    fn clear_all(&mut self) {
        self.board.strokes.clear();
        self.board.text_notes.clear();
        self.dirty = true;
        self.status = "Canvas cleared".to_string();
    }
}

impl eframe::App for RecallApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Keyboard shortcuts
        ctx.input_mut(|i| {
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::S) {
                self.save_current();
            }
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::Z) {
                self.undo_last_stroke();
            }
        });

        // ── Toolbar ──
        Panel::top("toolbar")
            .frame(Frame {
                fill: Color32::from_gray(24),
                inner_margin: Margin::symmetric(8, 4),
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    let pen_sel = self.mode == ToolMode::Pen;
                    let text_sel = self.mode == ToolMode::Text;
                    if ui.selectable_label(pen_sel, "\u{270F}\u{FE0F} Pen").clicked() {
                        self.mode = ToolMode::Pen;
                        self.editing_note = None;
                        self.status = "Pen mode".to_string();
                    }
                    if ui.selectable_label(text_sel, "\u{1F524} Text").clicked() {
                        self.mode = ToolMode::Text;
                        self.current_stroke = None;
                        self.status = "Text mode".to_string();
                    }

                    ui.separator();

                    if ui.button("\u{21A9} Undo").clicked() {
                        self.undo_last_stroke();
                    }
                    if ui.button("\u{1F5D1} Clear").clicked() {
                        self.clear_all();
                    }

                    ui.separator();

                    if ui.button("\u{1F4BE} Save").clicked() {
                        self.save_current();
                    }
                    if ui.button("\u{1F4C2} Load").clicked() {
                        let path = self
                            .board_path
                            .clone()
                            .unwrap_or_else(|| "./data/board.json".to_string());
                        self.load(&path);
                    }
                    if ui.button("Save As...").clicked() {
                        let ts = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                        let path = format!("./data/board_{ts}.json");
                        self.board_path = Some(path.clone());
                        self.save_current();
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(&self.status);
                    });
                });
            });

        // ── Status bar ──
        Panel::bottom("status")
            .frame(Frame {
                fill: Color32::from_gray(24),
                inner_margin: Margin::symmetric(8, 2),
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    let mode_label = match self.mode {
                        ToolMode::Pen => "\u{270F}\u{FE0F} Pen",
                        ToolMode::Text => "\u{1F524} Text",
                    };
                    ui.label(mode_label);
                    ui.separator();
                    ui.label(format!(
                        "Strokes: {}  Notes: {}",
                        self.board.strokes.len(),
                        self.board.text_notes.len()
                    ));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if self.dirty {
                            ui.label(egui::RichText::new("\u{25CF} unsaved").color(Color32::YELLOW));
                        }
                        ui.label(&self.status);
                    });
                });
            });

        // ── Canvas ──
        egui::CentralPanel::default()
            .frame(Frame {
                fill: Color32::from_gray(18),
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                let (response, painter) = ui.allocate_painter(
                    ui.available_size(),
                    egui::Sense::click_and_drag(),
                );

                let canvas_rect = response.rect;
                let to_canvas = |pos: Pos2| -> [f32; 2] {
                    [pos.x - canvas_rect.min.x, pos.y - canvas_rect.min.y]
                };
                let from_canvas =
                    |p: &[f32; 2]| -> Pos2 { Pos2::new(canvas_rect.min.x + p[0], canvas_rect.min.y + p[1]) };

                // ── Handle input ──
                match self.mode {
                    ToolMode::Pen => {
                        if response.dragged() {
                            if let Some(pos) = response.interact_pointer_pos() {
                                let cp = to_canvas(pos);
                                if self.current_stroke.is_none() {
                                    let mut stroke = DrawingStroke::new([180, 200, 255], 3.0);
                                    stroke.points.push(cp);
                                    self.current_stroke = Some(stroke.points);
                                } else {
                                    self.current_stroke.as_mut().unwrap().push(cp);
                                }
                            }
                        }
                        if response.drag_stopped() {
                            if let Some(points) = self.current_stroke.take() {
                                if points.len() >= 2 {
                                    self.board.strokes.push(DrawingStroke {
                                        points,
                                        color: [180, 200, 255],
                                        width: 3.0,
                                    });
                                    self.dirty = true;
                                }
                            }
                        }
                    }
                    ToolMode::Text => {
                        if response.clicked() {
                            if let Some(pos) = response.interact_pointer_pos() {
                                let cp = to_canvas(pos);
                                let clicked_id = self
                                    .board
                                    .text_notes
                                    .iter()
                                    .find(|n| {
                                        let np = Pos2::new(
                                            canvas_rect.min.x + n.x,
                                            canvas_rect.min.y + n.y,
                                        );
                                        pos.distance(np) < 30.0
                                    })
                                    .map(|n| n.id);

                                if let Some(nid) = clicked_id {
                                    let text = self
                                        .board
                                        .text_notes
                                        .iter()
                                        .find(|n| n.id == nid)
                                        .map(|n| n.text.clone())
                                        .unwrap_or_default();
                                    self.editing_note = Some(nid);
                                    self.edit_buffer = text;
                                    self.status = "Editing note".to_string();
                                } else {
                                    let id = self.next_note_id;
                                    self.next_note_id += 1;
                                    self.board.text_notes.push(TextNote {
                                        id,
                                        x: cp[0] - 50.0,
                                        y: cp[1] - 10.0,
                                        text: "New note".to_string(),
                                    });
                                    self.editing_note = Some(id);
                                    self.edit_buffer = String::new();
                                    self.dirty = true;
                                    self.status = "New note (type to edit)".to_string();
                                }
                            }
                        }

                        if response.dragged() && self.drag_note_id.is_none() {
                            if let Some(pos) = response.interact_pointer_pos() {
                                for note in &self.board.text_notes {
                                    let np = Pos2::new(
                                        canvas_rect.min.x + note.x,
                                        canvas_rect.min.y + note.y,
                                    );
                                    if pos.distance(np) < 30.0 {
                                        self.drag_note_id = Some(note.id);
                                        self.drag_offset = pos - np;
                                        break;
                                    }
                                }
                            }
                        }
                        if let Some(drag_id) = self.drag_note_id {
                            if response.dragged() {
                                if let Some(pos) = response.interact_pointer_pos() {
                                    if let Some(note) = self
                                        .board
                                        .text_notes
                                        .iter_mut()
                                        .find(|n| n.id == drag_id)
                                    {
                                        note.x = pos.x - canvas_rect.min.x - self.drag_offset.x;
                                        note.y = pos.y - canvas_rect.min.y - self.drag_offset.y;
                                        self.dirty = true;
                                    }
                                }
                            }
                            if response.drag_stopped() {
                                self.drag_note_id = None;
                            }
                        }
                    }
                }

                // ── Draw strokes ──
                for stroke in &self.board.strokes {
                    if stroke.points.len() >= 2 {
                        let pts: Vec<Pos2> = stroke.points.iter().map(from_canvas).collect();
                        painter.add(egui::Shape::line(
                            pts,
                            Stroke::new(
                                stroke.width,
                                Color32::from_rgb(stroke.color[0], stroke.color[1], stroke.color[2]),
                            ),
                        ));
                    }
                }

                // ── Draw current in-progress stroke ──
                if let Some(points) = &self.current_stroke {
                    if points.len() >= 2 {
                        let pts: Vec<Pos2> = points.iter().map(from_canvas).collect();
                        painter.add(egui::Shape::line(
                            pts,
                            Stroke::new(3.0, Color32::from_rgb(180, 200, 255)),
                        ));
                    }
                }

                // ── Draw text notes ──
                let editing_note = self.editing_note;
                let mut edit_buffer = std::mem::take(&mut self.edit_buffer);
                let mut edit_result: Option<(u64, String)> = None;

                for note in &self.board.text_notes {
                    let pos = from_canvas(&[note.x, note.y]);
                    let is_editing = editing_note == Some(note.id);

                    let note_rect = egui::Rect::from_min_size(pos, egui::vec2(200.0, 32.0));
                    painter.rect_filled(
                        note_rect,
                        CornerRadius::same(4),
                        Color32::from_black_alpha(160),
                    );

                    if is_editing {
                        let area_id = egui::Id::new("note_edit");
                        let mut edit_done = false;
                        let mut edit_cancelled = false;

                        egui::Area::new(area_id)
                            .fixed_pos(pos + Vec2::new(4.0, 4.0))
                            .show(&ctx, |ui| {
                                ui.set_max_width(190.0);
                                let resp = ui.add(
                                    egui::TextEdit::singleline(&mut edit_buffer)
                                        .desired_width(190.0)
                                        .text_color(Color32::WHITE),
                                );
                                if resp.lost_focus()
                                    || ui.input(|i| i.key_pressed(egui::Key::Enter))
                                {
                                    edit_done = true;
                                }
                                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                                    edit_cancelled = true;
                                }
                            });

                        if edit_cancelled {
                            self.editing_note = None;
                        } else if edit_done {
                            edit_result = Some((note.id, edit_buffer.clone()));
                            self.editing_note = None;
                            self.dirty = true;
                        }
                    } else {
                        painter.text(
                            pos + Vec2::new(6.0, 6.0),
                            egui::Align2::LEFT_TOP,
                            &note.text,
                            egui::TextStyle::Body.resolve(&ctx.global_style()),
                            Color32::WHITE,
                        );
                    }
                }

                self.edit_buffer = edit_buffer;
                if let Some((nid, text)) = edit_result {
                    if let Some(note) = self.board.text_notes.iter_mut().find(|n| n.id == nid) {
                        note.text = text;
                    }
                }
            });
    }
}
