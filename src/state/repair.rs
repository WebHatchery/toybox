use super::collision::keep_off_fixtures;
use super::{
    seeded_offset, GameSession, InteractionResult, RepairPartKind, RepairState, ToyState,
    WorldPoint,
};
use crate::data::{BenchDef, GameData};
use macroquad::prelude::*;

impl ToyState {
    pub fn is_repair_part(&self) -> bool {
        matches!(self.repair_state, RepairState::BrokenPart { .. })
    }

    pub fn is_consumed_repair_part(&self) -> bool {
        matches!(self.repair_state, RepairState::ConsumedPart { .. })
    }

    pub fn repair_part_kind(&self) -> Option<RepairPartKind> {
        match self.repair_state {
            RepairState::BrokenPart { part, .. } => Some(part),
            _ => None,
        }
    }

    fn repair_id(&self) -> Option<&str> {
        match &self.repair_state {
            RepairState::BrokenPart { repair_id, .. } => Some(repair_id),
            RepairState::ConsumedPart { repair_id } => Some(repair_id),
            RepairState::Whole => None,
        }
    }

    fn repaired_name(&self) -> Option<&str> {
        match &self.repair_state {
            RepairState::BrokenPart { repaired_name, .. } => Some(repaired_name),
            _ => None,
        }
    }
}

/// What a bench is holding, readable at a glance from across the room.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BenchStatus {
    pub filled: usize,
    pub capacity: usize,
    pub stage: BenchStage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchStage {
    /// No parts waiting.
    Empty,
    /// Parts waiting, but the set is not complete yet.
    AwaitingMatch,
    /// Every slot filled by parts from different broken toys.
    Mismatched,
    /// Every slot filled by parts of the same toy — pressing E repairs it.
    Ready,
}

/// Where the other half of a carried repair part currently sits — what the
/// Toy Scanner reports once it is unlocked.
#[derive(Debug, Clone, Copy)]
pub struct CounterpartLocation {
    pub part: RepairPartKind,
    pub position: WorldPoint,
    pub on_bench: bool,
}

impl RepairPartKind {
    pub fn label(self) -> &'static str {
        match self {
            RepairPartKind::Head => "head",
            RepairPartKind::Body => "body",
        }
    }

    /// Breaks always produce exactly one head and one body per `repair_id`
    /// (see [`split_initial_broken_toys`]), so the counterpart of a part is
    /// simply the other kind — no scan over the toy list needed.
    fn counterpart(self) -> Self {
        match self {
            RepairPartKind::Head => RepairPartKind::Body,
            RepairPartKind::Body => RepairPartKind::Head,
        }
    }
}

impl GameSession {
    pub fn is_near_repair_bench(&self, data: &GameData) -> bool {
        self.nearest_bench(data).is_some()
    }

