// A discrete press requests one roll; repeats are already filtered by native_key.
fn combat_roll_key(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    use crate::game::horizon_combat_roll as roll;
    if !pressed || !game.native_on(roll::ID) || !game.native_playing() {
        return;
    }
    if key != game.native_mod_key(roll::ID, roll::KEY, roll::DEFAULT_KEY) {
        return;
    }
    let window = game.native_client_state().map_or(0, |c| c.window_id);
    roll::queue(window);
}
