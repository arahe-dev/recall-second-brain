use std::fs;

use recall_sketch_notes::canvas_ops;
use recall_sketch_notes::flowchart::{self, FlowchartSpec};
use recall_sketch_notes::model::Board;
use recall_sketch_notes::storage;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!("  recall_ops flowchart <spec.json> --out <board.json>");
        eprintln!("  recall_ops validate-flowchart <spec.json>");
        eprintln!("  recall_ops inspect <board.json>");
        eprintln!("  recall_ops ops <ops.json> --out <board.json>");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "flowchart" => cmd_flowchart(&args[2..]),
        "validate-flowchart" => cmd_validate(&args[2..]),
        "inspect" => cmd_inspect(&args[2..]),
        "ops" => cmd_ops(&args[2..]),
        _ => { eprintln!("Unknown command: {}", args[1]); std::process::exit(1); }
    }
}

fn cmd_flowchart(args: &[String]) {
    if args.len() < 3 || args[1] != "--out" {
        eprintln!("Usage: recall_ops flowchart <spec.json> --out <board.json>");
        std::process::exit(1);
    }
    let spec_path = &args[0];
    let out_path = &args[2];
    let spec_json = fs::read_to_string(spec_path).unwrap_or_else(|e| {
        eprintln!("Failed to read spec: {e}"); std::process::exit(1);
    });
    let spec: FlowchartSpec = serde_json::from_str(&spec_json).unwrap_or_else(|e| {
        eprintln!("Failed to parse spec: {e}"); std::process::exit(1);
    });
    match flowchart::spec_to_board(&spec) {
        Ok(board) => {
            storage::save_board(&board, out_path).unwrap_or_else(|e| {
                eprintln!("Failed to save board: {e}"); std::process::exit(1);
            });
            println!("Board created: {} objects at {out_path}", board.canvas_objects.len());
        }
        Err(errors) => {
            for e in &errors { eprintln!("  Error: {e}"); }
            std::process::exit(1);
        }
    }
}

fn cmd_validate(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: recall_ops validate-flowchart <spec.json>");
        std::process::exit(1);
    }
    let spec_json = fs::read_to_string(&args[0]).unwrap_or_else(|e| {
        eprintln!("Failed to read spec: {e}"); std::process::exit(1);
    });
    let spec: FlowchartSpec = serde_json::from_str(&spec_json).unwrap_or_else(|e| {
        eprintln!("Parse error: {e}"); std::process::exit(1);
    });
    match flowchart::validate_spec(&spec) {
        Ok(()) => println!("PASS: spec is valid"),
        Err(errors) => {
            for e in &errors { eprintln!("  FAIL: {e}"); }
            std::process::exit(1);
        }
    }
}

fn cmd_inspect(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: recall_ops inspect <board.json>");
        std::process::exit(1);
    }
    match storage::load_board(&args[0]) {
        Ok(board) => {
            println!("Title: {}", board.title);
            println!("Objects: {}", board.canvas_objects.len());
            let strokes = board.canvas_objects.iter().filter(|o| matches!(o, recall_sketch_notes::model::CanvasObject::Stroke(_))).count();
            let texts = board.canvas_objects.iter().filter(|o| matches!(o, recall_sketch_notes::model::CanvasObject::TextNote(_))).count();
            let shapes = board.canvas_objects.iter().filter(|o| matches!(o, recall_sketch_notes::model::CanvasObject::Shape(_))).count();
            println!("  Strokes: {strokes}");
            println!("  TextNotes: {texts}");
            println!("  Shapes: {shapes}");
            let arrows = board.canvas_objects.iter().filter(|o| {
                matches!(o, recall_sketch_notes::model::CanvasObject::Shape(s) if s.shape_type == recall_sketch_notes::model::ShapeType::Arrow)
            }).count();
            println!("  Arrows: {arrows}");
        }
        Err(e) => {
            eprintln!("Failed to load board: {e}");
            std::process::exit(1);
        }
    }
}

fn cmd_ops(args: &[String]) {
    if args.len() < 3 || args[1] != "--out" {
        eprintln!("Usage: recall_ops ops <ops.json> --out <board.json>");
        std::process::exit(1);
    }
    let ops_json = fs::read_to_string(&args[0]).unwrap_or_else(|e| {
        eprintln!("Failed to read ops: {e}"); std::process::exit(1);
    });
    let ops: Vec<canvas_ops::CanvasOp> = serde_json::from_str(&ops_json).unwrap_or_else(|e| {
        eprintln!("Failed to parse ops: {e}"); std::process::exit(1);
    });
    let mut board = Board::new("Ops-generated board");
    let mut next_id: u64 = 1;
    let results = canvas_ops::apply_ops(&mut board, &ops, &mut next_id);
    let ok_count = results.iter().filter(|r| r.ok).count();
    let err_count = results.iter().filter(|r| !r.ok).count();
    for r in &results {
        if !r.ok {
            eprintln!("  Op error: {}", r.error.as_deref().unwrap_or("unknown"));
        }
    }
    storage::save_board(&board, &args[2]).unwrap_or_else(|e| {
        eprintln!("Failed to save board: {e}"); std::process::exit(1);
    });
    println!("Applied {ok_count} ops ({err_count} errors), {} objects at {}", board.canvas_objects.len(), &args[2]);
}
