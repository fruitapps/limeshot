# Next steps (prioritised)

1. **Run CI on all platforms.** Needs a GitHub repository (`.github/workflows/ci.yml` is ready). Push and fix what
   Windows and Linux report, then update PF-002…PF-005 in `features.json`.
2. **Grant Accessibility** to the Claude app (System Settings › Privacy & Security › Accessibility) so REA can read
   FL Studio's UI and drive it. Then resolve the INFERRED/UNKNOWN items in `fl-studio-analysis.md` (clip looping D14,
   default note length, step drag-paint), and run a real-input pass over the live LimeShot window with the checklist
   in `progress.md`.
3. **Mixer:** sidechain inputs for the compressor (MIX-009), pre-fader sends, a gain-reduction meter, and remembering
   window layout per project/settings.
4. **Effects polish:** delay tempo sync, EQ curve display, presets for effects and instruments (FX-014, INS-010).
5. **Audio clips** (PL-014) with more decoders (INS-008, symphonia), then **automation clips** (AUT-001…003).
6. **MIDI input** (WF-003, `midir`) and MIDI recording (REC-002); **MIDI file import/export** (PR-020, EXP-012).
7. **Piano Roll tools:** paint/slice/select tool palette (PR-012), scale highlighting (PR-013), quantize dialog.
8. **Detachable panels** (UI-007) so the Channel Rack and Piano Roll can be seen together.
9. **Autosave/recovery** (APP-016); **packaging** (PF-006).
10. **Plugin hosting** (PLG-*): CLAP first, in an isolated crate.
