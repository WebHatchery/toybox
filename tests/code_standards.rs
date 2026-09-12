// The shared file-size gate from CODE_STANDARDS §2.2 — the 800-line hard
// limit on total physical source lines — enforced under plain `cargo test`.

use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn source_files_stay_under_the_limit() {
    macroquad_toolkit::source_gate::assert_source_files_within_limit(
        env!("CARGO_MANIFEST_DIR"),
        &[],
    );
}

/// The view layer reads the session and returns `UiAction`s; a dispatcher
/// applies them. Handing rendering a `&mut GameSession` would let a camera
/// angle or a visual effect change what the player scored, and the damage
/// would be invisible — no test of the simulation would ever see it.
/// `UiContext` holds a shared reference today; this keeps it that way.
#[test]
fn the_view_layer_cannot_mutate_the_session() {
    let ui_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui");
    let mut sources = vec![Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui.rs")];
    collect_rust_sources(&ui_root, &mut sources);
    assert!(
        sources.len() > 5,
        "expected to find the ui modules, found {sources:?}"
    );

    let mut offenders = Vec::new();
    for path in &sources {
        let text = fs::read_to_string(path).expect("read ui source");
        for (line_number, line) in text.lines().enumerate() {
            if line.contains("&mut GameSession") || line.contains("session: &mut") {
                offenders.push(format!(
                    "{}:{}: {}",
                    path.display(),
                    line_number + 1,
                    line.trim()
                ));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "the view layer must not take the session mutably; emit a UiAction and \
         let Game::apply_action mutate it instead:\n{}",
        offenders.join("\n")
    );
}

/// Rendering must animate off the simulation clock, never the wall clock.
///
/// The screenshot harness simulates a fixed number of frames at a fixed
/// timestep so a capture is reproducible. Four `get_time()` calls — a lamp
/// breathing, dust motes, completion sparkles, the scanner beacon — defeated
/// that: the same scene captured twice came out with different bytes, because
/// the animation phase depended on how long the process took to reach the
/// capture frame. A reference screenshot that changes every time it is taken
/// cannot be diffed, which is why fifty stale toy images went unnoticed for
/// three weeks. `ui::animation_seconds()` accumulates the same `dt` the
/// simulation gets, and this keeps the wall clock out.
#[test]
fn rendering_animates_off_the_simulation_clock() {
    let ui_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui");
    let toys_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/toys");
    let mut sources = vec![
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui.rs"),
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/toys.rs"),
    ];
    collect_rust_sources(&ui_root, &mut sources);
    collect_rust_sources(&toys_root, &mut sources);
    assert!(
        sources.len() > 20,
        "expected the ui and toy modules, found {}",
        sources.len()
    );

    let mut offenders = Vec::new();
    for path in &sources {
        let text = fs::read_to_string(path).expect("read render source");
        for (line_number, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or("");
            if code.contains("get_time()") {
                offenders.push(format!(
                    "{}:{}: {}",
                    path.display(),
                    line_number + 1,
                    line.trim()
                ));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "rendering read the wall clock, which makes screenshot captures \
         irreproducible; use `crate::ui::animation_seconds()`:\n{}",
        offenders.join("\n")
    );
}

fn collect_rust_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rust_sources(&path, out);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            out.push(path);
        }
    }
}
