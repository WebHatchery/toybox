//! Per-frame simulation, input polling, and benchmark support.

use super::*;

impl Game {
    pub fn update(&mut self, dt: f32) {
        if self.gallery.is_some() {
            return;
        }
        self.notifications.update(dt);
        self.audio.update(dt);
        ui::advance_animation_clock(dt);
        self.debug_overlay.record_frame(dt);
        self.update_debug_toggle();
        self.update_bench(dt);

        if self.screen != GameScreen::Playing {
            self.queue_overlay_navigation();
            self.apply_queued_actions();
            return;
        }

        self.update_shift_clock(dt);
        self.update_player_input(dt);
        self.queue_gameplay_shortcuts();
        self.apply_queued_actions();
        self.finish_tutorial_if_ready();
        self.record_finished_run();
    }

    fn update_debug_toggle(&mut self) {
        if self.data.config.debug_overlay_enabled && is_key_pressed(KeyCode::F3) {
            self.debug_overlay.toggle();
        }
    }

    fn queue_overlay_navigation(&mut self) {
        match self.screen {
            GameScreen::Settings if is_key_pressed(KeyCode::Escape) => {
                self.events.push(UiAction::CloseSettings);
            }
            GameScreen::Help if is_key_pressed(KeyCode::Escape) => {
                self.events.push(UiAction::CloseHelp);
            }
            GameScreen::ToolShop
                if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::T) =>
            {
                self.events.push(UiAction::CloseToolShop);
            }
            _ => {}
        }
    }

    fn update_shift_clock(&mut self, dt: f32) {
        let remaining_before = self.session.shift_remaining(&self.data);
        if self.session.update_timer(dt, &self.data) {
            self.audio.play(Cue::ClosingWarning);
            self.notifications
                .warning("The doors are open - shift over");
        }
        let remaining_after = self.session.shift_remaining(&self.data);
        if !self.session.shift_mode.shows_countdown() {
            return;
        }
        if !self.warned_five_minutes && remaining_before > 300.0 && remaining_after <= 300.0 {
            self.warned_five_minutes = true;
            self.audio.play(Cue::ClosingWarning);
            self.notifications.warning("Five minutes until opening");
        }
        if !self.warned_one_minute && remaining_before > 60.0 && remaining_after <= 60.0 {
            self.warned_one_minute = true;
            self.audio.play(Cue::ClosingWarning);
            self.notifications.danger("One minute until opening");
        }
    }

    fn update_player_input(&mut self, dt: f32) {
        let mouse_delta = ui::continuous_mouse_delta_pixels();
        let touch_look = std::mem::take(&mut self.touch_look);
        let look_delta = ui::look_delta_from_input(
            mouse_delta,
            dt,
            self.preferences.mouse_sensitivity,
            self.data.config.sensitivity_min,
            self.data.config.sensitivity_max,
        ) + touch_look * 1.75 * dt;
        self.session.update_player_look(look_delta.x, look_delta.y);

        let touch_movement = std::mem::take(&mut self.touch_movement);
        let movement = ui::movement_from_keys() + touch_movement;
        self.session.move_player(movement, &self.data, dt);
        if movement.length_squared() > 0.0 {
            self.audio.play_at(Cue::Footstep, 0.20);
        }
        self.tutorial.observe_navigation(
            movement.length_squared() > 0.0,
            look_delta.length_squared() > 0.0,
        );

        if self.tutorial.is_active() && is_key_pressed(KeyCode::H) {
            self.events.push(UiAction::SkipTutorial);
        }
    }

    fn queue_gameplay_shortcuts(&mut self) {
        if is_key_pressed(KeyCode::Escape) {
            self.events.push(UiAction::Settings);
        }
        if is_key_pressed(KeyCode::R) {
            self.events.push(match self.session.shift_mode {
                ShiftMode::Timed => UiAction::NewGame,
                ShiftMode::Relaxed => UiAction::NewRelaxedGame,
            });
        }
        if is_key_pressed(KeyCode::F5) {
            self.events.push(UiAction::ReplayShiftSeed);
        }
        if is_key_pressed(KeyCode::E) || is_key_pressed(KeyCode::Space) {
            self.events.push(UiAction::Interact);
        }
        if is_key_pressed(KeyCode::Q) {
            self.events.push(UiAction::CycleCarry);
        }
        if is_key_pressed(KeyCode::G) {
            self.events.push(UiAction::DropActive);
        }
        if is_key_pressed(KeyCode::T) {
            self.events.push(UiAction::OpenToolShop);
        }
        if is_key_pressed(KeyCode::S) && is_control_down() {
            self.events.push(UiAction::Save);
        }
        if is_key_pressed(KeyCode::L) && is_control_down() {
            self.events.push(UiAction::Load);
        }
    }

    fn record_finished_run(&mut self) {
        if self.recorded_run || !self.session.phase.is_over() {
            return;
        }
        self.recorded_run = true;

        let summary = self.session.shift_summary(&self.data);
        let restored = self.session.phase == GamePhase::Finished;
        let run = ShiftRecord::from_summary(&summary, restored);
        self.beat_record = self.best_runs.submit(self.session.shift_mode, run);
        if !self.beat_record {
            return;
        }

        if let Err(err) = self.best_runs.save(
            &self.data.config.game_name,
            &self.data.config.records_slot,
            &self.data.config.version,
        ) {
            self.notifications
                .danger(format!("Could not save your best run: {}", err));
        }
    }

    fn update_bench(&mut self, dt: f32) {
        let Some(bench) = &mut self.bench else {
            return;
        };
        bench.frames += 1;
        bench.elapsed_seconds += dt;

        let frame_seconds = get_frame_time();
        if bench.frames > BENCH_WARMUP_FRAMES {
            bench.measured_frames += 1;
            bench.worst_frame_seconds = bench.worst_frame_seconds.max(frame_seconds);
            if frame_seconds > BENCH_SLOW_FRAME_SECONDS {
                bench.slow_frames += 1;
            }
        }

        self.session.update_player_look(0.55 * dt, 0.0);

        if bench.elapsed_seconds >= bench.duration_seconds {
            let average_fps = bench.frames as f32 / bench.elapsed_seconds.max(f32::EPSILON);
            println!(
                "BENCH toys={} frames={} seconds={:.2} avg_fps={:.1} \
                 worst_frame_ms={:.2} slow_frames={}/{} (>{:.1}ms, after {} warm-up)",
                self.session.toys.len(),
                bench.frames,
                bench.elapsed_seconds,
                average_fps,
                bench.worst_frame_seconds * 1000.0,
                bench.slow_frames,
                bench.measured_frames,
                BENCH_SLOW_FRAME_SECONDS * 1000.0,
                BENCH_WARMUP_FRAMES,
            );
            quit();
        }
    }

    fn apply_queued_actions(&mut self) {
        let actions: Vec<UiAction> = self.events.drain().collect();
        for action in actions {
            self.apply_action(action);
        }
    }
}
