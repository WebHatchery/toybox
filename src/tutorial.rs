//! Contextual first-shift guidance. It teaches one action at a time and waits
//! for repair and trolley advice until those mechanics are actually relevant.

use crate::data::{GameData, TutorialStepCopy};
use crate::state::{GameSession, InteractionResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TutorialStep {
    Navigate,
    PickUp,
    Shelve,
    Repair,
    Tools,
    Trolley,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TutorialHint {
    pub step: TutorialStep,
    pub eyebrow: String,
    pub title: String,
    pub body: String,
    pub keys: Vec<String>,
}

impl TutorialHint {
    fn from_copy(step: TutorialStep, copy: &TutorialStepCopy) -> Self {
        Self {
            step,
            eyebrow: copy.eyebrow.clone(),
            title: copy.title.clone(),
            body: copy.body.clone(),
            keys: copy.keys.clone(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TutorialProgress {
    active: bool,
    moved: bool,
    looked: bool,
    picked_up: bool,
    shelved_correctly: bool,
    repaired: bool,
    opened_tools: bool,
    cycled_trolley: bool,
}

impl TutorialProgress {
    pub fn new(active: bool) -> Self {
        Self {
            active,
            ..Default::default()
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn observe_navigation(&mut self, moved: bool, looked: bool) {
        if !self.active {
            return;
        }
        self.moved |= moved;
        self.looked |= looked;
    }

    pub fn observe_interaction(&mut self, result: &InteractionResult) {
        if !self.active {
            return;
        }
        match result {
            InteractionResult::PickedUp { .. } => self.picked_up = true,
            InteractionResult::Placed {
                was_wrong: false, ..
            } => self.shelved_correctly = true,
            InteractionResult::Repaired { .. } => self.repaired = true,
            _ => {}
        }
    }

    pub fn opened_tools(&mut self) {
        if self.active {
            self.opened_tools = true;
        }
    }

    pub fn cycled_trolley(&mut self, had_multiple_toys: bool) {
        if self.active && had_multiple_toys {
            self.cycled_trolley = true;
        }
    }

    /// Mark the basic sorting loop as demonstrated by a scripted guide or a
    /// tutorial preview without pretending that a live interaction happened.
    pub fn mark_sorting_loop_ready(&mut self) {
        self.moved = true;
        self.looked = true;
        self.picked_up = true;
        self.shelved_correctly = true;
    }

    pub fn skip(&mut self) {
        self.active = false;
    }

    pub fn is_complete(&self) -> bool {
        self.moved
            && self.looked
            && self.picked_up
            && self.shelved_correctly
            && self.repaired
            && self.opened_tools
            && self.cycled_trolley
    }

    pub fn hint(&self, session: &GameSession, data: &GameData) -> Option<TutorialHint> {
        if !self.active || self.is_complete() {
            return None;
        }
        if !self.moved || !self.looked {
            return Some(TutorialHint::from_copy(
                TutorialStep::Navigate,
                &data.ui_copy.tutorial[0],
            ));
        }
        if !self.picked_up {
            return Some(TutorialHint::from_copy(
                TutorialStep::PickUp,
                &data.ui_copy.tutorial[1],
            ));
        }
        if !self.shelved_correctly {
            return Some(TutorialHint::from_copy(
                TutorialStep::Shelve,
                &data.ui_copy.tutorial[2],
            ));
        }

        let carrying_part = session.active_toy().is_some_and(|toy| toy.is_repair_part());
        if !self.repaired && carrying_part {
            return Some(TutorialHint::from_copy(
                TutorialStep::Repair,
                &data.ui_copy.tutorial[3],
            ));
        }
        if !self.opened_tools && session.next_available_upgrade(data).is_some() {
            return Some(TutorialHint::from_copy(
                TutorialStep::Tools,
                &data.ui_copy.tutorial[4],
            ));
        }
        if !self.cycled_trolley
            && session.has_upgrade("sorting_trolley")
            && session.player.carried_toy_ids.len() > 1
        {
            return Some(TutorialHint::from_copy(
                TutorialStep::Trolley,
                &data.ui_copy.tutorial[5],
            ));
        }
        None
    }
}
