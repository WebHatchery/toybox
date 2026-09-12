//! Public results returned by the interaction and tool-shop APIs.

use super::RepairPartKind;

#[derive(Debug, Clone)]
pub enum InteractionResult {
    PickedUp {
        toy_name: String,
    },
    Dropped {
        toy_name: String,
    },
    Placed {
        toy_name: String,
        display_name: String,
        was_wrong: bool,
        completed_display: Option<String>,
        /// The aisle this placement finished off, if it finished one. Zones are
        /// the milestone the run is paced around — four displays each, so one
        /// landing is a bigger moment than a single shelf filling.
        completed_zone: Option<String>,
        available_tools: Vec<String>,
        finished: bool,
    },
    PlacementPrevented {
        toy_name: String,
        display_name: String,
        guards_remaining: u32,
    },
    PlacedOnRepairBench {
        toy_name: String,
    },
    Repaired {
        toy_name: String,
    },
    NeedsRepair {
        toy_name: String,
    },
    NeedsRepairParts {
        toy_name: String,
    },
    InventoryFull,
    RepairBenchFull,
    RepairMismatch,
    ShelfFull,
    ShelfSlotUnavailable,
    NothingNearby,
}

#[derive(Debug, Clone)]
pub enum InteractionPreview {
    PlaceOnShelf,
    PlaceOnRepairBench,
    RepairReady {
        toy_name: String,
    },
    RepairBenchFull,
    RepairMismatch,
    /// A part waits on the bench and its counterpart is still out in the store.
    AwaitingRepairMatch {
        toy_name: String,
        missing_part: RepairPartKind,
    },
    NeedsRepair,
    PutDown,
    Pickup {
        toy_name: String,
    },
    InventoryFull,
    ShelfFull,
    LookAtEmptySlot,
    NothingNearby,
    /// The shop was fully restored.
    Finished,
    /// The doors opened with work left. Distinct from `Finished` because
    /// "Shop restored" over a floor still covered in toys is a lie.
    ShiftOver,
}

#[derive(Debug, Clone)]
pub enum ToolPurchaseResult {
    Purchased {
        tool_name: String,
        remaining_credits: usize,
    },
    AlreadyOwned {
        tool_name: String,
    },
    Locked {
        tool_name: String,
        required_displays: usize,
        completed_displays: usize,
    },
    NeedMoreCredits {
        tool_name: String,
        cost: usize,
        available_credits: usize,
    },
    ServicePurchased {
        service_name: &'static str,
        seconds_active: f32,
        remaining_credits: usize,
    },
    ServiceAtCapacity {
        service_name: &'static str,
        seconds_active: f32,
    },
    NoToolsAvailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplaySlotTarget {
    pub display_index: usize,
    pub slot_index: usize,
}
