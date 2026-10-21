# Next steps (prioritised)

1. **Run CI on all platforms.** Needs a GitHub repository (`.github/workflows/ci.yml` is ready). Push and fix what
   Windows and Linux report, then update PF-002…PF-005 in `features.json`.
2. **More live FL observation** (Accessibility is granted; session 4 settled clip looping, note length and step
   painting). Next: Playlist clip resize from the left edge and slip editing, Piano Roll right-click default length,
   Mixer send/route gestures, and Channel Settings layout. Repeat the real-input pass after UI changes (notes in the
   session 4 entry of `progress.md`).
3. **Keybindings and settings** (`docs/roadmap/keybindings.json`, `settings.json`): check the INFERRED bindings
   against the FL manual or the app, add the missing ones that are cheap (Alt+Up/Down channel move, F2 rename, F12,
   Esc, Ctrl+Q dialog), then a Settings window with pages (APP-025) and remappable shortcuts (UI-011).
4. **Mixer:** sidechain inputs for the compressor (MIX-009), pre-fader sends, a gain-reduction meter, and remembering
   window layout per project/settings.
5. **Effects polish:** delay tempo sync, EQ curve display, presets for effects and instruments (FX-014, INS-010).
6. **Audio and automation follow-ups:** more decoders (INS-008, symphonia), clip fades, automated controls moving on
   screen (AUT-013), point modes (AUT-012), tempo automation (AUT-008), taller tracks or a curve editor for automation.
7. **MIDI input** (WF-003, `midir`) and MIDI recording (REC-002). Open an exported `.mid` in another program once.
8. **Piano Roll:** stamp tool (PR-027), scale highlighting (PR-013), quantize and chop dialogs; the same tool strip for
   the Playlist (KB-064).
9. **Detachable panels** (UI-007) so the Channel Rack and Piano Roll can be seen together.
10. **Packaging** (PF-006).
    Also: `.flp` import follow-ups (APP-022, `docs/research/flp-format.md`): automation clip curves into LimeShot
    automation clips, FL's mixer effects that have a LimeShot equivalent, further arrangements, and test files made
    in an older FL version.
11. **Plugin hosting** (PLG-*): CLAP first, in an isolated crate.
