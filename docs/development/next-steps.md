# Next steps (prioritised)

## Mandatory

1. **Run CI on all platforms.** `.github/workflows/ci.yml` is on `fruitapps/limeshot`, but every run stays "pending"
   with no jobs (session 7): an org admin must check the org's Actions and billing settings. Then push, fix what
   Windows and Linux report, and update PF-002…PF-005 in `features.json`. `gh` is installed and logged in.
2. **Done (session 9):** the 16 issues from the owner's hands-on test (2026-10-21) are fixed and tested
   (`progress.md`, Session 9). Left open: a live FL check of Alt+wheel vertical zoom and a real-input pass over the
   new gestures in the live app.
3. **VST support** (required by the project owner, 2026-10-21; in progress). Session 10 added `limeshot-plugins`
   (loading, scanning, instancing, processing, state, latency; tested with built-in plugins and all 36 bundles
   installed on the dev Mac). Next: plugin channels and insert slots in the model and engine, state in `.lime`,
   a child-process scan into a plugin database, the plugin pickers, then editor windows. Host VST3 plugins as instruments (Channel Rack) and
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
