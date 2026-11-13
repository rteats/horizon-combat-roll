fn combat_roll_list(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::horizon_combat_roll as roll;
    mods.push(crate::game::native_hooks::NativeMod {
        id: roll::ID.into(),
        title: format!("Combat Roll {}", roll::VERSION),
        description: roll::DESCRIPTION.into(),
        keys: vec![[roll::KEY.into(), "Dodge / roll".into(), roll::DEFAULT_KEY.into()]],
    });
}
