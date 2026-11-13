// A small textual charge/cooldown meter. No original copyrighted UI assets.
fn combat_roll_hud(
    game: &mut crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    corners: &mut horizon_client::extras_hud::Corners,
) {
    use crate::game::horizon_combat_roll as roll;
    if !game.native_on(roll::ID) || !game.native_playing() { return; }
    let settings = roll::settings(|key| game.native_setting(roll::ID, key));
    if !settings.show_hud { return; }
    let window = game.native_client_state().map_or(0, |c| c.window_id);
    let m = roll::meter(window, settings);
    let text = if m.charges > 0 {
        "Roll: READY".to_string()
    } else {
        format!("Roll: {:.1}s", (m.recharge_length - m.recharge) as f32 / 20.0)
    };
    let width = game.native_text_width(&text) + 6.0;
    let (x, y) = corners.place(horizon_client::extras::Corner::BottomRight, width, 12.0);
    game.native_text(batch, &text, [x + 3.0, y + 1.0], 0xFFFFFF, true);
}
