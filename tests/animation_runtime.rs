use std::time::Duration;

use rgui::runtime::{AnimationClock, AnimationId};

#[test]
fn animation_clock_advances_values_by_elapsed_time() {
    let mut clock = AnimationClock::default();
    let id = clock.start(0.0, 1.0, Duration::from_millis(100));
    assert_eq!(clock.value(id), Some(0.0));
    clock.advance(Duration::from_millis(50));
    assert_eq!(clock.value(id), Some(0.5));
    clock.advance(Duration::from_millis(50));
    assert_eq!(clock.value(id), Some(1.0));
    assert!(clock.is_finished(id));
}
