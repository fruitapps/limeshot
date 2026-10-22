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
