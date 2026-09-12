use toybox_after_hours::ui::scene3d::toy_draw_scale;

#[test]
fn two_halves_on_a_bench_are_drawn_the_same_size() {
    // The pair this was found on: body at toy index 0, head at 195.
    assert_eq!(toy_draw_scale(0, true), toy_draw_scale(195, true));
    assert_ne!(
        toy_draw_scale(0, false),
        toy_draw_scale(195, false),
        "the loose-toy jitter is what made them differ; keep it"
    );

    // No index escapes the rule, and the floor keeps its variety.
    let benched: Vec<f32> = (0..500).map(|index| toy_draw_scale(index, true)).collect();
    assert!(benched.iter().all(|scale| *scale == benched[0]));

    let loose: Vec<f32> = (0..500).map(|index| toy_draw_scale(index, false)).collect();
    let low = loose.iter().copied().fold(f32::MAX, f32::min);
    let high = loose.iter().copied().fold(f32::MIN, f32::max);
    assert!(low < high, "loose toys lost their size variety");
    assert!(
        (0.85..=1.10).contains(&low) && (0.85..=1.10).contains(&high),
        "loose scales drifted out of range: {low}..{high}"
    );
}
