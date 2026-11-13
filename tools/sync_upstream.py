#!/usr/bin/env python3
"""Open-PR preparation: follow the published Horizon nightly and mixin revision."""
from __future__ import annotations

import json
import os
from pathlib import Path
import re
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
CONFIG = ROOT / "compatibility.json"
MANIFEST = ROOT / "mixins/combat_roll/horizon-mixin.toml"
SHA40 = re.compile(r"[0-9a-f]{40}\Z")


def github(path: str) -> dict:
    req = urllib.request.Request(
        "https://api.github.com/" + path,
        headers={
            "Accept": "application/vnd.github+json",
            "User-Agent": "horizon-combat-roll-sync",
            **({"Authorization": "Bearer " + os.environ["GITHUB_TOKEN"]}
               if os.environ.get("GITHUB_TOKEN") else {}),
        },
    )
    with urllib.request.urlopen(req, timeout=30) as response:
        return json.load(response)


def raw(repo: str, revision: str, path: str) -> str:
    req = urllib.request.Request(
        f"https://raw.githubusercontent.com/{repo}/{revision}/{path}",
        headers={"User-Agent": "horizon-combat-roll-sync"},
    )
    with urllib.request.urlopen(req, timeout=30) as response:
        return response.read().decode("utf-8")


def main() -> None:
    config = json.loads(CONFIG.read_text(encoding="utf-8"))
    release = github("repos/horizon-rust/horizon/releases/tags/nightly")
    rev = release["target_commitish"]
    if not isinstance(rev, str) or not SHA40.fullmatch(rev):
        raise RuntimeError("nightly release is not pinned to a full commit")
    if rev == config["horizon_revision"]:
        print("Already synchronized with Horizon nightly", rev)
        return

    # Do not silently generate a revision incompatible with our hook adapters.
    hooks = raw("horizon-rust/horizon", rev,
                "crates/horizon-client/src/game/native_hooks.rs")
    for hook in (
        "fn native_key(", "fn native_client_tick_post(",
        "fn native_movement_input(", "fn native_mods(",
        "fn native_hud(", "fn native_settings_click(",
    ):
        if hook not in hooks:
            raise RuntimeError(f"required Horizon hook missing: {hook}")
    readme = raw("horizon-rust/horizon", rev, "README.md")
    match = re.search(r"Minecraft: Java Edition (\d+(?:\.\d+)+)", readme)
    if not match:
        raise RuntimeError("could not identify release game version")

    mixins = github("repos/horizon-rust/horizon/commits/" + rev)
    if mixins.get("sha") != rev:
        raise RuntimeError("the published Horizon revision is unavailable")
    mixins_rev = github("repos/horizon-rust/horizon-mixins/commits/main")["sha"]
    if not SHA40.fullmatch(mixins_rev):
        raise RuntimeError("invalid mixins commit")

    config["horizon_revision"] = rev
    config["horizon_mixins_revision"] = mixins_rev
    config["game"] = match.group(1)
    text = MANIFEST.read_text(encoding="utf-8")
    text, count = re.subn(
        r'(?m)^horizon_revision = "[0-9a-f]{40}"$',
        f'horizon_revision = "{rev}"', text,
    )
    if count != 1:
        raise RuntimeError("manifest compatibility revision not unique")
    # Check our build tooling and packaging on the branch before opening a PR.
    MANIFEST.write_text(text, encoding="utf-8")
    CONFIG.write_text(json.dumps(config, indent=2) + "\n", encoding="utf-8")
    print(f"Proposed Horizon nightly {rev}; mixins {mixins_rev}; game {config['game']}")


if __name__ == "__main__":
    main()
