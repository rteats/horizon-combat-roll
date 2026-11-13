// Disable jump during the 8-tick roll window.
fn combat_roll_input(game: &crate::game::Game<'_>, jump: &mut bool) {
    use crate::game::horizon_combat_roll as roll;
    if !game.native_on(roll::ID) { return; }
    let window = game.native_client_state().map_or(0, |c| c.window_id);
    let settings = roll::settings(|key| game.native_setting(roll::ID, key));
    if roll::meter(window, settings).rolling {
        *jump = false;
    }
}