    /// The closest bench whose interaction radius contains the player.
    fn nearest_bench<'a>(&self, data: &'a GameData) -> Option<&'a BenchDef> {
        let player = self.player.position.to_vec2();
        data.layout
            .benches
            .iter()
            .map(|bench| (bench, player.distance_squared(bench_point(bench).to_vec2())))
            .filter(|(bench, distance_sq)| *distance_sq <= bench.radius * bench.radius)
            .min_by(|(_, left), (_, right)| left.total_cmp(right))
            .map(|(bench, _)| bench)
    }

    pub(super) fn repair_bench_has_room(&self, data: &GameData) -> bool {
        self.nearest_bench(data)
            .is_some_and(|bench| self.next_bench_slot(bench).is_some())
    }

    pub(super) fn repair_bench_is_full(&self, data: &GameData) -> bool {
        self.nearest_bench(data)
            .is_some_and(|bench| self.benched_toy_indices(bench).len() >= bench.capacity)
    }

    pub(super) fn benched_repair_name(&self, data: &GameData) -> Option<String> {
        let bench = self.nearest_bench(data)?;
        let part_indices = self.benched_repair_part_indices(bench);
        if part_indices.len() != bench.capacity {
            return None;
        }
        self.matched_repair_name(&part_indices)
    }

    /// The repaired name shared by every part in `part_indices`, or `None` if
    /// they came from different breaks.
    fn matched_repair_name(&self, part_indices: &[usize]) -> Option<String> {
        let first_part = &self.toys[*part_indices.first()?];
        let repair_id = first_part.repair_id()?;
        let matching_parts = part_indices
            .iter()
            .all(|&toy_index| self.toys[toy_index].repair_id() == Some(repair_id));

        matching_parts.then(|| {
            first_part
                .repaired_name()
                .unwrap_or(&first_part.name)
                .to_owned()
        })
    }

    /// The other half of the part the player is carrying, wherever it ended up.
    /// `None` when nothing is carried, the carried toy is whole, or the
    /// counterpart has already been consumed by a repair.
    pub fn carried_counterpart(&self) -> Option<CounterpartLocation> {
        let active = self.active_toy()?;
        let repair_id = active.repair_id()?;
        let counterpart = self.toys.iter().find(|toy| {
            toy.id != active.id && toy.is_repair_part() && toy.repair_id() == Some(repair_id)
        })?;

        Some(CounterpartLocation {
            part: counterpart.repair_part_kind()?,
            position: counterpart.position,
            on_bench: counterpart.bench_slot_index.is_some(),
        })
    }

    /// Per-bench summary for the renderer, so a player can read every bench
    /// from across the room instead of walking to each one.
    pub fn bench_status(&self, bench: &BenchDef) -> BenchStatus {
        let part_indices = self.benched_repair_part_indices(bench);
        let filled = part_indices.len();
        let stage = if filled == 0 {
            BenchStage::Empty
        } else if filled < bench.capacity {
            BenchStage::AwaitingMatch
        } else if self.matched_repair_name(&part_indices).is_some() {
            BenchStage::Ready
        } else {
            BenchStage::Mismatched
        };

        BenchStatus {
            filled,
            capacity: bench.capacity,
            stage,
        }
    }

    /// A part left waiting on the nearest bench whose counterpart is still out
    /// in the store, as `(repaired name, missing part)`. `None` once the bench
    /// is empty or every slot is filled.
    pub fn lone_benched_part(&self, data: &GameData) -> Option<(String, RepairPartKind)> {
        let bench = self.nearest_bench(data)?;
        let part_indices = self.benched_repair_part_indices(bench);
        if part_indices.len() != 1 {
            return None;
        }

        let waiting = &self.toys[part_indices[0]];
        let missing = waiting.repair_part_kind()?.counterpart();
        Some((
            waiting.repaired_name().unwrap_or(&waiting.name).to_owned(),
            missing,
        ))
    }

    /// Whether `carried` belongs to the same broken toy as everything already
    /// waiting on the nearest bench. True for an empty bench, so the caller
    /// only has to check it before offering a place-on-bench prompt.
    pub(super) fn carried_part_matches_bench(&self, data: &GameData, carried: &ToyState) -> bool {
        let Some(bench) = self.nearest_bench(data) else {
            return true;
        };
        let Some(carried_repair_id) = carried.repair_id() else {
            return true;
        };
        self.benched_repair_part_indices(bench)
            .iter()
            .all(|&toy_index| self.toys[toy_index].repair_id() == Some(carried_repair_id))
    }

    pub(super) fn place_active_on_repair_bench(&mut self, data: &GameData) -> InteractionResult {
        let Some(active_toy) = self.active_toy() else {
            return InteractionResult::NothingNearby;
        };
        let active_id = active_toy.id.clone();
        let active_name = active_toy.name.clone();
        if !active_toy.is_repair_part() {
            return InteractionResult::NothingNearby;
        };
        let Some(bench) = self.nearest_bench(data) else {
            return InteractionResult::NothingNearby;
        };
        // Refuse a part from a different break. Accepting it fills the bench
        // with two halves that can never be joined, and the only way out is to
        // notice, aim at one, and take it back — a dead end the game gave no
        // hint about. Refusing means a bench is always empty, holding one
        // part, or holding a pair that repairs.
        if !self.carried_part_matches_bench(data, active_toy) {
            return InteractionResult::RepairMismatch;
        }
        let Some(slot_index) = self.next_bench_slot(bench) else {
            return InteractionResult::RepairBenchFull;
        };
        let Some(active_index) = self.toys.iter().position(|toy| toy.id == active_id) else {
            return InteractionResult::NothingNearby;
        };

        self.toys[active_index].is_held = false;
        self.toys[active_index].placed_display_id = None;
        self.toys[active_index].placed_slot_index = None;
        self.toys[active_index].bench_slot_index = Some(slot_index);
        self.toys[active_index].bench_id = Some(bench.id.clone());
        self.toys[active_index].wrong_marker_seconds = 0.0;
        self.toys[active_index].position = repair_bench_slot_position(bench, slot_index);
        self.spatial
            .sync_toy(active_index, &self.toys[active_index]);

        self.player
            .carried_toy_ids
            .retain(|toy_id| toy_id != &active_id);
        self.normalize_active_carry(self.carry_limit(data));

        InteractionResult::PlacedOnRepairBench {
            toy_name: active_name,
        }
    }

    pub(super) fn repair_benched_toys(&mut self, data: &GameData) -> InteractionResult {
        let Some(bench) = self.nearest_bench(data) else {
            return InteractionResult::NothingNearby;
        };
        let consumed_rest_position = repair_bench_slot_position(bench, 1);
        let part_indices = self.benched_repair_part_indices(bench);
        if part_indices.len() != bench.capacity {
            return InteractionResult::NeedsRepairParts {
                toy_name: "repair".to_owned(),
            };
        }
        let Some(repaired_name) = self.benched_repair_name(data) else {
            return InteractionResult::RepairMismatch;
        };
        let survivor_index = part_indices
            .iter()
            .copied()
            .find(|&toy_index| {
                self.toys[toy_index].repair_part_kind() == Some(RepairPartKind::Body)
            })
            .unwrap_or(part_indices[0]);
        let Some(repair_id) = self.toys[survivor_index].repair_id().map(str::to_owned) else {
            return InteractionResult::RepairMismatch;
        };

        for &index in part_indices
            .iter()
            .filter(|&&toy_index| toy_index != survivor_index)
        {
            self.toys[index].is_held = false;
            self.toys[index].placed_display_id = None;
            self.toys[index].placed_slot_index = None;
            self.toys[index].bench_slot_index = None;
            self.toys[index].bench_id = None;
            self.toys[index].wrong_marker_seconds = 0.0;
            self.toys[index].position = consumed_rest_position;
            self.toys[index].repair_state = RepairState::ConsumedPart {
                repair_id: repair_id.clone(),
            };
            self.spatial.sync_toy(index, &self.toys[index]);
        }

        self.toys[survivor_index].name = repaired_name.clone();
        self.toys[survivor_index].is_held = true;
        self.toys[survivor_index].placed_display_id = None;
        self.toys[survivor_index].placed_slot_index = None;
        self.toys[survivor_index].bench_slot_index = None;
        self.toys[survivor_index].bench_id = None;
        self.toys[survivor_index].wrong_marker_seconds = 0.0;
        self.toys[survivor_index].position = self.player.position;
        self.toys[survivor_index].repair_state = RepairState::Whole;
        self.spatial
            .sync_toy(survivor_index, &self.toys[survivor_index]);

        self.player.carried_toy_ids.clear();
        self.player
            .carried_toy_ids
            .push(self.toys[survivor_index].id.clone());
        self.player.active_carry_index = 0;
        self.player.repairs += 1;

        InteractionResult::Repaired {
            toy_name: repaired_name,
        }
    }

    pub(super) fn repair_bench_slots(&mut self, data: &GameData) {
        for toy in &mut self.toys {
            // Legacy saves predate bench ids: adopt them onto the primary bench.
            if toy.bench_slot_index.is_some() && toy.bench_id.is_none() {
                toy.bench_id = Some(data.primary_bench().id.clone());
            }
            if toy.bench_slot_index.is_none()
                || toy.bench_id.as_deref().is_some_and(|bench_id| {
                    !data.layout.benches.iter().any(|bench| bench.id == bench_id)
                })
            {
                toy.bench_slot_index = None;
                toy.bench_id = None;
            }
        }

        for bench in &data.layout.benches {
            let mut used_slots = vec![false; bench.capacity];
            for toy in &mut self.toys {
                if toy.bench_id.as_deref() != Some(bench.id.as_str()) {
                    continue;
                }
                let Some(slot_index) = toy.bench_slot_index else {
                    continue;
                };

                if toy.is_held
                    || !toy.is_repair_part()
                    || slot_index >= bench.capacity
                    || used_slots[slot_index]
                {
                    toy.bench_slot_index = None;
                    toy.bench_id = None;
                    continue;
                }

                used_slots[slot_index] = true;
                toy.placed_display_id = None;
                toy.placed_slot_index = None;
                toy.position = repair_bench_slot_position(bench, slot_index);
            }
        }
    }

    fn next_bench_slot(&self, bench: &BenchDef) -> Option<usize> {
        let benched = self.benched_toy_indices(bench);
        (0..bench.capacity).find(|slot_index| {
            !benched
                .iter()
                .any(|&toy_index| self.toys[toy_index].bench_slot_index == Some(*slot_index))
        })
    }

    fn benched_toy_indices(&self, bench: &BenchDef) -> Vec<usize> {
        self.spatial
            .indices_near(bench_point(bench).to_vec2(), bench.radius)
            .into_iter()
            .filter(|&toy_index| {
                let toy = &self.toys[toy_index];
                toy.bench_slot_index.is_some() && toy.bench_id.as_deref() == Some(bench.id.as_str())
            })
            .collect()
    }

    fn benched_repair_part_indices(&self, bench: &BenchDef) -> Vec<usize> {
        let mut parts: Vec<(usize, usize)> = self
            .benched_toy_indices(bench)
            .into_iter()
            .filter(|&toy_index| self.toys[toy_index].is_repair_part())
            .filter_map(|toy_index| {
                self.toys[toy_index]
                    .bench_slot_index
                    .map(|slot_index| (slot_index, toy_index))
            })
            .collect();
        parts.sort_by_key(|(slot_index, _)| *slot_index);
        parts.into_iter().map(|(_, toy_index)| toy_index).collect()
    }
}

