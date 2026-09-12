use std::collections::HashSet;
use toybox_after_hours::data::ToyCategory;
use toybox_after_hours::toys::library::toy_profile;
use toybox_after_hours::toys::part_accents::{accent_for, PartAccent};

/// Board games and building blocks have no face, so their renderers never
/// call `draw_muzzle` or use `eye_spread`. Two accents differing only in
/// those fields would be indistinguishable on a game lid, so the comparison
/// has to drop them.
fn draws_a_face(category: ToyCategory) -> bool {
    !matches!(
        category,
        ToyCategory::BoardGames | ToyCategory::BuildingBlocks
    )
}

/// Everything about an accent that reaches the screen for this category,
/// as bits so floats can go in a set.
fn visible_signature(category: ToyCategory, accent: PartAccent) -> (u8, u32, u32, u32) {
    let crest = accent.crest as u8;
    let (muzzle, eyes) = if draws_a_face(category) {
        (accent.muzzle.to_bits(), accent.eye_spread.to_bits())
    } else {
        (0, 0)
    };
    (crest, accent.accent_scale.to_bits(), muzzle, eyes)
}

/// The ten identities of a category all share one pair of renderers, so if
/// two of them carry the same accent their broken halves are the same
/// object in two colours — precisely the problem the table exists to fix.
/// A new identity copy-pasted from its neighbour fails here.
#[test]
fn no_two_identities_in_a_category_break_into_the_same_part() {
    for category in [
        ToyCategory::Plushies,
        ToyCategory::TinyDragons,
        ToyCategory::ActionFigures,
        ToyCategory::BoardGames,
        ToyCategory::BuildingBlocks,
    ] {
        let mut seen: HashSet<(u8, u32, u32, u32)> = HashSet::new();
        for slot_number in 1..=10 {
            let profile = toy_profile(category, slot_number);
            let signature = visible_signature(category, accent_for(profile.identity));
            assert!(
                seen.insert(signature),
                "{:?} in {category:?} breaks into a part identical to an earlier \
                 identity's: crest {:?}, scale {}",
                profile.identity,
                accent_for(profile.identity).crest,
                accent_for(profile.identity).accent_scale
            );
        }
        assert_eq!(seen.len(), 10);
    }
}

/// Crest sizes stay in a band the renderers were drawn against. A scale of
/// 4.0 would not error anywhere — it would quietly put a Rabbit's ears
/// through the ceiling.
#[test]
fn accent_scales_stay_within_the_drawn_range() {
    for slot_number in 1..=10 {
        for category in [
            ToyCategory::Plushies,
            ToyCategory::TinyDragons,
            ToyCategory::ActionFigures,
            ToyCategory::BoardGames,
            ToyCategory::BuildingBlocks,
        ] {
            let identity = toy_profile(category, slot_number).identity;
            let accent = accent_for(identity);
            assert!(
                (0.5..=2.0).contains(&accent.accent_scale),
                "{identity:?} crest scale {} is outside the range the crest \
                 shapes were sized for",
                accent.accent_scale
            );
            assert!((0.0..=1.0).contains(&accent.muzzle));
            assert!((0.5..=2.0).contains(&accent.eye_spread));
        }
    }
}
