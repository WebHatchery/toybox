#![allow(unused_imports)]

use macroquad::prelude::*;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use toybox_after_hours::capture_scenes;
use toybox_after_hours::data::*;
use toybox_after_hours::state::*;
use toybox_after_hours::toys::*;

#[path = "state_tests/pickup_and_movement.rs"]
mod pickup_and_movement;
#[path = "state_tests/placement.rs"]
mod placement;
#[path = "state_tests/records.rs"]
mod records;
#[path = "state_tests/repair_and_tools.rs"]
mod repair_and_tools;
#[path = "state_tests/replay.rs"]
mod replay;
#[path = "state_tests/save_migration.rs"]
mod save_migration;
#[path = "state_tests/session_setup.rs"]
mod session_setup;
#[path = "state_tests/shift_clock.rs"]
mod shift_clock;
#[path = "state_tests/shift_summary.rs"]
mod shift_summary;
#[path = "state_tests/upgrade_effects.rs"]
mod upgrade_effects;
#[path = "state_tests/zone_progress.rs"]
mod zone_progress;
