// Client-side authoritative movement for the local window's player.
fn combat_roll_tick(game: &mut crate::game::Game<'_>) {
    use crate::game::horizon_combat_roll as roll;
    let window = game.native_client_state().map_or(0, |c| c.window_id);
    if !game.native_on(roll::ID) || !game.native_playing() {
        roll::reset(window);
        return;
    }
    let settings = roll::settings(|key| game.native_setting(roll::ID, key));
    let Some(p) = game.player.as_ref() else { roll::reset(window); return; };
    let eligible = !p.spectator
        && !p.health.dead_or_dying()
        && !p.is_passenger
        && !p.swimming
        && p.item_use_speed.is_none()
        && p.health.food.food > 6
        && (settings.allow_airborne || p.ground.on_ground)
        && game.hand_strength.scale(0.0, game.main_hand_attack_speed()) >= 0.95;
    let (strafe, forward) = (p.applied_input[0], p.applied_input[1]);
    let wet = p.fluids.was_touching_water;
    if !roll::advance(window, settings, eligible) { return; }
    let mut kick = roll::velocity(strafe, forward, game.yaw, settings.extra_distance);
    // The original reduces the distance in water; precise fluid-height and
    // slippery-ground attenuation still need their native equivalents.
    if wet { kick[0] *= 0.3; kick[2] *= 0.3; }
    game.native_push_player(kick);
    if let Some(p) = game.player.as_mut() {
        p.cause_food_exhaustion(0.1);
    }
}
