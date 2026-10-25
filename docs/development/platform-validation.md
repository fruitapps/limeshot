# Platform validation

Evidence checked 2026-10-22. Headless CI and native GUI/audio-device checks are separate.
Successful results apply to the listed commit, not automatically to later work.

| Platform | Commit / job | Result | Still open |
|---|---|---|---|
| macOS Apple Silicon | `38d9a7b`, [119990085852](https://github.com/fruitapps/limeshot/actions/runs/39826217710/job/119990085852) | Clippy, all-target build, workspace tests, deterministic CLI render/reload, inventories passed | Live native plugin embedding; later commits need CI |
| Windows x86-64 | `38d9a7b`, [119990085936](https://github.com/fruitapps/limeshot/actions/runs/39826217710/job/119990085936) | Same checks passed | Native GUI/audio devices; later commits need CI |
| Linux x86-64 | `38d9a7b`, [119990085807](https://github.com/fruitapps/limeshot/actions/runs/39826217710/job/119990085807) | Same checks passed | Native GUI/audio devices; later commits need CI |
| Linux ARM64 | `38d9a7b`, [119990085903](https://github.com/fruitapps/limeshot/actions/runs/39826217710/job/119990085903) | Same checks passed natively | Native GUI/audio devices; later commits need CI |
| Windows ARM64 | `38d9a7b`, [119990085709](https://github.com/fruitapps/limeshot/actions/runs/39826217710/job/119990085709) | Cross-build passed | Native runtime, GUI and audio devices |
| macOS Intel | `38d9a7b`, [119990085995](https://github.com/fruitapps/limeshot/actions/runs/39826217710/job/119990085995) | Clippy, all-target build, workspace tests, deterministic CLI render/reload, inventories passed natively on `macos-15-intel` | Native GUI/audio devices; later commits need CI |

All seven jobs (including rustfmt) in [39826217710](https://github.com/fruitapps/limeshot/actions/runs/39826217710) passed. This confirms the replacement for
retired `macos-13`; previous Intel jobs with no steps were not test failures.
The subsequent [PR run 39827943653](https://github.com/fruitapps/limeshot/actions/runs/39827943653) at `ec46e8f`
had passed rustfmt, Windows ARM64 cross-build and Linux ARM64 when inspected; other jobs were still running/queued.
New branch changes require another CI run.

Live checks were deferred at the owner's request while the Mac was locked. PLG-006 stays IMPLEMENTED.

No later platform CI pass is recorded: subsequent runs were blocked by GitHub account billing (see progress.md).
PRs #7, #10, #11 and #12 have since merged into main, so main's current commit has no recorded CI evidence.


## 2026-10-25 CI/release repair

PRs #7 and #10…#14 are merged; main is `5ab09c9`. Billing no longer blocks execution.
[CI 40211683423](https://github.com/fruitapps/limeshot/actions/runs/40211683423) passes Windows x86-64 and
Windows ARM64 cross-compilation. Both Linux architectures pass Clippy/build/bundle generation, then SIGSEGV in
`demo_song_renders_and_reloads_identically` while the concurrent MVP test finishes. Apple Silicon fails
`dragging_a_sample_to_the_rack_makes_a_channel_or_replaces_one`: the channel is still named Kick because the
background import has not committed after one synthetic frame. The fix waits for the actual edit through the
existing bounded `wait_for` helper and keeps all resulting channel assertions.

[Native run 40205510432](https://github.com/fruitapps/limeshot/actions/runs/40205510432) passes macOS and Windows;
Linux's release bundle build lacks `wayland-client.pc`. Add `libwayland-dev` to its dependency installation.
VST3 modules now remain strongly cached for the process lifetime (D41), with a concurrent reload regression.
Fresh platform checks are required before declaring the observed Linux crashes resolved.

[Alpha packaging run 40211718081](https://github.com/fruitapps/limeshot/actions/runs/40211718081) targets
`v0.1.0-alpha.1` at `5ab09c9`. Windows, Apple Silicon and both Linux packaging/install checks pass;
all five packaging jobs and draft-release assembly have now passed. The prerelease remains a draft.
The tag and its packages predate this repair; passing packaging alone does not establish workspace-test health.


All ten original draft assets were downloaded and their SHA-256 bytes verified. GitHub had renamed the Debian
filenames from `0.1.0~alpha.1` to `0.1.0.alpha.1`, but the generated manifest kept the former names: unmodified
`shasum -c` failed for both Debian packages. Corrected the draft's `SHA256SUMS.txt` to the actual uploaded names,
verified all ten entries, uploaded only the corrected manifest and downloaded it again to confirm the replacement.
Future Debian artifact filenames use the original `0.1.0-alpha.1` version; control metadata keeps `0.1.0~alpha.1`.
Both macOS DMGs pass `hdiutil verify`; the Windows portable ZIP passes CRC checks; both Linux tarballs contain
all twelve VST3 bundles, CLI and install/uninstall scripts. No tag was moved and no release was published.

Repair validation: [CI 40214027778](https://github.com/fruitapps/limeshot/actions/runs/40214027778),
[native 40214027805](https://github.com/fruitapps/limeshot/actions/runs/40214027805), and
[packaging 40214064487](https://github.com/fruitapps/limeshot/actions/runs/40214064487) run the initial source fix
`ee55b76`. The filename repair follows separately; final results are recorded in PR #15. Local verification:
427 workspace tests including 157 UI tests, strict all-target Clippy/build, formatting, inventories, references,
five repository-checker tests, 25 independent rack-drop runs, and byte-identical hosted demo/reload WAVs (-3.7 dBFS).
