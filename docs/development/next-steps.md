# Next steps (prioritised)

1. **Run CI on all platforms.** Create the GitHub repository, push, and fix whatever Windows and Linux report. Then
   update PF-002…PF-005 in `features.json`. The CI file is `.github/workflows/ci.yml`. Expect Linux system-package
   tweaks.
2. **Manual GUI pass by a person** using the checklist in `progress.md`. Record results in `progress.md`. Fix
   interaction rough edges found (window placement defaults overlap at small window sizes).
3. **Live FL Studio observation.** Grant Screen Recording/Accessibility to the REA host, restart Claude Code so the REA
   MCP tools load, run `observe-native-ui` on FL Studio, and resolve the UNKNOWN/INFERRED items in
   `fl-studio-analysis.md` (clip looping D14, default note length, step drag-paint).
4. **Copy/paste** for notes, steps and clips (PR-019, SEQ-015) and **Quantize** (PR-014).
5. **Insert effects framework** (FX-001), then EQ, filter, compressor, reverb and delay (FX-002…FX-008), and mixer
   sends (MIX-008).
6. **Swing** (SEQ-010) in `sequencer::compile`.
7. **Audio clips** (PL-014) with more decoders (INS-008: symphonia, permissive).
8. **Automation clips** (AUT-001…003): a third `ClipSource`, per-block parameter evaluation in the renderer.
9. **MIDI input** (WF-003) with `midir`, routed to the selected channel; then MIDI recording (REC-002).
10. **MIDI file import/export** (PR-020, EXP-012), own SMF implementation.
11. **Autosave/recovery** (APP-016).
12. **Packaging** (PF-006): app bundle with `samples/` in `Resources`, Windows installer, AppImage.
13. **Plugin hosting** (PLG-*): CLAP first, in an isolated crate where `unsafe` is confined to FFI modules.
