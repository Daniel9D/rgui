use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AnimationId(u64);

#[derive(Clone, Debug)]
struct Animation {
    from: f32,
    to: f32,
    duration: Duration,
    elapsed: Duration,
}

#[derive(Default)]
pub struct AnimationClock {
    next_id: u64,
    animations: std::collections::HashMap<AnimationId, Animation>,
}

impl AnimationClock {
    pub fn start(&mut self, from: f32, to: f32, duration: Duration) -> AnimationId {
        self.next_id += 1;
        let id = AnimationId(self.next_id);
        self.animations.insert(
            id,
            Animation {
                from,
                to,
                duration,
                elapsed: Duration::ZERO,
            },
        );
        id
    }

    pub fn advance(&mut self, delta: Duration) {
        for animation in self.animations.values_mut() {
            animation.elapsed = (animation.elapsed + delta).min(animation.duration);
        }
    }

    pub fn value(&self, id: AnimationId) -> Option<f32> {
        let animation = self.animations.get(&id)?;
        let t = if animation.duration.is_zero() {
            1.0
        } else {
            (animation.elapsed.as_secs_f32() / animation.duration.as_secs_f32()).clamp(0.0, 1.0)
        };
        Some(animation.from + (animation.to - animation.from) * t)
    }

    pub fn is_finished(&self, id: AnimationId) -> bool {
        self.animations
            .get(&id)
            .is_some_and(|animation| animation.elapsed >= animation.duration)
    }
}
