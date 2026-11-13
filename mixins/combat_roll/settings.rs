fn combat_roll_settings(game: &mut crate::game::Game<'_>, mod_id: &str, buttons: &mut Vec<String>) {
    use crate::game::horizon_combat_roll as roll;
    if mod_id != roll::ID { return; }
    let settings = roll::settings(|key| game.native_setting(roll::ID, key));
    buttons.extend(settings.labels());
}
