# Combat Roll for Horizon (Rust)

A native Rust adaptation of [ZsoltMolnarrr/CombatRoll](https://github.com/ZsoltMolnarrr/CombatRoll) for [Horizon](https://github.com/horizon-rust/horizon). Requires the **native mod** compilation path (not Fabric, NeoForge or a standalone `.hmod`).

## Implementation status

**Early, client-side playable prototype; not feature-equivalent with upstream CombatRoll.**

Implemented: directional roll on a remappable **R** key, client-side horizontal push, 8-tick roll state, one regenerating charge (4 seconds at 20 ticks/s), hunger and on-ground restrictions, disallow roll when swimming/riding/using an item or with weapon cooldown, jump suppression, basic water slowdown, food exhaustion, text HUD and native mod settings. While stationary, roll follows player facing as in the original.

Not yet implemented: server-side roll authorization and invulnerability window; visual/sound effects; animation and synchronized rolls for other players; custom attributes/enchantments; fluid-depth/slippery-surface scaling; blocking attacks and item use throughout the roll. Multiplayer servers can rubber-band the client-side impulse. **Do not interpret the current port as granting invulnerability**. The original default invulnerability setting is zero.

## Install / build

Horizon native mods compile their Rust code into a separate Horizon build. Source lives in `mixins/combat_roll/`; the launcher must trust native code. Install a generated **.hnmod** bundle using Horizon's native-mod flow, or build its mixins locally with Horizon and horizon-mixins at the revisions in `compatibility.json`.

```sh
cargo test
python3 tools/package.py
# outputs dist/combat_roll-0.1.0-<Horizon SHA>.hnmod
```

For manual native builds see [Horizon's native mod documentation](https://github.com/horizon-rust/horizon/blob/main/docs/native-mods.md). The launcher builds the native bundle against the pinned Horizon revision; the `.hnmod` is source, **not** a precompiled Minecraft executable.

## CI and upstream synchronization

Pull requests and pushes run Rust tests, formatting/lints, package checks and the full Horizon source-mixin compatibility build. Successful main-branch CI publishes a versioned `.hnmod` GitHub Release. A scheduled workflow checks Horizon's **nightly** release and its mixin compiler revision, opens a compatibility-update PR, and relies on the same CI before it can be merged. It intentionally does not silently promote an untested nightly.

## Licensing and attribution

Original [CombatRoll](https://github.com/ZsoltMolnarrr/CombatRoll) by Zsolt Molnar, licensed under GNU GPL v3. This Rust derivative is distributed under **GPL-3.0-only**; original game mod sources and its artwork/animations are not embedded. Horizon itself and horizon-mixins are separate upstream projects.
