use super::{
    display_slot_position, toy_matches_display, GamePhase, GameSession, InteractionPreview,
    InteractionResult, ToyState,
};
use crate::data::GameData;

mod targeting;

impl GameSession {
    pub fn interact(&mut self, data: &GameData) -> InteractionResult {
        if self.phase != GamePhase::Playing {
            return InteractionResult::NothingNearby;
        }

        if let Some(active_toy) = self.active_toy() {
            if active_toy.is_repair_part() {
                if self.is_near_repair_bench(data) {
                    return self.place_active_on_repair_bench(data);
                }
                if self.targeted_display_slot(data).is_some() || self.is_near_display(data) {
                    return InteractionResult::NeedsRepair {
                        toy_name: active_toy.name.clone(),
                    };
                }
                if let Some(toy_index) = self.targeted_pickup_addition(data) {
                    return self.pick_up_toy(toy_index, data);
                }
                return self.drop_active_as_interaction(data);
            }
            // Same rule as the empty-handed branch below: the pitch-aware
            // crosshair beats 2D shelf targeting. Looking down at a toy loads it
            // onto the trolley; looking at the shelf shelves what is in hand,
            // which is what a player does anyway to choose a slot. This never
            // fires at a carry limit of one, so a bare-handed player's `E` still
            // shelves exactly as before.
            if let Some(toy_index) = self.targeted_pickup_addition(data) {
                return self.pick_up_toy(toy_index, data);
            }
            if let Some(target) = self.targeted_empty_display_slot(data) {
                return self.place_active_toy(target.display_index, target.slot_index, data);
            }
            if let Some(target) = self.targeted_display_slot(data) {
                let display = &data.displays[target.display_index];
                if self.display_is_full(display, data.config.room_width) {
                    return InteractionResult::ShelfFull;
                }
                return InteractionResult::ShelfSlotUnavailable;
            }
            if self.is_near_display(data) {
                return InteractionResult::ShelfSlotUnavailable;
            }
            return self.drop_active_as_interaction(data);
        }

        if self.is_near_repair_bench(data) && self.benched_repair_name(data).is_some() {
            return self.repair_benched_toys(data);
        }

        // The loose-toy test accounts for pitch; shelf-slot targeting is 2D from
        // yaw alone, so a player looking at their own feet still "looks at"
        // every slot in front of them. Asking the precise test first is what
        // stops standing at a stocked shelf and reaching for a toy underfoot
        // from lifting something back off the display instead. Looking level at
        // the shelf puts the floor toy outside the 3D cone, so retrieval still
        // works — see the pair of tests in `tests/pickup_and_movement.rs`.
        if let Some(toy_index) = self.targeted_loose_toy_index(data) {
            if self.player.carried_toy_ids.len() >= self.carry_limit(data) {
                return InteractionResult::InventoryFull;
            }
            return self.pick_up_toy(toy_index, data);
        }

        if let Some(target) = self.targeted_display_slot(data) {
            let display = &data.displays[target.display_index];
            if let Some(toy_index) =
                self.toy_index_at_display_slot(display, target.slot_index, data.config.room_width)
            {
                if self.player.carried_toy_ids.len() >= self.carry_limit(data) {
                    return InteractionResult::InventoryFull;
                }
                return self.pick_up_placed_toy(toy_index, data);
            }
        }

        if self.player.carried_toy_ids.len() >= self.carry_limit(data) {
            return InteractionResult::InventoryFull;
        }

        if self.is_near_repair_bench(data) && self.repair_bench_is_full(data) {
            return InteractionResult::RepairMismatch;
        }

        InteractionResult::NothingNearby
    }

