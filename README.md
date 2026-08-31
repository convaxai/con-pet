# Con Pet

[English](README.md) | [简体中文](README.zh-CN.md)

Con Pet is an always-available desktop companion for Windows and macOS. When you type a configured keyword in any application, Con Pet plays a transparent, click-through pet animation on your screen.

## Features

- Observes only newly typed keys. The default keyword is `codex`, and English matching is case-insensitive.
- Does not read existing input-field content, use OCR, access the clipboard, or persist keystrokes and matching buffers.
- Keeps at most one keyword-length buffer. Backspace removes one character; shortcuts, spaces, Enter, and other control keys interrupt the sequence; stale input expires after the configured timeout.
- Reliably supports physical English A–Z keys on Windows and macOS. Unicode text can also match when the platform hook supplies committed text, but most Chinese IMEs expose only their physical Pinyin keys to a global hook.
- Bundles 100 popular Codex Pets for offline use, excluding Leaders and known political-person content, and also discovers `~/.codex/pets/*/pet.json` automatically.
- Includes a visual pet gallery with search, animated hover previews, and direct import of a standard Pet ZIP.
- Randomly selects among the nine standard Codex Pet animation states on each trigger, avoiding immediate repeats.
- Includes a larger `Spider-Man 4 · Wire Swing` animation that swings from the top center of the screen.
- Supports 0.4–2.5× scaling, configurable loops, multi-monitor targeting, corners, top-center, random placement, and fixed coordinates.
- Uses a transparent, always-on-top, click-through overlay that never steals typing focus.
- Keeps running in the system tray after the settings window closes and offers configurable launch at login.
- Provides separate English and Simplified Chinese interfaces.

## Roadmap / TODO

- [ ] Add local-only typing activity statistics, such as aggregate keystroke counts and daily trends. Store counters only—never typed content, individual keys, or key sequences—and never upload the data.
- [ ] Explore persistent Pet interaction and progression, such as petting, feeding, playing, mood, and companionship levels.
- [ ] Add idle progression so a Pet can grow over time, gain experience, and unlock new states or animations while accompanying the user.

## Pet Package Format

Con Pet is directly compatible with the standard Codex Pet package:

```text
my-pet/
├── pet.json
└── spritesheet.webp
```

Example `pet.json`:

```json
{
  "id": "my-pet",
  "displayName": "My Pet",
  "description": "A tiny companion.",
  "spritesheetPath": "spritesheet.webp"
}
```

The standard atlas is `1536×1872`: 8 columns × 9 rows, with each frame measuring `192×208`. Rows represent idle, running right, running left, waving, jumping, failed, waiting, running, and review. Con Pet also supports the current 8×10 and 8×11 extended Codex Pet atlases without vertically squeezing them. `Spider-Man 4 · Wire Swing` stores 20 consecutive frames across the first three rows.

## Local Development

Requirements: Node.js 20+, pnpm 10+, stable Rust, and the platform dependencies required by Tauri 2.

```bash
pnpm install
pnpm test
pnpm build
cargo test --manifest-path src-tauri/Cargo.toml
pnpm tauri dev
```

Build an installer:

```bash
pnpm tauri build
```

On first launch, macOS users must allow Con Pet under **System Settings → Privacy & Security → Input Monitoring**, then restart the app. Screen Recording permission is not required.

macOS binds Input Monitoring approval to the application's code-signing designated requirement. Ad-hoc development builds bind that requirement to the current CDHash: normal quit/relaunch keeps permission, but rebuilding and replacing the `.app` requires approval again. Stable releases must use the same Apple Development or Developer ID Application certificate. You can inspect local identities with `security find-identity -v -p codesigning`. This behavior comes from macOS TCC rather than Con Pet resetting permission.

The bundled 100-Pet catalog is roughly 170 MiB, and the arm64 macOS `.app` is roughly 186 MiB. Assets are read directly from the application bundle and are not copied into the user directory on first launch.

## Implementation

- [Tauri 2](https://github.com/tauri-apps/tauri) provides cross-platform windows, tray integration, events, and packaging.
- [keytap](https://github.com/jamiepine/keytap) provides the observe-only global keyboard event stream.
- [tauri-plugin-macos-permissions](https://github.com/ayangweb/tauri-plugin-macos-permissions) checks and opens the macOS Input Monitoring permission flow.
- React, Tailwind CSS, and Motion power the settings UI and animation components.

See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for detailed dependency and licensing notes.

## Special Thanks

Con Pet would not be what it is without the open-source work and ideas shared by these projects:

- [BongoCat](https://github.com/ayangweb/BongoCat) — the primary architectural reference for a cross-platform Tauri desktop pet, background lifecycle, and macOS permission experience.
- [YAKC](https://github.com/iammodev/YAKC) — a valuable reference for small transparent, always-on-top, click-through desktop overlays.
- [beUI](https://github.com/starc007/ui-components) — the source and design inspiration for the Motion-based controls adapted in the settings window.
- [keytap](https://github.com/jamiepine/keytap) — a focused, observe-only global keyboard listener that made the native input path smaller and more reliable.
- [tauri-plugin-macos-permissions](https://github.com/ayangweb/tauri-plugin-macos-permissions) — the permission bridge used to provide a clear macOS Input Monitoring setup flow.
- [Codex Pets](https://codex-pets.net/) and its community creators — for the Pet package format, animation-state conventions, and the ecosystem that made an offline gallery possible.

Thank you to every maintainer and contributor behind these projects.

## Verification and Packaging

CI runs the frontend tests, TypeScript/Vite production build, Rust formatting check, Clippy, and Rust tests on macOS and Windows. The macOS input listener and permission lifecycle still require manual verification on a physical Mac with Input Monitoring permission.

After CI succeeds on `main`, the `Package installers` workflow builds a [macOS Universal DMG](https://github.com/convaxai/con-pet/releases/download/nightly/Con-Pet-macOS-universal.dmg) and [Windows x64 NSIS installer](https://github.com/convaxai/con-pet/releases/download/nightly/Con-Pet-Windows-x64.exe), then updates the rolling [Nightly Release](https://github.com/convaxai/con-pet/releases/tag/nightly) and its [SHA-256 checksums](https://github.com/convaxai/con-pet/releases/download/nightly/SHA256SUMS.txt). These stable download links do not expire automatically. Commit-specific Actions artifacts are also retained for 30 days for debugging, and the workflow can be started manually without publishing a release.

## License

The source code is licensed under the MIT License. Bundled or user-selected Codex Pet artwork remains the property of its respective rights holders.
