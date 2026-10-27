# REA findings

> REA = "Reverse Engineer Anything", https://github.com/morluto/rea (commit `17f581ff75d2b6cf643a997275e3e296f6c95760`, npm `rea-agents@6.7.0`).
> REA is a research tool only. It is not a LimeShot dependency and is not bundled.

## Installation and verification (2026-10-20)

| Step | Command | Result |
|---|---|---|
| Read official docs | `README.md`, `docs/installation.md` in the cloned repo | Setup entry point is `npx rea-agents setup`; `--client claude_code`, `--dry-run`, `--yes` are supported. |
| Read-only plan | `npx -y rea-agents@latest setup --client claude_code --dry-run --json` | Planned two actions: add `mcpServers.rea` to `~/.claude.json` (backup `~/.claude.json.rea.backup`) and install the skill to `~/.claude/skills/reverse-engineer-anything`. |
| Apply | `npx -y rea-agents@6.7.0 setup --client claude_code --yes --json` | Status `ready`. Verified: `~/.claude.json` contains `"rea": {"command": "npx", "args": ["-y", "rea-agents@6.7.0", "mcp"]}`; `SKILL.md` (skill version 42, 207 tools) is present. |
| MCP in the current session | — | **Not available in the session that ran setup.** MCP servers load at Claude Code start. The REA CLI exposes the same operations and was used instead. |

### Continuation steps for MCP use

1. Restart Claude Code (quit and reopen the desktop app or `claude` CLI).
2. Confirm that tools prefixed `mcp__rea__` are listed. If not, run `npx -y rea-agents@6.7.0 doctor --client claude_code --skill --json`.
3. For screenshots and accessibility trees, grant **Screen Recording** (and Accessibility) to the host process that runs REA, in System Settings › Privacy & Security. Only the user can do this.
4. Re-run the `observe-native-ui` step below and add the results to `fl-studio-analysis.md`.

### Analysis providers

`rea doctor` reports no deep-analysis provider (Ghidra, Hopper, IDA, Kuna are not installed or configured). **Decompilation was deliberately not set up.** Clean-room rules ([principles](../development/principles.md)) prohibit using decompiled proprietary code, and the information LimeShot needs is behavioural. Only static metadata and observation were used.

## Evidence collected

All raw outputs are in `docs/research/evidence/rea/` (REA Evidence JSON with SHA-256 digests and tool provenance).

| File | Operation | Confidence | Findings |
|---|---|---|---|
| `inspect-plist.json` | `rea inspect-plist "/Applications/FL Studio 2026.app"` | VERIFIED | Bundle `com.image-line.flstudio`, version `26.1.1.5313`, executable `OsxFL`. Document types: `.flp` (project, Editor role), `.fst` (preset). URL scheme `flstudio://`. `NSHighResolutionCapable = YES`. Microphone and camera usage strings are declared. |
| `inspect-macho.json` | `rea inspect-macho` on `OsxFL` | VERIFIED | Universal binary (x86_64 + arm64). Minimum macOS 10.13, SDK 13.3. The launcher links only system frameworks (Foundation, Cocoa, AppKit, QuartzCore, CoreData, CoreFoundation) and uses `dlopen`/`dlsym`. The engine is therefore loaded dynamically. |
| `inspect-macho-flengine.json` | `rea inspect-macho` on `Contents/Libs/FLEngine_x64.dylib` | VERIFIED | The main engine is a separate universal dylib. Via `otool -L` it links CoreAudio, AudioUnit, AudioToolbox, CoreMIDI, CoreText, CoreGraphics, WebKit and SystemConfiguration. The embedded install name points at a CI build path. |
| `inspect-app.json` | `rea inspect` (app overview) | — | Failed with `provider_unavailable` (no Ghidra/Hopper). Expected. |
| `observe-ui-794.json` | `rea observe-native-ui --pid … --window-id 794` | — | Failed with `screen-recording-denied`. `screencapture -l` failed the same way. Live UI observation is blocked until the user grants permission. |

