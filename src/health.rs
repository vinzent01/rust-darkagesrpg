pub const MAX_HEALTH: i32 = 100;
pub const MELEE_DAMAGE: i32 = 20;

pub struct Health {
    pub current: i32,
    pub maximum: i32,
}

impl Health {
    pub fn new(maximum: i32) -> Self {
        let maximum = maximum.max(0);
        Self {
            current: maximum,
            maximum,
        }
    }

    pub fn take_damage(&mut self, amount: i32) -> bool {
        self.current = (self.current - amount.max(0)).clamp(0, self.maximum);
        self.current == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn damage_reduces_health_and_clamps_at_zero() {
        let mut health = Health::new(MAX_HEALTH);

        assert!(!health.take_damage(MELEE_DAMAGE));
        assert_eq!(health.current, 80);
        assert!(health.take_damage(MAX_HEALTH));
        assert_eq!(health.current, 0);
    }
}
