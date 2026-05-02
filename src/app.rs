fn draw_arrowhead(painter: &egui::Painter, from: Pos2, to: Pos2, stroke: Stroke) {
    let dir = to - from;
    let len = dir.length();
    if len < 1.0 { return; }
    let dir = dir / len;
    let perp = Vec2::new(-dir.y, dir.x);
    let arrow_len = 12.0;
    let arrow_width = 6.0;
    let left = to - dir * arrow_len + perp * arrow_width;
    let right = to - dir * arrow_len - perp * arrow_width;
    painter.add(egui::Shape::line_segment([to, left], stroke));
    painter.add(egui::Shape::line_segment([to, right], stroke));
}

fn dist_point_segment(px: f32, py: f32, ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let dx = bx - ax;
    let dy = by - ay;
    let len_sq = dx * dx + dy * dy;
    if len_sq < 1e-12 {
        let ex = px - ax;
        let ey = py - ay;
        return (ex * ex + ey * ey).sqrt();
    }
    let t = ((px - ax) * dx + (py - ay) * dy) / len_sq;
    let t = t.clamp(0.0, 1.0);
    let cx = ax + t * dx;
    let cy = ay + t * dy;
    let ex = px - cx;
    let ey = py - cy;
    (ex * ex + ey * ey).sqrt()
}

fn point_in_ellipse(px: f32, py: f32, ex: f32, ey: f32, ew: f32, eh: f32) -> bool {
    if ew < 0.01 || eh < 0.01 { return false; }
    let rx = ew / 2.0;
    let ry = eh / 2.0;
    let cx = ex + rx;
    let cy = ey + ry;
    let nx = (px - cx) / rx;
    let ny = (py - cy) / ry;
    (nx * nx + ny * ny) <= 1.0
}

use eframe::egui;
use egui::{Color32, CornerRadius, Frame, Margin, Panel, Pos2, Stroke, StrokeKind, Vec2};
use egui::epaint::EllipseShape;

use crate::model::{Board, CanvasObject, DrawingStroke, EraserMode, Shape, ShapeType, TextNote};
use crate::storage;

#[derive(Debug, PartialEq, Clone)]
enum ToolMode {
    Cursor,
    Pen,
    Text,
    Eraser,
    Line,
    Arrow,
    Rect,
    Oval,
}

pub struct RecallApp {
    board: Board,
    board_path: Option<String>,
    mode: ToolMode,
    current_stroke: Option<Vec<[f32; 2]>>,
    current_shape: Option<([f32; 2], [f32; 2], ShapeType)>,
    next_note_id: u64,
    status: String,
    editing_note: Option<u64>,
    edit_buffer: String,
    drag_note_id: Option<u64>,
    drag_offset: Vec2,
    dirty: bool,
    show_load_input: bool,
    load_path_input: String,
    board_list: Vec<String>,
    selected_object_id: Option<u64>,
    zoom: f32,
    pan: Vec2,
    eraser_mode: EraserMode,
    eraser_size_el: f32,
    eraser_size_br: f32,
    eraser_last_pos: Option<[f32; 2]>,
    eraser_is_dragging: bool,
}

