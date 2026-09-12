use toybox_after_hours::data::GameData;
use toybox_after_hours::state::{GameSession, InteractionResult};
use toybox_after_hours::tutorial::{TutorialProgress, TutorialStep};

#[test]
fn guidance_advances_from_navigation_to_the_sorting_loop() {
    let data = GameData::load().unwrap();
    let session = GameSession::new(&data);
    let mut tutorial = TutorialProgress::new(true);
    assert_eq!(
        tutorial.hint(&session, &data).unwrap().step,
        TutorialStep::Navigate
    );

    tutorial.observe_navigation(true, true);
    assert_eq!(
        tutorial.hint(&session, &data).unwrap().step,
        TutorialStep::PickUp
    );
    tutorial.observe_interaction(&InteractionResult::PickedUp {
        toy_name: "Test Toy".to_owned(),
    });
    assert_eq!(
        tutorial.hint(&session, &data).unwrap().step,
        TutorialStep::Shelve
    );
}

#[test]
fn contextual_lessons_wait_until_their_mechanic_is_relevant() {
    let data = GameData::load().unwrap();
    let session = GameSession::new(&data);
    let mut tutorial = TutorialProgress::new(true);
    tutorial.mark_sorting_loop_ready();

    assert!(tutorial.hint(&session, &data).is_none());
}

#[test]
fn every_taught_action_closes_the_guide() {
    let mut tutorial = TutorialProgress::new(true);
    tutorial.observe_navigation(true, true);
    tutorial.observe_interaction(&InteractionResult::PickedUp {
        toy_name: "Test Toy".to_owned(),
    });
    tutorial.observe_interaction(&InteractionResult::Placed {
        toy_name: "Test Toy".to_owned(),
        display_name: "Test Display".to_owned(),
        was_wrong: false,
        completed_display: None,
        completed_zone: None,
        available_tools: Vec::new(),
        finished: false,
    });
    tutorial.observe_interaction(&InteractionResult::Repaired {
        toy_name: "Test Toy".to_owned(),
    });
    tutorial.opened_tools();
    tutorial.cycled_trolley(true);

    assert!(tutorial.is_complete());
}