    pub fn interaction_preview(&self, data: &GameData) -> InteractionPreview {
        if self.phase != GamePhase::Playing {
            return match self.phase {
                GamePhase::TimeUp => InteractionPreview::ShiftOver,
                _ => InteractionPreview::Finished,
            };
        }

        if let Some(active_toy) = self.active_toy() {
            if active_toy.is_repair_part() {
                if self.is_near_repair_bench(data) {
                    // Warn before the placement rather than after the failed
                    // repair: the part already waiting belongs to another toy.
                    if !self.carried_part_matches_bench(data, active_toy) {
                        return InteractionPreview::RepairMismatch;
                    }
                    if self.repair_bench_has_room(data) {
                        return InteractionPreview::PlaceOnRepairBench;
                    }
                    return InteractionPreview::RepairBenchFull;
                }
                if self.targeted_display_slot(data).is_some() || self.is_near_display(data) {
                    return InteractionPreview::NeedsRepair;
                }
                if let Some(toy_index) = self.targeted_pickup_addition(data) {
                    return InteractionPreview::Pickup {
                        toy_name: self.toys[toy_index].name.clone(),
                    };
                }
                return InteractionPreview::PutDown;
            }
            if let Some(toy_index) = self.targeted_pickup_addition(data) {
                return InteractionPreview::Pickup {
                    toy_name: self.toys[toy_index].name.clone(),
                };
            }
            if self.targeted_empty_display_slot(data).is_some() {
                return InteractionPreview::PlaceOnShelf;
            }
            if let Some(target) = self.targeted_display_slot(data) {
                let display = &data.displays[target.display_index];
                if self.display_is_full(display, data.config.room_width) {
                    return InteractionPreview::ShelfFull;
                }
                return InteractionPreview::LookAtEmptySlot;
            }
            if self.is_near_display(data) {
                return InteractionPreview::LookAtEmptySlot;
            }
            return InteractionPreview::PutDown;
        }

        if self.is_near_repair_bench(data) {
            if let Some(toy_name) = self.benched_repair_name(data) {
                return InteractionPreview::RepairReady { toy_name };
            }
        }

        // Mirrors `interact`: the pitch-aware test wins over the 2D one.
        if let Some(toy_index) = self.targeted_loose_toy_index(data) {
            if self.player.carried_toy_ids.len() >= self.carry_limit(data) {
                return InteractionPreview::InventoryFull;
            }
            return InteractionPreview::Pickup {
                toy_name: self.toys[toy_index].name.clone(),
            };
        }

        if let Some(target) = self.targeted_display_slot(data) {
            let display = &data.displays[target.display_index];
            if let Some(toy) =
                self.toy_at_display_slot(display, target.slot_index, data.config.room_width)
            {
                if self.player.carried_toy_ids.len() >= self.carry_limit(data) {
                    return InteractionPreview::InventoryFull;
                }
                return InteractionPreview::Pickup {
                    toy_name: toy.name.clone(),
                };
            }
        }

        if self.player.carried_toy_ids.len() >= self.carry_limit(data) {
            return InteractionPreview::InventoryFull;
        }

        if self.is_near_repair_bench(data) {
            if self.repair_bench_is_full(data) {
                return InteractionPreview::RepairMismatch;
            }
            // Lower priority than picking something up, so standing at a bench
            // never hides a nearby toy's prompt.
            if let Some((toy_name, missing_part)) = self.lone_benched_part(data) {
                return InteractionPreview::AwaitingRepairMatch {
                    toy_name,
                    missing_part,
                };
            }
        }

        InteractionPreview::NothingNearby
    }