impl Default for RecallApp {
    fn default() -> Self {
        Self {
            board: Board::new("Untitled Board"),
            board_path: None,
            mode: ToolMode::Cursor,
            current_stroke: None,
            current_shape: None,
            next_note_id: 1,
            status: "Ready".to_string(),
            editing_note: None,
            edit_buffer: String::new(),
            drag_note_id: None,
            drag_offset: Vec2::ZERO,
            dirty: false,
            show_load_input: false,
            load_path_input: String::new(),
            board_list: Vec::new(),
            selected_object_id: None,
            zoom: 1.0,
            pan: Vec2::ZERO,
            eraser_mode: EraserMode::Element,
            eraser_size_el: 12.0,
            eraser_size_br: 20.0,
            eraser_last_pos: None,
            eraser_is_dragging: false,
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
                        .canvas_objects
                        .iter()
                        .filter_map(|obj| match obj {
                            CanvasObject::TextNote(n) => Some(n.id),
                            _ => None,
                        })
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
        self.scan_board_files();
    }

    fn load(&mut self, path: &str) {
        match storage::load_board(path) {
            Ok(board) => {
                self.board = board;
                self.board_path = Some(path.to_string());
                self.next_note_id = self
                    .board
                    .canvas_objects
                    .iter()
                    .filter_map(|obj| match obj {
                        CanvasObject::TextNote(n) => Some(n.id),
                        _ => None,
                    })
                    .max()
                    .unwrap_or(0)
                    + 1;
                self.selected_object_id = None;
                self.zoom = 1.0;
                self.pan = Vec2::ZERO;
                self.dirty = false;
                self.status = format!("Loaded: {path}");
            }
            Err(e) => {
                self.status = format!("Load failed: {e}");
            }
        }
        self.scan_board_files();
    }

    fn undo_last_stroke(&mut self) {
        // Remove the last Stroke variant from canvas_objects
        let mut remove_idx = None;
        for (i, obj) in self.board.canvas_objects.iter().enumerate().rev() {
            if matches!(obj, CanvasObject::Stroke(_)) {
                remove_idx = Some(i);
                break;
            }
        }
        if let Some(idx) = remove_idx {
            self.board.canvas_objects.remove(idx);
            self.dirty = true;
            self.status = "Undo last stroke".to_string();
        }
    }

    fn scan_board_files(&mut self) {
        let data_dir = "./data";
        self.board_list.clear();
        if let Ok(entries) = std::fs::read_dir(data_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    if let Some(s) = path.to_str() {
                        self.board_list.push(s.to_string());
                    }
                }
            }
        }
        self.board_list.sort();
    }

    fn new_board(&mut self) {
        if self.dirty {
            self.save_current();
        }
        self.board = Board::new("Untitled Board");
        self.board_path = None;
        self.next_note_id = 1;
        self.current_stroke = None;
        self.editing_note = None;
        self.selected_object_id = None;
        self.zoom = 1.0;
        self.pan = Vec2::ZERO;
        self.dirty = false;
        self.status = "New board".to_string();
        self.scan_board_files();
    }

    fn switch_board(&mut self, path: &str) {
        if self.dirty {
            self.save_current();
        }
        self.load(path);
    }

    fn clear_all(&mut self) {
        self.board.canvas_objects.clear();
        self.selected_object_id = None;
        self.dirty = true;
        self.status = "Canvas cleared".to_string();
    }

    fn count_objects(&self) -> usize {
        self.board.canvas_objects.len()
    }

    fn count_strokes(&self) -> usize {
        self.board
            .canvas_objects
            .iter()
            .filter(|obj| matches!(obj, CanvasObject::Stroke(_)))
            .count()
    }

    fn count_notes(&self) -> usize {
        self.board
            .canvas_objects
            .iter()
            .filter(|obj| matches!(obj, CanvasObject::TextNote(_)))
            .count()
    }

    fn find_note(&self, id: u64) -> Option<TextNote> {
        for obj in &self.board.canvas_objects {
            if let CanvasObject::TextNote(n) = obj {
                if n.id == id {
                    return Some(n.clone());
                }
            }
        }
        None
    }

    fn find_note_mut(&mut self, id: u64) -> Option<&mut TextNote> {
        for obj in &mut self.board.canvas_objects {
            if let CanvasObject::TextNote(n) = obj {
                if n.id == id {
                    return Some(n);
                }
            }
        }
        None
    }

    fn find_note_at(&self, canvas_rect: egui::Rect, pos: Pos2, threshold: f32) -> Option<u64> {
        for obj in &self.board.canvas_objects {
            if let CanvasObject::TextNote(n) = obj {
                let np = Pos2::new(canvas_rect.min.x + n.x, canvas_rect.min.y + n.y);
                if pos.distance(np) < threshold {
                    return Some(n.id);
                }
            }
        }
        None
    }

    fn find_draggable_note(&self, canvas_rect: egui::Rect, pos: Pos2, threshold: f32) -> Option<u64> {
        self.find_note_at(canvas_rect, pos, threshold)
    }

    fn find_object_at(&self, canvas_rect: egui::Rect, pos: Pos2, threshold: f32) -> Option<u64> {
        for obj in &self.board.canvas_objects {
            match obj {
                CanvasObject::TextNote(n) => {
                    let np = Pos2::new(canvas_rect.min.x + n.x, canvas_rect.min.y + n.y);
                    if pos.distance(np) < threshold {
                        return Some(n.id);
                    }
                }
                CanvasObject::Stroke(s) => {
                    for p in &s.points {
                        let pp = Pos2::new(canvas_rect.min.x + p[0], canvas_rect.min.y + p[1]);
                        if pos.distance(pp) < threshold {
                            return Some(s.id);
                        }
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn hit_test_object(&self, obj: &CanvasObject, wx: f32, wy: f32, radius: f32) -> bool {
        match obj {
            CanvasObject::TextNote(n) => {
                let dx = n.x - wx;
                let dy = n.y - wy;
                (dx * dx + dy * dy) < radius * radius
            }
            CanvasObject::Stroke(s) => {
                s.points.iter().any(|p| {
                    let dx = p[0] - wx;
                    let dy = p[1] - wy;
                    (dx * dx + dy * dy) < radius * radius
                })
            }
            CanvasObject::Shape(sh) => {
                let r = radius;
                match sh.shape_type {
                    ShapeType::Line | ShapeType::Arrow => {
                        dist_point_segment(wx, wy, sh.x1, sh.y1, sh.x2, sh.y2) < r
                    }
                    ShapeType::Rect => {
                        let xmin = sh.x1.min(sh.x2);
                        let xmax = sh.x1.max(sh.x2);
                        let ymin = sh.y1.min(sh.y2);
                        let ymax = sh.y1.max(sh.y2);
                        if wx >= xmin - r && wx <= xmax + r && wy >= ymin - r && wy <= ymax + r {
                            let near_edge = (wx - xmin).abs() < r || (wx - xmax).abs() < r
                                || (wy - ymin).abs() < r || (wy - ymax).abs() < r;
                            near_edge || (wx >= xmin && wx <= xmax && wy >= ymin && wy <= ymax)
                        } else {
                            false
                        }
                    }
                    ShapeType::Oval => {
                        let xmin = sh.x1.min(sh.x2);
                        let xmax = sh.x1.max(sh.x2);
                        let ymin = sh.y1.min(sh.y2);
                        let ymax = sh.y1.max(sh.y2);
                        let ew = (xmax - xmin).max(0.01);
                        let eh = (ymax - ymin).max(0.01);
                        let inflate = r;
                        point_in_ellipse(wx, wy, xmin - inflate, ymin - inflate, ew + 2.0 * inflate, eh + 2.0 * inflate)
                    }
                }
            }
        }
    }

    fn hit_test_index(&self, wx: f32, wy: f32, radius: f32) -> Option<usize> {
        for (i, obj) in self.board.canvas_objects.iter().enumerate().rev() {
            if self.hit_test_object(obj, wx, wy, radius) {
                return Some(i);
            }
        }
        None
    }

    fn erase_elements_at(&mut self, wx: f32, wy: f32, radius: f32, delete_one: bool) {
        let mut to_remove: Vec<usize> = Vec::new();
        if delete_one {
            if let Some(idx) = self.hit_test_index(wx, wy, radius) {
                to_remove.push(idx);
            }
        } else {
            for (i, obj) in self.board.canvas_objects.iter().enumerate() {
                if self.hit_test_object(obj, wx, wy, radius) {
                    to_remove.push(i);
                }
            }
        }
        if to_remove.is_empty() { return; }
        for i in to_remove.into_iter().rev() {
            self.board.canvas_objects.remove(i);
        }
        self.dirty = true;
    }

    fn erase_brush_at(&mut self, wx: f32, wy: f32, radius: f32) {
        let radius_sq = radius * radius;
        let objects = std::mem::take(&mut self.board.canvas_objects);
        let mut erased_anything = false;

        for obj in objects {
            match obj {
                CanvasObject::Stroke(s) => {
                    let keep: Vec<bool> = s.points.iter().map(|p| {
                        let dx = p[0] - wx;
                        let dy = p[1] - wy;
                        dx * dx + dy * dy >= radius_sq
                    }).collect();

                    if keep.iter().all(|&k| k) {
                        self.board.canvas_objects.push(CanvasObject::Stroke(s));
                    } else {
                        erased_anything = true;
                        let mut segment: Vec<[f32; 2]> = Vec::new();
                        for (i, pt) in s.points.iter().enumerate() {
                            if keep[i] {
                                segment.push(*pt);
                            } else {
                                if segment.len() >= 2 {
                                    self.board.canvas_objects.push(
                                        CanvasObject::Stroke(DrawingStroke {
                                            id: s.id,
                                            points: segment,
                                            color: s.color,
                                            width: s.width,
                                        }),
                                    );
                                }
                                segment = Vec::new();
                            }
                        }
                        if segment.len() >= 2 {
                            self.board.canvas_objects.push(
                                CanvasObject::Stroke(DrawingStroke {
                                    id: s.id,
                                    points: segment,
                                    color: s.color,
                                    width: s.width,
                                }),
                            );
                        }
                    }
                }
                other => {
                    // For shapes and text notes: delete if hit by brush
                    let hit = self.hit_test_object(&other, wx, wy, radius);
                    if hit {
                        erased_anything = true;
                        // Shape deleted entirely
                    } else {
                        self.board.canvas_objects.push(other);
                    }
                }
            }
        }

        if erased_anything {
            self.dirty = true;
        }
    }

    fn interpolate_erase(&mut self, wx: f32, wy: f32, radius: f32, brush_mode: bool) {
        if let Some(last) = self.eraser_last_pos {
            let dx = wx - last[0];
            let dy = wy - last[1];
            let dist = (dx * dx + dy * dy).sqrt();
            let step = (radius * 0.5).max(1.0);
            if dist > step {
                let steps = (dist / step).min(32.0) as u32;
                for s in 1..=steps {
                    let t = s as f32 / steps as f32;
                    let ix = last[0] + dx * t;
                    let iy = last[1] + dy * t;
                    if brush_mode {
                        self.erase_brush_at(ix, iy, radius);
                    } else {
                        self.erase_elements_at(ix, iy, radius, false);
                    }
                }
                return;
            }
        }
        if brush_mode {
            self.erase_brush_at(wx, wy, radius);
        } else {
            self.erase_elements_at(wx, wy, radius, false);
        }
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
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::O) {
                self.show_load_input = !self.show_load_input;
                if self.show_load_input {
                    self.load_path_input = self
                        .board_path
                        .clone()
                        .unwrap_or_else(|| "./data/board.json".to_string());
                    self.status = "Enter file path and press Load".to_string();
                }
            }
        });

        // ── Toolbar ──
        Panel::top("toolbar")
            .frame(Frame {
                fill: Color32::from_gray(20),
                inner_margin: Margin::symmetric(10, 5),
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Recall").color(Color32::from_rgb(88, 166, 255)).size(13.0).strong());
                    ui.separator();

                    let tools = [
                        (ToolMode::Cursor, "C", "Cursor (select)"),
                        (ToolMode::Pen, "P", "Pen (draw)"),
                        (ToolMode::Text, "T", "Text (type)"),
                        (ToolMode::Eraser, "Er", "Eraser"),
                    ];
                    for (tool, icon, label) in &tools {
                        let sel = self.mode == *tool;
                        let btn = egui::Button::new(egui::RichText::new(*icon).size(13.0).strong()).fill(if sel { Color32::from_gray(40) } else { Color32::TRANSPARENT }).min_size(Vec2::new(32.0, 24.0));
                        let resp = ui.add(btn).on_hover_text(*label);
                        if resp.clicked() {
                            self.mode = tool.clone();
                            self.editing_note = None;
                            self.current_stroke = None;
                            self.status = format!("{:?} mode", tool);
                        }
                        // Eraser right-click context menu
                        if *tool == ToolMode::Eraser {
                            if resp.secondary_clicked() {
                                self.eraser_mode = if self.eraser_mode == EraserMode::Element { EraserMode::Brush } else { EraserMode::Element };
                                self.status = format!("Eraser: {:?}", self.eraser_mode);
                            }
                            // Show current eraser mode indicator
                            let mode_label = match self.eraser_mode {
                                EraserMode::Element => "El",
                                EraserMode::Brush => "Br",
                            };
                            ui.label(egui::RichText::new(mode_label).color(Color32::from_gray(160)).size(9.0));
                        }
                        // Eraser size slider when eraser is active
                        if self.mode == ToolMode::Eraser && *tool == ToolMode::Eraser {
                            ui.separator();
                            let size_label = match self.eraser_mode {
                                EraserMode::Element => "El sz",
                                EraserMode::Brush => "Br sz",
                            };
                            ui.label(egui::RichText::new(size_label).color(Color32::from_gray(140)).size(9.0));
                            let (min_sz, max_sz, cur) = match self.eraser_mode {
                                EraserMode::Element => (1.0, 64.0, &mut self.eraser_size_el),
                                EraserMode::Brush => (2.0, 128.0, &mut self.eraser_size_br),
                            };
                            ui.add(egui::Slider::new(cur, min_sz..=max_sz).show_value(false));
                            ui.label(egui::RichText::new(format!("{:.0}", *cur)).color(Color32::from_gray(160)).size(9.0));
                        }
                    }

                    ui.separator();

                    let shapes = [
                        (ToolMode::Line, "Ln", "Line"),
                        (ToolMode::Arrow, "Ar", "Arrow"),
                        (ToolMode::Rect, "Rc", "Rectangle"),
                        (ToolMode::Oval, "Ov", "Oval"),
                    ];
                    for (tool, icon, label) in &shapes {
                        let sel = self.mode == *tool;
                        let btn = egui::Button::new(egui::RichText::new(*icon).size(11.0)).fill(if sel { Color32::from_gray(40) } else { Color32::TRANSPARENT }).min_size(Vec2::new(32.0, 24.0));
                        if ui.add(btn).on_hover_text(*label).clicked() {
                            self.mode = tool.clone();
                            self.status = format!("{:?}", tool);
                        }
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
                        self.show_load_input = !self.show_load_input;
                        if self.show_load_input {
                            self.load_path_input = self
                                .board_path
                                .clone()
                                .unwrap_or_else(|| "./data/board.json".to_string());
                            self.status = "Enter file path".to_string();
                        }
                    }
                    if self.show_load_input {
                        let resp = ui.add(
                            egui::TextEdit::singleline(&mut self.load_path_input)
                                .desired_width(240.0)
                                .hint_text("path/to/board.json"),
                        );
                        let load_clicked =
                            resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                        if load_clicked {
                            let path = self.load_path_input.clone();
                            self.load(&path);
                            self.show_load_input = false;
                        }
                        if ui.button("Go").clicked() {
                            let path = self.load_path_input.clone();
                            self.load(&path);
                            self.show_load_input = false;
                        }
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

        // ── Sidebar (board list) ──
        self.scan_board_files();
        Panel::left("sidebar")
            .frame(Frame {
                fill: Color32::from_gray(22),
                inner_margin: Margin::symmetric(4, 4),
                ..Default::default()
            })
            .resizable(false)
            .default_size(160.0)
            .show_inside(ui, |ui| {
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("Boards")
                            .color(Color32::from_gray(160))
                            .size(10.0),
                    );
                    ui.add_space(2.0);
                    if ui.button(egui::RichText::new("+ New").size(11.0)).clicked() {
                        self.new_board();
                    }
                    ui.separator();
                    let current_basename = self
                        .board_path
                        .as_ref()
                        .map(|p| {
                            std::path::Path::new(p)
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("untitled")
                                .to_string()
                        })
                        .unwrap_or_else(|| "untitled".to_string());
                    ui.label(
                        egui::RichText::new(&current_basename)
                            .color(Color32::from_rgb(88, 166, 255))
                            .size(12.0),
                    );
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .max_height(ui.available_height() - 40.0)
                        .show(ui, |ui| {
                            let mut clicked: Option<String> = None;
                            for path in &self.board_list {
                                let basename = std::path::Path::new(path)
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or(path);
                                let is_current =
                                    self.board_path.as_ref().map(|p| p == path).unwrap_or(false);
                                if ui.selectable_label(is_current, basename).clicked()
                                    && !is_current
                                {
                                    clicked = Some(path.clone());
                                }
                            }
                            if let Some(p) = clicked {
                                self.switch_board(&p);
                            }
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
                    let mode_label: String = match self.mode {
                        ToolMode::Cursor => "C Select".into(),
                        ToolMode::Pen => "P Pen".into(),
                        ToolMode::Text => "T Text".into(),
                        ToolMode::Eraser => match self.eraser_mode {
                            EraserMode::Element => format!("Er El sz{:.0}", self.eraser_size_el),
                            EraserMode::Brush => format!("Er Br sz{:.0}", self.eraser_size_br),
                        },
                        ToolMode::Line => "Ln Line".into(),
                        ToolMode::Arrow => "Ar Arrow".into(),
                        ToolMode::Rect => "Rc Rect".into(),
                        ToolMode::Oval => "Ov Oval".into(),
                    };
                    ui.label(&mode_label);
                    ui.separator();
                    let sel_str = match self.selected_object_id {
                        Some(id) => format!(" | Sel:{}", id),
                        None => String::new(),
                    };
                    ui.label(format!(
                        "{}S {}N {}{}O{}",
                        mode_label, self.count_strokes(), self.count_notes(), self.count_objects(), sel_str,
                    ));
                    ui.separator();
                    ui.label(format!("Zoom: {:.0}%", self.zoom * 100.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if self.dirty {
                            ui.label(
                                egui::RichText::new("\u{25CF} unsaved").color(Color32::YELLOW),
                            );
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
                let (response, painter) =
                    ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());

                let canvas_rect = response.rect;
                let zoom = self.zoom;
                let pan = self.pan;

                let screen_to_world = |screen: Pos2| -> [f32; 2] {
                    let rel = screen - canvas_rect.min - pan;
                    [rel.x / zoom, rel.y / zoom]
                };
                let world_to_screen = |world: &[f32; 2]| -> Pos2 {
                    Pos2::new(
                        canvas_rect.min.x + pan.x + world[0] * zoom,
                        canvas_rect.min.y + pan.y + world[1] * zoom,
                    )
                };

                // ── Cursor-centered zoom with mouse wheel ──
                let scroll = ctx.input(|i| i.smooth_scroll_delta);
                if scroll.y != 0.0 || scroll.x != 0.0 {
                    let zoom_factor = 1.0 + scroll.y * 0.001;
                    let old_zoom = self.zoom;
                    let new_zoom = (old_zoom * zoom_factor).clamp(0.1, 10.0);
                    // Get cursor position in screen coordinates
                    let cursor_screen = ctx.input(|i| i.pointer.hover_pos()).unwrap_or(canvas_rect.center());
                    // Convert cursor to world coords BEFORE zoom change
                    let rel = cursor_screen - canvas_rect.min - self.pan;
                    let world_before = [rel.x / old_zoom, rel.y / old_zoom];
                    // Apply zoom
                    self.zoom = new_zoom;
                    // Adjust pan so same world point stays under cursor
                    self.pan = cursor_screen - canvas_rect.min - Vec2::new(world_before[0] * new_zoom, world_before[1] * new_zoom);
                }

                // ── Pan with middle mouse ──
                if response.dragged_by(egui::PointerButton::Middle) {
                    let _ = response.interact_pointer_pos();
                    let delta = response.drag_delta();
                    self.pan.x += delta.x;
                    self.pan.y += delta.y;
                }

                // ── Handle input ──
                match self.mode {
                    ToolMode::Cursor => {
                        if response.clicked() {
                            if let Some(pos) = response.interact_pointer_pos() {
                                let id = self.find_object_at(canvas_rect, pos, 30.0);
                                self.selected_object_id = id;
                                self.status = match id {
                                    Some(_) => "Selected".to_string(),
                                    None => "Deselected".to_string(),
                                };
                            }
                        }
                        if response.dragged() && self.selected_object_id.is_some() {
                            if let Some(pos) = response.interact_pointer_pos() {
                                if let Some(note) = self.find_note_mut(self.selected_object_id.unwrap())
                                {
                                    note.x = pos.x - canvas_rect.min.x - 50.0;
                                    note.y = pos.y - canvas_rect.min.y - 10.0;
                                    self.dirty = true;
                                }
                            }
                        }
                    }
                    ToolMode::Eraser => {
                        let pointer_down = ctx.input(|i| i.pointer.button_down(egui::PointerButton::Primary));
                        let hover = ctx.input(|i| i.pointer.hover_pos());

                        match self.eraser_mode {
                            EraserMode::Element => {
                                let radius = self.eraser_size_el / zoom.max(0.1);
                                if response.clicked() {
                                    if let Some(pos) = response.interact_pointer_pos() {
                                        let wp = screen_to_world(pos);
                                        if self.hit_test_index(wp[0], wp[1], radius).is_some() {
                                            self.erase_elements_at(wp[0], wp[1], radius, true);
                                            self.status = "Deleted element".to_string();
                                        }
                                    }
                                    self.eraser_is_dragging = false;
                                    self.eraser_last_pos = None;
                                }
                                if pointer_down && hover.is_some() && canvas_rect.contains(hover.unwrap()) {
                                    if !self.eraser_is_dragging {
                                        self.eraser_is_dragging = true;
                                        if let Some(pos) = hover {
                                            let wp = screen_to_world(pos);
                                            self.eraser_last_pos = Some(wp);
                                        }
                                    }
                                    if self.eraser_is_dragging {
                                        if let Some(pos) = hover {
                                            let wp = screen_to_world(pos);
                                            self.interpolate_erase(wp[0], wp[1], radius, false);
                                            self.eraser_last_pos = Some(wp);
                                            self.status = "Erasing elements...".to_string();
                                        }
                                        ctx.request_repaint();
                                    }
                                }
                                if !pointer_down && self.eraser_is_dragging {
                                    self.eraser_is_dragging = false;
                                    self.eraser_last_pos = None;
                                }
                            }
                            EraserMode::Brush => {
                                let radius = self.eraser_size_br / zoom.max(0.1);
                                if pointer_down && hover.is_some() && canvas_rect.contains(hover.unwrap()) {
                                    if !self.eraser_is_dragging {
                                        self.eraser_is_dragging = true;
                                        if let Some(pos) = hover {
                                            let wp = screen_to_world(pos);
                                            self.eraser_last_pos = Some(wp);
                                        }
                                    }
                                    if self.eraser_is_dragging {
                                        if let Some(pos) = hover {
                                            let wp = screen_to_world(pos);
                                            self.interpolate_erase(wp[0], wp[1], radius, true);
                                            self.eraser_last_pos = Some(wp);
                                            self.status = "Brush erasing...".to_string();
                                        }
                                        ctx.request_repaint();
                                    }
                                }
                                if !pointer_down && self.eraser_is_dragging {
                                    self.eraser_is_dragging = false;
                                    self.eraser_last_pos = None;
                                }
                            }
                        }
                    }
                    ToolMode::Line | ToolMode::Arrow | ToolMode::Rect | ToolMode::Oval => {
                        let st = match self.mode {
                            ToolMode::Line => ShapeType::Line,
                            ToolMode::Arrow => ShapeType::Arrow,
                            ToolMode::Rect => ShapeType::Rect,
                            ToolMode::Oval => ShapeType::Oval,
                            _ => unreachable!(),
                        };
                        if response.dragged() {
                            if let Some(pos) = response.interact_pointer_pos() {
                                let cp = screen_to_world(pos);
                                if self.current_shape.is_none() {
                                    self.current_shape = Some((cp, cp, st.clone()));
                                } else {
                                    let start = self.current_shape.as_ref().unwrap().0;
                                    self.current_shape = Some((start, cp, st.clone()));
                                }
                            }
                        }
                        if response.drag_stopped() {
                            if let Some((start, end, st2)) = self.current_shape.take() {
                                let id = self.next_note_id;
                                self.next_note_id += 1;
                                self.board.canvas_objects.push(
                                    CanvasObject::Shape(Shape::new(id, st2, start[0], start[1], end[0], end[1])),
                                );
                                self.dirty = true;
                            }
                        }
                    }
                    ToolMode::Pen => {
                        if response.dragged() {
                            if let Some(pos) = response.interact_pointer_pos() {
                                let cp = screen_to_world(pos);
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
                                    let mut stroke =
                                        DrawingStroke::new([180, 200, 255], 3.0);
                                    stroke.points = points;
                                    self.board
                                        .canvas_objects
                                        .push(CanvasObject::Stroke(stroke));
                                    self.dirty = true;
                                }
                            }
                        }
                    }
                    ToolMode::Text => {
                        if response.clicked() {
                            if let Some(pos) = response.interact_pointer_pos() {
                                let cp = screen_to_world(pos);
                                let clicked_id =
                                    self.find_note_at(canvas_rect, pos, 30.0);

                                if let Some(nid) = clicked_id {
                                    let text = self
                                        .find_note(nid)
                                        .map(|n| n.text.clone())
                                        .unwrap_or_default();
                                    self.editing_note = Some(nid);
                                    self.edit_buffer = text;
                                    self.status = "Editing note".to_string();
                                } else {
                                    let id = self.next_note_id;
                                    self.next_note_id += 1;
                                    self.board.canvas_objects.push(
                                        CanvasObject::TextNote(TextNote {
                                            id,
                                            x: cp[0] - 50.0,
                                            y: cp[1] - 10.0,
                                            text: "New note".to_string(),
                                        }),
                                    );
                                    self.editing_note = Some(id);
                                    self.edit_buffer = String::new();
                                    self.dirty = true;
                                    self.status = "New note (type to edit)".to_string();
                                }
                            }
                        }

                        if response.dragged() && self.drag_note_id.is_none() {
                            if let Some(pos) = response.interact_pointer_pos() {
                                if let Some(note_id) =
                                    self.find_draggable_note(canvas_rect, pos, 30.0)
                                {
                                    self.drag_note_id = Some(note_id);
                                    let note = self.find_note(note_id).unwrap();
                                    let np = Pos2::new(
                                        canvas_rect.min.x + note.x,
                                        canvas_rect.min.y + note.y,
                                    );
                                    self.drag_offset = pos - np;
                                }
                            }
                        }
                        if let Some(drag_id) = self.drag_note_id {
                            if response.dragged() {
                                if let Some(pos) = response.interact_pointer_pos() {
                                    let dx = self.drag_offset.x;
                                    let dy = self.drag_offset.y;
                                    if let Some(note) = self.find_note_mut(drag_id) {
                                        note.x = pos.x - canvas_rect.min.x - dx;
                                        note.y = pos.y - canvas_rect.min.y - dy;
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

                // ── Draw strokes and shapes ──
                for obj in &self.board.canvas_objects {
                    match obj {
                        CanvasObject::Stroke(stroke) => {
                            if stroke.points.len() >= 2 {
                                let pts: Vec<Pos2> =
                                    stroke.points.iter().map(world_to_screen).collect();
                                painter.add(egui::Shape::line(
                                    pts,
                                    Stroke::new(
                                        stroke.width,
                                        Color32::from_rgb(
                                            stroke.color[0],
                                            stroke.color[1],
                                            stroke.color[2],
                                        ),
                                    ),
                                ));
                            }
                        }
                        CanvasObject::Shape(shape) => {
                            let p1 = world_to_screen(&[shape.x1, shape.y1]);
                            let p2 = world_to_screen(&[shape.x2, shape.y2]);
                            let color = Color32::from_rgb(shape.color[0], shape.color[1], shape.color[2]);
                            let sw = Stroke::new(shape.stroke_width, color);
                            match shape.shape_type {
                                ShapeType::Line => {
                                    painter.add(egui::Shape::line_segment([p1, p2], sw));
                                }
                                ShapeType::Arrow => {
                                    painter.add(egui::Shape::line_segment([p1, p2], sw));
                                    draw_arrowhead(&painter, p1, p2, sw);
                                }
                                ShapeType::Rect => {
                                    let rect = egui::Rect::from_two_pos(p1, p2);
                                    painter.rect_stroke(rect, CornerRadius::same(0), sw, egui::StrokeKind::Outside);
                                }
                                ShapeType::Oval => {
                                    let rect = egui::Rect::from_two_pos(p1, p2);
                                    let center = rect.center();
                                    let radius = rect.size() * 0.5;
                                    let es = EllipseShape { center, radius, fill: Color32::TRANSPARENT, stroke: sw, angle: 0.0 };
                                    painter.add(egui::Shape::Ellipse(es));
                                }
                            }
                        }
                        _ => {}
                    }
                }

                // ── Draw current in-progress stroke ──
                if let Some(points) = &self.current_stroke {
                    if points.len() >= 2 {
                        let pts: Vec<Pos2> = points.iter().map(world_to_screen).collect();
                        painter.add(egui::Shape::line(
                            pts,
                            Stroke::new(3.0, Color32::from_rgb(180, 200, 255)),
                        ));
                    }
                }

                // ── Draw in-progress shape ──
                if let Some((start, end, st)) = &self.current_shape {
                    let p1 = world_to_screen(start);
                    let p2 = world_to_screen(end);
                    let col = Color32::from_rgb(180, 200, 255);
                    let sw = Stroke::new(2.0, col);
                    match st {
                        ShapeType::Line => {
                            painter.add(egui::Shape::line_segment([p1, p2], sw));
                        }
                        ShapeType::Arrow => {
                            painter.add(egui::Shape::line_segment([p1, p2], sw));
                            draw_arrowhead(&painter, p1, p2, sw);
                        }
                        ShapeType::Rect => {
                            let rect = egui::Rect::from_two_pos(p1, p2);
                            painter.rect_stroke(rect, CornerRadius::same(0), sw, StrokeKind::Outside);
                        }
                        ShapeType::Oval => {
                            let rect = egui::Rect::from_two_pos(p1, p2);
                            let center = rect.center();
                            let radius = rect.size() * 0.5;
                            let es = EllipseShape { center, radius, fill: Color32::TRANSPARENT, stroke: sw, angle: 0.0 };
                            painter.add(egui::Shape::Ellipse(es));
                        }
                    }
                }

                // ── Draw text notes ──
                let editing_note = self.editing_note;
                let mut edit_buffer = std::mem::take(&mut self.edit_buffer);
                let mut edit_result: Option<(u64, String)> = None;

                for obj in &self.board.canvas_objects {
                    if let CanvasObject::TextNote(note) = obj {
                        let pos = world_to_screen(&[note.x, note.y]);
                        let is_editing = editing_note == Some(note.id);

                        let note_rect =
                            egui::Rect::from_min_size(pos, egui::vec2(200.0, 32.0));
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
                }

                self.edit_buffer = edit_buffer;
                if let Some((nid, text)) = edit_result {
                    if let Some(note) = self.find_note_mut(nid) {
                        note.text = text;
                    }
                }
            });
    }
}
