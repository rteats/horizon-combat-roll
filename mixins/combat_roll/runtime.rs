// SPDX-License-Identifier: GPL-3.0-or-later
// Gameplay and cooldown logic adapted from ZsoltMolnarrr/CombatRoll.
// Kept free of Horizon dependencies so the same source can be unit-tested.
use std::{cell::RefCell, collections::BTreeMap};

pub const ID: &str = "combat_roll";
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str = "Directional combat dodge with charges and recharge.";
pub const KEY: &str = "roll";
pub const DEFAULT_KEY: &str = "key.keyboard.r";

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub cooldown_ticks: u32,
    pub duration_ticks: u32,
    pub extra_distance: f32,
    pub allow_airborne: bool,
    pub show_hud: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            cooldown_ticks: 80, // CombatRoll: 4 seconds at 20 TPS
            duration_ticks: 8,
            extra_distance: 0.0,
            allow_airborne: false,
            show_hud: true,
        }
    }
}
impl Settings {
    pub fn read(mut get: impl FnMut(&str) -> Option<String>) -> Self {
        let mut out = Self::default();
        if let Some(n) = get("cooldown_ticks").and_then(|s| s.trim().parse::<u32>().ok()) {
            out.cooldown_ticks = n.clamp(1, 1200);
        }
        if let Some(n) = get("duration_ticks").and_then(|s| s.trim().parse::<u32>().ok()) {
            out.duration_ticks = n.clamp(1, 40);
        }
        if let Some(n) = get("extra_distance").and_then(|s| s.trim().parse::<f32>().ok()) {
            if n.is_finite() {
                out.extra_distance = n.clamp(-2.0, 12.0);
            }
        }
        out.allow_airborne = get("allow_airborne").is_some_and(|s| s.trim() == "true");
        out.show_hud = get("show_hud").is_none_or(|s| s.trim() != "false");
        out
    }
    pub fn labels(self) -> [String; 3] {
        [
            format!("Roll cooldown: {:.1}s", self.cooldown_ticks as f32 / 20.0),
            format!("Extra roll distance: {:.1}", self.extra_distance),
            format!("Roll while airborne: {}", if self.allow_airborne { "ON" } else { "OFF" }),
        ]
    }
    pub fn click(&mut self, index: usize) {
        match index {
            0 => self.cooldown_ticks = match self.cooldown_ticks { 40 => 80, 80 => 120, _ => 40 },
            1 => self.extra_distance = if self.extra_distance >= 3.0 { 0.0 } else { self.extra_distance + 1.0 },
            2 => self.allow_airborne = !self.allow_airborne,
            _ => {}
        }
    }
    pub fn stored(self) -> [(&'static str, String); 3] {
        [
            ("cooldown_ticks", self.cooldown_ticks.to_string()),
            ("extra_distance", self.extra_distance.to_string()),
            ("allow_airborne", self.allow_airborne.to_string()),
        ]
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Meter {
    pub charges: u32,
    pub maximum: u32,
    pub recharge: u32,
    pub recharge_length: u32,
    pub rolling: bool,
}

#[derive(Clone, Debug)]
struct State {
    pending: bool,
    elapsed: u32,
    charges: u32,
    recharge: u32,
}
impl Default for State {
    fn default() -> Self {
        // CombatRoll starts with zero charges, recharging one after the first cooldown.
        Self { pending: false, elapsed: u32::MAX, charges: 0, recharge: 0 }
    }
}
impl State {
    fn meter(&self, settings: Settings) -> Meter {
        Meter {
            charges: self.charges,
            maximum: 1,
            recharge: self.recharge,
            recharge_length: settings.cooldown_ticks,
            rolling: self.elapsed <= settings.duration_ticks,
        }
    }
    fn tick(&mut self, settings: Settings, eligible: bool) -> bool {
        self.elapsed = self.elapsed.saturating_add(1);
        if self.charges == 0 {
            self.recharge = self.recharge.saturating_add(1);
            if self.recharge >= settings.cooldown_ticks {
                self.charges = 1;
                self.recharge -= settings.cooldown_ticks;
            }
        }
        if self.charges >= 1 {
            self.recharge = 0;
        }
        let requested = std::mem::take(&mut self.pending);
        if requested && eligible && self.charges > 0 && self.elapsed > settings.duration_ticks {
            self.elapsed = 0;
            self.charges -= 1;
            return true;
        }
        false
    }
}
thread_local! {
    // Each window/splitscreen seat gets its own meter and input edge.
    static STATES: RefCell<BTreeMap<u64, State>> = RefCell::new(BTreeMap::new());
    static SETTINGS: RefCell<Option<Settings>> = const { RefCell::new(None) };
}
pub fn settings(mut get: impl FnMut(&str) -> Option<String>) -> Settings {
    SETTINGS.with_borrow_mut(|cached| {
        *cached.get_or_insert_with(|| Settings::read(&mut get))
    })
}
pub fn set_settings(settings: Settings) {
    SETTINGS.with_borrow_mut(|cached| *cached = Some(settings));
}
pub fn queue(window: u64) {
    STATES.with_borrow_mut(|states| states.entry(window).or_default().pending = true);
}
pub fn reset(window: u64) {
    STATES.with_borrow_mut(|states| { states.remove(&window); });
}
pub fn advance(window: u64, config: Settings, eligible: bool) -> bool {
    STATES.with_borrow_mut(|states| states.entry(window).or_default().tick(config, eligible))
}
pub fn meter(window: u64, config: Settings) -> Meter {
    STATES.with_borrow(|states| states.get(&window).cloned().unwrap_or_default().meter(config))
}

/// Horizontal push in blocks/tick, matching CombatRoll's normalized
/// strafe/forward vector, the player's yaw and the 0.475 distance scale.
/// At rest, the upstream mod rolls towards the player's facing direction.
pub fn velocity(strafe: f32, forward: f32, yaw_degrees: f32, extra_distance: f32) -> [f32; 3] {
    if ![strafe, forward, yaw_degrees, extra_distance].iter().all(|x| x.is_finite()) {
        return [0.0; 3];
    }
    let (x, z) = if strafe.abs() + forward.abs() < f32::EPSILON {
        (0.0, 1.0)
    } else {
        let len = strafe.hypot(forward);
        (strafe / len, forward / len)
    };
    let r = (-yaw_degrees).to_radians();
    let distance = 0.475 * (3.0 + extra_distance).clamp(0.0, 15.0);
    [(x * r.cos() + z * r.sin()) * distance, 0.0, (z * r.cos() - x * r.sin()) * distance]
}

#[cfg(test)]
mod tests {
    use super::*;
    fn cfg() -> Settings { Settings::default() }
    #[test]
    fn settings_default_and_bounds() {
        assert_eq!(Settings::read(|_| None), cfg());
        let s = Settings::read(|name| match name {
            "cooldown_ticks" => Some("0".into()),
            "extra_distance" => Some("NaN".into()),
            "allow_airborne" => Some("true".into()),
            _ => None
        });
        assert_eq!(s.cooldown_ticks, 1);
        assert_eq!(s.extra_distance, 0.0);
        assert!(s.allow_airborne);
    }
    #[test]
    fn recharge_then_one_roll_and_recharge() {
        let mut s = State::default();
        for _ in 0..79 { assert!(!s.tick(cfg(), true)); }
        assert_eq!(s.meter(cfg()).charges, 0);
        assert!(!s.tick(cfg(), true));
        assert_eq!(s.meter(cfg()).charges, 1);
        s.pending = true;
        assert!(s.tick(cfg(), true));
        assert_eq!(s.meter(cfg()).charges, 0);
        assert!(s.meter(cfg()).rolling);
        s.pending = true;
        assert!(!s.tick(cfg(), true), "no double-roll while active");
        for _ in 0..78 { s.tick(cfg(), true); }
        assert_eq!(s.meter(cfg()).charges, 1);
    }
    #[test]
    fn rejected_request_is_not_buffered() {
        let mut s = State::default();
        for _ in 0..80 { s.tick(cfg(), true); }
        s.pending = true;
        assert!(!s.tick(cfg(), false));
        assert!(!s.tick(cfg(), true));
        assert_eq!(s.meter(cfg()).charges, 1);
    }
    #[test]
    fn directions_and_yaw() {
        let v = velocity(0.0, 0.0, 0.0, 0.0);
        assert!((v[2] - 1.425).abs() < 1e-5);
        let v = velocity(1.0, 0.0, 0.0, 0.0);
        assert!((v[0] - 1.425).abs() < 1e-5);
        let v = velocity(0.0, 1.0, 90.0, 0.0);
        assert!((v[0] + 1.425).abs() < 1e-5);
        let v = velocity(1.0, 1.0, 0.0, 0.0);
        assert!((v[0].hypot(v[2]) - 1.425).abs() < 1e-5);
        assert_eq!(velocity(0.0, 1.0, f32::NAN, 0.0), [0.0; 3]);
    }
    #[test]
    fn window_isolation_and_reset() {
        let a=12345;
        let b=67890;
        queue(a);
        assert!(!advance(a, cfg(), true));
        assert!(!advance(b, cfg(), true));
        reset(a);
        assert_eq!(meter(a, cfg()).charges, 0);
    }
}
