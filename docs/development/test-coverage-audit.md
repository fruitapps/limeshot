# Test coverage audit — 2026-10-22

**2026-10-23 update:** the audit below is historical. Current real-VST3 integration covers hosted audio clips,
note pan, exact track EQ, Gate/Drive/Chorus, sidechain keys, reduction and voice statistics. Engine scheduling/DSP
tests use test-only processors through the real plugin rack. The 137 headless UI tests remain; they do not establish
live native/platform behavior. PR #7 review adds transport, sidechain remapping, bypass, sample-retirement and held-note
regressions. Current evidence: [review handoff](handoff-pr7-rewrite-review.md).

The existing tests are a useful regression suite for the implemented composition workflow, but
they are **not comprehensive enough to establish total application functionality**. Passing the
suite does not verify native windows, audio hardware, failure recovery, or every acceptance
criterion. Features that are not implemented cannot be established by these tests either.

This audit compared the test inventory and representative assertions with the feature records,
architecture, CI configuration and documented limitations. It is a structural coverage review,
not a measured line/branch coverage or mutation-testing result. No production code or test
behavior was changed. Remote CI and live hardware/native UI were not reverified in this audit.

## Checks performed

On the local macOS arm64 host:

| Check | Result |
|---|---|
| `cargo test --workspace` | 300 passed; 0 failed; 0 reported ignored |
| `cargo build --workspace` | Passed |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed |
| `cargo fmt --all --check` | Passed |
| `python3 scripts/features.py check` | Passed |
| `git diff --check` before adding this report | Passed |

The 300 tests comprise app 113 (111 headless UI + 2 workflow), core 65, DSP 23, sequencer 8,
engine 39, audio 5, project 22, plugins 22, and UI library 3. A passing count is not a coverage
percentage. `installed_fl_projects_all_import` returns successfully when FL Studio is absent,
so “0 ignored” does not guarantee all environment-dependent checks actually exercised fixtures.

The inventory contains 272 features: 136 VERIFIED, 19 IMPLEMENTED, 11 IN_PROGRESS,
101 NOT_STARTED, 4 DEFERRED and 1 RESEARCHED. These are tracking statuses, not test coverage
measurements; VERIFIED permits recorded manual evidence as well as automated tests.

## Prioritized findings

### 1. High: realtime guarantees have no enforceable regression gate

Scheduling is well tested across buffer sizes, but there is no allocation/deallocation detector,
queue saturation test, callback deadline gate, or sustained edit/playback stress test.

There is a concrete inspection finding behind this gap: in
[`AudioState::drain`](../../crates/limeshot-audio/src/lib.rs), the snapshot branch ignores the
error returned by a full garbage ring. The returned old snapshot is then dropped in the audio
callback; if it holds the final reference, this can free its large allocations there. This
contradicts the no-large-frees rule while all 300 tests still pass. Effects and instruments
already have explicit retain-for-later handling in the same method.

`Player::send` also drops messages when its ring is full, and the UI wrapper logs the loss.
No test establishes behavior when Stop, note-off, snapshot or processor messages cannot be
queued. This is a coverage finding; no queue-flood reproduction was performed here.

The bridge hang test establishes eventual silence and host survival, not uninterrupted audio:
[`a_hung_bridge_times_out`](../../crates/limeshot-plugins/tests/bridge.rs) allows up to three
500 ms timeouts. `BridgedProcessor::block` performs synchronous socket I/O on the processing
path. That cannot establish the architecture's no-I/O callback guarantee or absence of dropouts.

Needed: deterministic full-ring tests that inspect where objects are destroyed, critical-message
recovery tests, allocation guards around warmed rendering, and a separate measured realtime
stress/hang test with an explicit audio-buffer deadline.

### 2. High: native and device behavior is outside the harness

[`tests/ui.rs`](../../crates/limeshot-app/tests/ui.rs) creates `LimeShotApp::new_silent` with a
1440 × 900 egui harness. It checks substantial interaction behavior, but creates no native
application window or hardware audio stream. CI explicitly runs without native windows or a
sound card and only builds the screenshot example.

[`an_editor_opens_takes_edits_and_resizes_itself`](../../crates/limeshot-plugins/tests/host.rs)
uses the built-in plugin's `attach_headless`. It does not validate macOS NSView embedding,
rendering, actual keyboard focus, occlusion or third-party resize negotiation. The pending
PLG-006 live checks are therefore material, even with passing wrapper tests.

APP-012 is VERIFIED, but its automated test checks buffer-size request validation. Its notes
explicitly exclude switching between multiple hardware outputs. Stream restart, device loss,
unsupported configurations, negotiated formats and audible behavior need further coverage.
The sample writer test covers f32 and i16 examples; it does not exercise every supported device
format through the actual callback.