### Window inventory (CoreGraphics window list, OBSERVED)

With FL Studio 26.1.1 running, one main window of 1680×1020 pt (layer 0) was listed, plus a modal of 874×573 at layer 8 (likely the startup/registration dialog) and several 1680×30 strips (toolbar/hint-bar child windows). This matches the documented single main window with internal child windows (MDI). It does not match one native window per editor.

### Bundle anatomy (direct listing, OBSERVED)

- `Contents/Resources/FL/Shared/MacVclCocoa_x64.framework` suggests a VCL-style UI toolkit (Delphi/Pascal lineage) bridged to Cocoa. INFERRED. Combined with the window list, the interface is custom-drawn rather than built from native AppKit controls. This is why an accessibility tree would probably be sparse.
- `libblend2d.dylib`, `ILSvg_x64.dylib`, `libfreetype_x64.dylib`: vector 2D rendering and SVG icons. INFERRED: a resolution-independent, scalable UI.
- `elastique_x64.dylib` (time stretching), `libmp3lame_x64.dylib`, `EasyFLAC_x64.dylib`, `liboggio_x64.dylib`, `wavpack_x64.dylib`, `ffmpeg/`: export and import formats (WAV/MP3/FLAC/OGG/WavPack) and audio decoding.
- `Shared/Python`: the Python scripting runtime, matching `Settings/Hardware/*/script/*.py` controller scripts and `System/Config/Piano roll scripts`.
- `Help/HelpAliases.scr` (232 lines) maps help IDs to manual pages. It gives the list of documented windows: Channel Rack, Channel Settings (Sampler, Instrument, Misc, Layer, Time), Mixer (I/O routing, track properties, plugin slots), Browser (project / plugins / sounds), Automation (event editor, LFO, internal controllers, formula), Playlist automation clips, menus (File, Edit, Channels, View, Options, Tools, Help), Toolbar panels (CPU, monitor, pattern, transport, recording, shortcuts).
- `Data/Templates/{Advanced, Empty, Minimal, Other, Performance, Power User, Utility}`: project templates exist.
- `System/Config/NamePresets_{Channel,MixerSlot,MixerTrack,Pattern,PlaylistTrack}.txt`: naming presets per object type, which confirms that channels, mixer tracks, mixer slots, patterns and playlist tracks are separately named entities.
- `Plugins/Fruity/Generators` (52 entries) and `Plugins/Fruity/Effects` (99 entries): the native plugin catalogue used for the instrument/effect inventory.

## How these findings informed LimeShot

| Finding | Decision |
|---|---|
| Separate engine dylib, launcher shell | Engine logic in its own crate (`limeshot-engine`) with no UI dependency. |
| One main window with internal child windows | egui `Window`s constrained to the workspace area, toggled by F5–F9, rather than OS windows. |
| Custom-drawn, vector-scaled UI | Every editor (step grid, piano roll, playlist, mixer) is custom-painted. Icons are drawn in code and scale with egui's `pixels_per_point`. |
| `.flp` / `.fst` document types | LimeShot uses its own `.lime` project format. Reading `.flp` is tracked as a P3 interoperability feature (the format is proprietary and would need independent documentation). |
| Name presets for patterns, channels, tracks | Separate stable ids and names for channels, patterns, mixer tracks and playlist tracks. |
| Export libraries (MP3/FLAC/OGG) | WAV export first (MVP); other formats tracked as P1/P2. |

## Session 4 (2026-10-20): Accessibility granted

The REA MCP tools (`mcp__rea__*`) are loaded in Claude Code and Accessibility is granted. FL Studio draws its own UI
(see the VCL/Cocoa bridge above), so behaviour was observed by driving it with CoreGraphics mouse/keyboard events at
coordinates read from screenshots, then capturing the result with `screencapture`. Findings: `fl-studio-analysis.md`
§2–§4; screenshots `evidence/screenshots/fl-steps-toggle.png`, `fl-clip-no-loop.png`, `fl-note-length.png`. Only
behaviour was observed; FL was quit without saving and its installation was not modified.