fn bench_point(bench: &BenchDef) -> WorldPoint {
    WorldPoint {
        x: bench.x,
        y: bench.y,
    }
}

fn repair_bench_slot_position(bench: &BenchDef, slot_index: usize) -> WorldPoint {
    let x_offset = if slot_index == 0 { -0.38 } else { 0.38 };
    WorldPoint {
        x: bench.x + x_offset,
        y: bench.y - 0.08,
    }
}

/// Deterministically break a `broken_fraction` share of freshly spawned toys
/// into a body (kept in place) and a head scattered into a different zone.
pub(super) fn split_initial_broken_toys(
    toys: &mut Vec<ToyState>,
    shift_seed: u64,
    data: &GameData,
) {
    let broken_per_mille = (data.config.broken_fraction.clamp(0.0, 0.5) * 1000.0) as usize;
    if broken_per_mille == 0 {
        return;
    }

    let original_count = toys.len();
    let mut heads = Vec::with_capacity(original_count / 6);
    let selector_offset = seeded_offset(shift_seed, 0xB20C_EA75, 1000);
    for (index, toy) in toys.iter_mut().enumerate() {
        if (index * 379 + 97 + selector_offset) % 1000 >= broken_per_mille {
            continue;
        }
        if toy.is_repair_part() {
            continue;
        }

        let repair_id = format!("repair_{index:04}");
        let repaired_name = toy.name.clone();
        let mut head = toy.clone();
        head.id = format!("{}_head", toy.id);
        head.name = format!("{repaired_name} Head");
        head.position = head_scatter_position(index, toy.position, shift_seed, data);
        head.repair_state = RepairState::BrokenPart {
            repair_id: repair_id.clone(),
            part: RepairPartKind::Head,
            repaired_name: repaired_name.clone(),
        };

        toy.name = format!("{repaired_name} Body");
        toy.repair_state = RepairState::BrokenPart {
            repair_id,
            part: RepairPartKind::Body,
            repaired_name,
        };

        heads.push(head);
    }

    toys.extend(heads);
}