Needed: a recorded native plugin matrix (fixed/resizable editors, tabs, cover/pin, keyboard,
save/reopen), real device switching/disconnection/restart checks, and per-platform runtime
evidence. Test HiDPI, narrow windows and detached windows with actual rendering and input.

### 3. High: data-loss prevention is tested mainly on successful paths

APP-014 cites
[`new_project_asks_to_save_unsaved_changes`](../../crates/limeshot-app/tests/ui.rs). It checks
New with Cancel and Don't save. It does not exercise Save, save-dialog cancellation, save failure,
Open replacement, or the native Quit/close request. Sharing a dialog implementation reduces
risk but does not verify those entry paths and continuations.

[`autosave_survives_a_crash_and_is_removed_once_saved`](../../crates/limeshot-app/tests/ui.rs)
uses a zero interval and simulates a crash by dropping the harness. It checks successful recovery
and save cleanup, but not a killed process during either write, damaged/missing session metadata,
discard, failed writes, an original project with relative samples, or competing instances.
Autosave writes the project atomically and then writes `session.json` separately; interrupted
pair updates deserve explicit tests.

Project save tests establish ordinary round trips and overwrite success, not write/fsync/rename
failure or crash durability. `overwriting_keeps_one_file` checks the reloaded title without
actually counting directory entries. `unknown_fields_survive` injects fields only at the project
and channel levels; it does not exercise all the nested locations APP-008 promises.

Needed: injected filesystem failures with an existing destination preserved, all discard/save
continuations, process-termination recovery fixtures, relative-asset recovery, and unknown-field
round trips at every documented entity level.

### 4. Medium: feature evidence is incomplete and sometimes weaker than the claim

Five IMPLEMENTED records have empty test lists: APP-006 (history), BR-005 (current project tree),
TR-010 (panic), UI-005 (scaling), and PL-026 (clip clipboard). Empty lists do not prove the code
is wholly untested: the Edit-menu UI test already copies/pastes/cuts one pattern clip. However,
it checks clipboard kind and clip counts, not the full PL-026 contract: audio/automation sources,
offsets, lengths, Song-mode placement and selection. Its “No automated test yet” note is stale.

APP-013 references `window_shortcuts_toggle_editors`, which is not a current test function;
the existing window shortcut tests use other names. The inventory validator only requires a
nonempty `tests` list for VERIFIED: it does not resolve names or prove acceptance criteria.

WF-001's evidence is the CI demo/render comparison. That does not test CLI `run` command scripts,
invalid arguments, failed commands, exit codes or all other subcommands. APP-007 has no historical
migration fixture, and currently no historical migration is implemented (version 1 is the first
format); refusing newer files alone should not be interpreted as testing future migrations.

Needed: an acceptance-criterion-to-test mapping, resolvable automated test references, explicit
manual evidence records, and tests for missing feature contracts. Preserve the distinction
between implemented, partly tested and runtime verified.

### 5. Medium: interoperability, boundary cases and broad combinations remain limited

MIDI has useful hand-authored running-status/resolution and malformed-input tests, but export
round trips primarily use LimeShot's own reader. Matching reader/writer errors can pass.
FLP has value-checked FL 26 fixtures; the installed-demo sweep checks import success rather than
musical correctness, and older-version value fixtures remain missing.

DSP tests check useful individual responses and invalid parameters, and engine tests cover
mixing, automation, clip chase, effects and PDC. They do not establish all supported combinations
of sample rates, parameter extremes, dynamic reconfiguration, routing depth, capacity limits,
long sessions and third-party plugins. The 128-voice limit has a test; that does not substitute
for testing the other bounded queues, event lists and delay-path limits.

There is no property-testing/fuzzing harness or measured mutation/branch coverage gate in the
repository. Existing hostile MIDI/FLP/WAV examples are valuable but finite. Command sequences
would benefit from invariant checks: failed edits preserve state, undo/redo restores exact
state, references remain valid after removals/reorders, and sanitation remains stable.

Needed: independent MIDI/WAV validation, older real FLP value fixtures, parser fuzzing,
generated command sequences, and targeted boundary/combination tests. Add coverage reporting
to locate unexercised branches; do not use a percentage alone as the release criterion.

## Recommended completion order

1. Add failing regressions for full audio rings and callback destruction; address the behavior
   and define the bridge's realtime expectations.
2. Complete native plugin and hardware/platform runtime checks already listed in
   [next-steps.md](next-steps.md), with dated results.
3. Cover save/close/open failure paths and interrupted autosave/recovery.
4. Repair test references and map each implemented feature's acceptance criteria to evidence.
5. Expand independent format fixtures, invariant/property tests, boundary cases and visual checks.

The current suite supports confidence in the tested MVP and many editor regressions. It does
not support a claim that every implemented feature, platform or failure mode works.
