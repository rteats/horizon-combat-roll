fn combat_roll_settings_click(game: &mut crate::game::Game<'_>, mod_id: &str, button: usize) {
    use crate::game::horizon_combat_roll as roll;
    if mod_id != roll::ID { return; }
    let mut settings = roll::settings(|key| game.native_setting(roll::ID, key));
    settings.click(button);
    roll::set_settings(settings);
    for (key, value) in settings.stored() {
        game.native_set_setting(roll::ID, key, &value);
    }
}
