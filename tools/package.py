#!/usr/bin/env python3
"""Deterministically package this Horizon native source-mixin bundle (.hnmod)."""
from __future__ import annotations

import argparse
import io
import json
import pathlib
import re
import tomllib
import zipfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
BUNDLE = ROOT / "mixins" / "combat_roll"
COMPAT = ROOT / "compatibility.json"
HEX40 = re.compile(r"[0-9a-f]{40}\Z")
MOD_ID = "combat_roll"


def checked_metadata() -> tuple[dict, dict]:
    config = json.loads(COMPAT.read_text(encoding="utf-8"))
    manifest = tomllib.loads((BUNDLE / "horizon-mixin.toml").read_text(encoding="utf-8"))
    for key in ("horizon_revision", "horizon_mixins_revision"):
        if not isinstance(config.get(key), str) or not HEX40.fullmatch(config[key]):
            raise ValueError(f"{key} must be a full lowercase 40-character commit")
    if manifest["mod"]["id"] != MOD_ID:
        raise ValueError("mod id mismatch")
    if manifest["mod"]["version"] != "0.1.0":
        raise ValueError("mod version and package metadata disagree")
    if manifest["compatibility"]["horizon_revision"] != config["horizon_revision"]:
        raise ValueError("Horizon revision differs between manifest and compatibility.json")
    if not isinstance(config.get("hooks"), int) or config["hooks"] < 360:
        raise ValueError("movement input requires native hooks API >= 360")
    if not re.fullmatch(r"\d+(?:\.\d+)+", config["game"]):
        raise ValueError("invalid Minecraft game version")
    sources = {p.name for p in BUNDLE.glob("*.rs")}
    declared = {m["source"] for m in manifest["module"]}
    declared.update(i["helper"] for i in manifest["injection"])
    if not declared <= sources:
        raise ValueError(f"unresolved sources: {sorted(declared - sources)}")
    if not declared == sources:
        raise ValueError(f"unreferenced Rust sources: {sorted(sources - declared)}")
    if any(i["kind"] != "HEAD" for i in manifest["injection"]):
        raise ValueError("only stable HEAD hooks are expected")
    return config, manifest


def package_bytes() -> bytes:
    config, manifest = checked_metadata()
    native = {
        "schema": 1,
        "id": MOD_ID,
        "version": manifest["mod"]["version"],
        "game": config["game"],
        "horizon": config["horizon_revision"],
        "mixins": config["horizon_mixins_revision"],
        "hooks": config["hooks"],
    }
    data = {"horizon-native.json": (json.dumps(native, sort_keys=True, indent=2) + "\n").encode()}
    # Every redistributed source bundle must carry its GPLv3 license and
    # identify the original mod author, not just link to this repository.
    data[f"mixins/{MOD_ID}/LICENSE"] = (ROOT / "LICENSE").read_bytes()
    data[f"mixins/{MOD_ID}/NOTICE"] = (
        "Horizon Combat Roll\n"
        "Gameplay based on CombatRoll by Zsolt Molnar:\n"
        "https://github.com/ZsoltMolnarrr/CombatRoll\n"
        "Original license: GNU General Public License version 3.\n"
        "Rust adaptation: GPL-3.0-only.\n"
    ).encode("utf-8")
    for path in sorted(BUNDLE.iterdir()):
        if path.is_symlink() or not path.is_file():
            raise ValueError(f"unexpected non-file in bundle: {path}")
        if path.suffix not in (".rs", ".toml"):
            raise ValueError(f"disallowed file type in native bundle: {path}")
        data[f"mixins/{MOD_ID}/{path.name}"] = path.read_bytes()
    out = io.BytesIO()
    with zipfile.ZipFile(out, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as zip_file:
        for name, contents in sorted(data.items()):
            entry = zipfile.ZipInfo(name, (1980, 1, 1, 0, 0, 0))
            entry.compress_type = zipfile.ZIP_DEFLATED
            entry.external_attr = 0o100644 << 16
            zip_file.writestr(entry, contents, compress_type=zipfile.ZIP_DEFLATED, compresslevel=9)
    return out.getvalue()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="verify inputs and deterministic output")
    args = parser.parse_args()
    data = package_bytes()
    with zipfile.ZipFile(io.BytesIO(data)) as z:
        assert z.testzip() is None
        assert json.loads(z.read("horizon-native.json"))["id"] == MOD_ID
        assert "mixins/combat_roll/LICENSE" in z.namelist()
        assert "mixins/combat_roll/NOTICE" in z.namelist()
    if args.check:
        if data != package_bytes():
            raise ValueError("the package is not reproducible")
        print("Validated reproducible native bundle")
        return
    config, manifest = checked_metadata()
    target = ROOT / "dist"
    target.mkdir(parents=True, exist_ok=True)
    filename = f"{MOD_ID}-{manifest['mod']['version']}-{config['horizon_revision'][:12]}.hnmod"
    (target / filename).write_bytes(data)
    print(target / filename)


if __name__ == "__main__":
    main()