    pub fn placed_toys_for_display<'a>(
        &'a self,
        display_id: &'a str,
    ) -> impl Iterator<Item = &'a ToyState> + 'a {
        self.toys.iter().filter(move |toy| {
            toy.placed_display_id
                .as_deref()
                .is_some_and(|placed_id| placed_id == display_id)
        })
    }

    fn drop_active_as_interaction(&mut self, data: &GameData) -> InteractionResult {
        match self.drop_active(data) {
            Some(toy_name) => InteractionResult::Dropped { toy_name },
            None => InteractionResult::NothingNearby,
        }
    }

    pub fn pick_up_toy(&mut self, toy_index: usize, data: &GameData) -> InteractionResult {
        let carry_limit = self.carry_limit(data);
        let toy_id = self.toys[toy_index].id.clone();
        let toy_name = self.toys[toy_index].name.clone();
        let already_carried = self.player.carried_toy_ids.iter().any(|id| id == &toy_id);
        if !already_carried && self.player.carried_toy_ids.len() >= carry_limit {
            return InteractionResult::InventoryFull;
        }

        self.toys[toy_index].is_held = true;
        self.toys[toy_index].placed_display_id = None;
        self.toys[toy_index].placed_slot_index = None;
        self.toys[toy_index].bench_slot_index = None;
        self.toys[toy_index].bench_id = None;
        self.toys[toy_index].wrong_marker_seconds = 0.0;
        self.spatial.sync_toy(toy_index, &self.toys[toy_index]);
        if !self.player.carried_toy_ids.iter().any(|id| id == &toy_id) {
            self.player.carried_toy_ids.push(toy_id);
        }
        self.player.active_carry_index = self.player.carried_toy_ids.len().saturating_sub(1);

        InteractionResult::PickedUp { toy_name }
    }

    pub fn place_active_toy(
        &mut self,
        display_index: usize,
        slot_index: usize,
        data: &GameData,
    ) -> InteractionResult {
        let Some(display) = data.displays.get(display_index) else {
            return InteractionResult::NothingNearby;
        };
        if slot_index >= display.capacity {
            return InteractionResult::ShelfSlotUnavailable;
        }
        if self
            .toy_index_at_display_slot(display, slot_index, data.config.room_width)
            .is_some()
        {
            return if self.display_is_full(display, data.config.room_width) {
                InteractionResult::ShelfFull
            } else {
                InteractionResult::ShelfSlotUnavailable
            };
        }

        let toy_id = match self.active_toy() {
            Some(toy) => toy.id.clone(),
            None => return InteractionResult::NothingNearby,
        };
        let toy_index = match self.toys.iter().position(|toy| toy.id == toy_id) {
            Some(index) => index,
            None => return InteractionResult::NothingNearby,
        };
        let toy_name = self.toys[toy_index].name.clone();
        if self.toys[toy_index].is_repair_part() {
            return InteractionResult::NeedsRepair { toy_name };
        }
        let previous_completed_count = self.completed_display_count();
        let previously_restored_zones = self.restored_zone_names(data);

        let is_wrong_display = !toy_matches_display(&self.toys[toy_index], display);
        if is_wrong_display {
            self.player.mistakes += 1;
            if self.player.mistake_guards_remaining > 0 {
                self.player.mistake_guards_remaining -= 1;
                return InteractionResult::PlacementPrevented {
                    toy_name,
                    display_name: display.name.clone(),
                    guards_remaining: self.player.mistake_guards_remaining,
                };
            }
            self.player.elapsed_seconds += data.config.mistake_penalty_seconds;
        }

        self.toys[toy_index].is_held = false;
        self.toys[toy_index].placed_display_id = Some(display.id.clone());
        self.toys[toy_index].placed_slot_index = Some(slot_index);
        self.toys[toy_index].bench_slot_index = None;
        self.toys[toy_index].bench_id = None;
        self.toys[toy_index].wrong_marker_seconds = if is_wrong_display {
            Self::WRONG_MARKER_SECONDS
        } else {
            0.0
        };
        self.toys[toy_index].position =
            display_slot_position(display, slot_index, data.config.room_width);
        self.spatial.sync_toy(toy_index, &self.toys[toy_index]);
        self.player.carried_toy_ids.retain(|id| id != &toy_id);
        self.normalize_active_carry(self.carry_limit(data));

        let was_complete = self.is_display_complete(&display.id);
        self.refresh_display_completion(data);
        let completed_display = if !was_complete && self.is_display_complete(&display.id) {
            Some(display.name.clone())
        } else {
            None
        };
        let completed_zone = self
            .restored_zone_names(data)
            .into_iter()
            .find(|zone| !previously_restored_zones.contains(zone));
        let available_tools = self.newly_available_upgrades(data, previous_completed_count);
        let finished = self
            .displays
            .iter()
            .all(|display_state| display_state.is_complete);
        if finished {
            self.phase = GamePhase::Finished;
        }

        InteractionResult::Placed {
            toy_name,
            display_name: display.name.clone(),
            was_wrong: is_wrong_display,
            completed_display,
            completed_zone,
            available_tools,
            finished,
        }
    }

    pub(super) fn repair_display_slots(&mut self, data: &GameData) {
        let player_position = self.player.position;
        for toy in &mut self.toys {
            if toy.is_held {
                toy.placed_display_id = None;
                toy.placed_slot_index = None;
                toy.bench_slot_index = None;
                toy.bench_id = None;
                continue;
            }

            let Some(display_id) = toy.placed_display_id.as_deref() else {
                if toy.bench_slot_index.is_none() {
                    toy.placed_slot_index = None;
                }
                continue;
            };
            toy.bench_slot_index = None;
            toy.bench_id = None;
            if data.display_by_id(display_id).is_none() {
                toy.placed_display_id = None;
                toy.placed_slot_index = None;
            }
        }

        for display in &data.displays {
            let mut used_slots = vec![false; display.capacity];

            for toy in &mut self.toys {
                if toy.placed_display_id.as_deref() != Some(display.id.as_str()) {
                    continue;
                }

                match toy.placed_slot_index {
                    Some(slot_index)
                        if slot_index < display.capacity && !used_slots[slot_index] =>
                    {
                        used_slots[slot_index] = true;
                        toy.position =
                            display_slot_position(display, slot_index, data.config.room_width);
                    }
                    _ => {
                        toy.placed_slot_index = None;
                    }
                }
            }

            for toy in &mut self.toys {
                if toy.placed_display_id.as_deref() != Some(display.id.as_str())
                    || toy.placed_slot_index.is_some()
                {
                    continue;
                }

                let intended_slot = toy.slot_number.saturating_sub(1);
                let slot_index = if intended_slot < display.capacity && !used_slots[intended_slot] {
                    Some(intended_slot)
                } else {
                    used_slots.iter().position(|used| !*used)
                };

                if let Some(slot_index) = slot_index {
                    used_slots[slot_index] = true;
                    toy.placed_slot_index = Some(slot_index);
                    toy.position =
                        display_slot_position(display, slot_index, data.config.room_width);
                } else {
                    toy.placed_display_id = None;
                    toy.placed_slot_index = None;
                    toy.bench_slot_index = None;
                    toy.bench_id = None;
                    toy.wrong_marker_seconds = 0.0;
                    toy.position = player_position;
                }
            }
        }
    }

    fn pick_up_placed_toy(&mut self, toy_index: usize, data: &GameData) -> InteractionResult {
        let result = self.pick_up_toy(toy_index, data);
        self.refresh_display_completion(data);
        result
    }
}