/// Deterministic head placement spread uniformly inside a zone that differs
/// from the body's zone, so finding the counterpart is a real errand.
fn head_scatter_position(
    toy_index: usize,
    body: WorldPoint,
    shift_seed: u64,
    data: &GameData,
) -> WorldPoint {
    let config = &data.config;
    let body_zone = data.layout.zone_name_at(body.x, body.y);

    let place_in_rect = |x: f32, y: f32, w: f32, h: f32| {
        let hash_x = (toy_index
            .wrapping_mul(48_271)
            .wrapping_add(11)
            .wrapping_add(seeded_offset(
                shift_seed,
                toy_index as u64 ^ 0xAEAD_0001,
                9_973,
            )))
            % 9_973;
        let hash_y = (toy_index
            .wrapping_mul(69_621)
            .wrapping_add(37)
            .wrapping_add(seeded_offset(
                shift_seed,
                toy_index as u64 ^ 0xAEAD_0002,
                9_973,
            )))
            % 9_973;
        let inset = 0.5_f32;
        let px = x + inset + (hash_x as f32 / 9_973.0) * (w - inset * 2.0).max(0.1);
        let py = y + inset + (hash_y as f32 / 9_973.0) * (h - inset * 2.0).max(0.1);
        let position = keep_off_fixtures(vec2(px, py), data);
        WorldPoint {
            x: position.x.clamp(0.8, config.room_width - 0.8),
            y: position.y.clamp(0.8, config.room_height - 0.8),
        }
    };

    let candidates: Vec<&crate::data::ZoneDef> = data
        .layout
        .zones
        .iter()
        .filter(|zone| Some(zone.name.as_str()) != body_zone)
        .collect();
    if candidates.is_empty() {
        return place_in_rect(0.0, 0.0, config.room_width, config.room_height);
    }

    // Fixture nudges can drift across a zone border, so validate the final
    // position and walk to the next candidate zone if it slid back home.
    let mut fallback = None;
    for attempt in 0..candidates.len() {
        let zone_offset =
            seeded_offset(shift_seed, toy_index as u64 ^ 0x20AE_0003, candidates.len());
        let zone = candidates[(toy_index * 13 + zone_offset + attempt) % candidates.len()];
        let position = place_in_rect(zone.x, zone.y, zone.w, zone.h);
        if data.layout.zone_name_at(position.x, position.y) != body_zone {
            return position;
        }
        fallback.get_or_insert(position);
    }
    fallback.expect("at least one candidate zone was tried")
}
