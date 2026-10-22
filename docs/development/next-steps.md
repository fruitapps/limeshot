# Next steps (prioritised)

## Mandatory

1. **Run CI on all platforms.** `.github/workflows/ci.yml` is on `fruitapps/limeshot`, but every run stays "pending"
   with no jobs (session 7): an org admin must check the org's Actions and billing settings. Then push, fix what
   Windows and Linux report, and update PF-002…PF-005 in `features.json`. `gh` is installed and logged in.
2. **Done (session 9):** the 16 issues from the owner's hands-on test (2026-10-21) are fixed and tested
   (`progress.md`, Session 9). Left open: a live FL check of Alt+wheel vertical zoom and a real-input pass over the
   new gestures in the live app.
3. **Done (session 11):** the owner's 19-item list (Playlist audio clips in the picker, Shift+drag, remembered clip
   length, drop ghost, plugin settings width, audition note-offs, graph overlay, Stop/Play silence, Browser preview,
   tool cursors, window raising/pinning, hint panel, detents, 1/3-step snap; `progress.md`, Session 11).
4. **VST support** (required by the project owner, 2026-10-21; in progress). Session 10 added `limeshot-plugins`
   (loading, scanning, instancing, processing, state, latency; tested with built-in plugins and all 36 bundles
   installed on the dev Mac), plugin channels and plugin inserts in the model, engine and UI, a child-process scan
   into a plugin database, and state in `.lime`. Session 12 added plugin windows (PLG-006: the plugin's own editor
   and a Settings tab, macOS; checked live with Vital, Serum 2, Surge XT and OTT). Next: plugin windows on Windows
   (`HWND`) and Linux (`X11EmbedWindowID` + `IRunLoop`) once CI runs there. Session 15 added FL's wrapper extras
   (presets ◀ ▶ and a presets menu with factory programs and `.vstpreset` files, the keyboard toggle, window places
   saved in the project) and plugin parameter automation (PLG-007); a live pass with a real plugin is still open.
   Delay compensation (PLG-009/MIX-012) is in too. Next: crash isolation while playing (PLG-010), a live pass of Find plugins…. Host VST3 plugins as instruments (Channel Rack) and
   effects (mixer insert slots): PLG-002 VST3 hosting, PLG-004 scanning, PLG-005 state saved in the `.lime` file,
   PLG-006 editor windows, PLG-007 parameter automation, PLG-009 latency. Put the FFI in its own crate (e.g.
   `limeshot-plugins`), the only crate allowed `unsafe`; record that exception and the chosen bindings in
   `decisions.md`. Keep the real-time rules (no allocation or locks in the audio callback) and plan crash isolation
   (PLG-010). Check the VST3 SDK's current licence against LimeShot's MIT/Apache licence before adding it, and test with
   a freely redistributable plugin. CLAP (PLG-001) and AU (PLG-003) can share the same host layer afterwards.
   **Lime Native VSTs** (owner, 2026-10-21): once hosting works, turn LimeSynth, the Sampler and every built-in effect
   (Lime EQ, Filter, Delay, Room, Comp, Limiter, Utility) into simple VST3 plugins built from the same DSP code, so
   they run in other hosts too and LimeShot can load them like any other plugin.

## Then

0. **FL Studio documentation, outside → in** (owner, 2026-10-22). Document each layer of FL's UI before building it,
   one layer per session, into machine-readable inventories under `docs/roadmap/` plus evidence screenshots:
   * [x] Main menu bar (session 13): `menus.json` / `menus.md`, every item and submenu.
   * [x] Dialogs and windows the main menus open (session 14): the settings window (10 tabs), render dialog, Plugin
     Manager, Patterns dialogs, Welcome — `settings.json` / `settings.md` (188 records). Not yet: the floating
     windows View opens (Project picker, Tempo tapper, Touch controller, Plugin performance monitor, Script output).
   Implementing from it: APP-025 should become FL's tabbed settings window (MIDI, Audio, General, File, Theme,
   Project, Info), starting with the pages LimeShot already has settings for.
   * [ ] Toolbar: every panel, button and drop-down (pattern selector, snap, hint bar, CPU panel, the toolbar's own
     menus) and the Browser's top bar.
   * [ ] Each editor's own menus and context menus (Channel Rack, Piano Roll, Playlist, Mixer; partly recorded in
     `fl-studio-analysis.md` §2–6).
   Then implement the menu bar (APP-026) from `menus.md`'s "open work by feature" table, highest priority first:
   SEQ-021 Pattern commands, Edit › Cut/Copy/Paste routed to the focused editor, a Tools menu with Macros, and
   View › Layout (UI-013). Keep `menus.json` in step with `menus.rs` (the check enforces labels).

4. **More live FL observation** (Accessibility is granted; session 4 settled clip looping, note length and step
   painting). Next: Playlist clip resize from the left edge and slip editing, Piano Roll right-click default length,
   Mixer send/route gestures, and Channel Settings layout. Repeat the real-input pass after UI changes (notes in the
   session 4 entry of `progress.md`).
5. **Keybindings and settings** (`docs/roadmap/keybindings.json`, `settings.json`): check the INFERRED bindings
   against the FL manual or the app, add the missing ones that are cheap (F12, Esc, Ctrl+Q dialog), then a Settings window with pages (APP-025) and remappable shortcuts (UI-011).
6. **Mixer:** pre-fader sends and remembering window layout per project/settings (sidechain is in Mandatory 2).
7. **Effects polish:** delay tempo sync, EQ curve display, presets for effects and instruments (FX-014, INS-010).
8. **Audio and automation follow-ups:** more decoders (INS-008, symphonia), clip fades, automated controls moving on
   screen (AUT-013), point modes (AUT-012), tempo automation (AUT-008), taller tracks or a curve editor for automation.
9. **MIDI input** (WF-003, `midir`) and MIDI recording (REC-002). Open an exported `.mid` in another program once.
10. **Piano Roll:** stamp tool (PR-027), scale highlighting (PR-013), quantize and chop dialogs.
11. **Detachable panels** (UI-007) so the Channel Rack and Piano Roll can be seen together.
12. **Packaging** (PF-006).
    Also: `.flp` import follow-ups (APP-022, `docs/research/flp-format.md`): automation clip curves into LimeShot
    automation clips, FL's mixer effects that have a LimeShot equivalent, further arrangements, and test files made
    in an older FL version.
13. **More plugin formats:** CLAP (PLG-001) and Audio Units (PLG-003) on the VST host layer.
