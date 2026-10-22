# Platform validation

Evidence checked 2026-10-22. Headless CI and native GUI/audio-device checks are separate.
Successful results apply to the listed commit, not automatically to later work.

| Platform | Commit / job | Result | Still open |
|---|---|---|---|
| macOS Apple Silicon | `7bd6001`, [119939442875](https://github.com/fruitapps/limeshot/actions/runs/39811714297/job/119939442875) | Clippy, all-target build, workspace tests, deterministic CLI render/reload, inventories passed | Live native plugin embedding; later commits need CI |
| Windows x86-64 | `ab54402`, [119960022648](https://github.com/fruitapps/limeshot/actions/runs/39817431504/job/119960022648) | Same checks passed; earlier hint-panel/path regressions confirmed fixed | Native GUI/audio devices; later commits need CI |
| Linux x86-64 | `ab54402`, [119960022486](https://github.com/fruitapps/limeshot/actions/runs/39817431504/job/119960022486) | Same checks passed | Native GUI/audio devices; later commits need CI |
| Linux ARM64 | `ab54402`, [119960022953](https://github.com/fruitapps/limeshot/actions/runs/39817431504/job/119960022953) | Same checks passed natively | Native GUI/audio devices; later commits need CI |
| Windows ARM64 | `ab54402`, [119960022229](https://github.com/fruitapps/limeshot/actions/runs/39817431504/job/119960022229) | Cross-build passed | Native runtime, GUI and audio devices |
| macOS Intel | `ab54402`, [119960022662](https://github.com/fruitapps/limeshot/actions/runs/39817431504/job/119960022662) | Queued with no steps on `macos-13` | CI on supported runner, native GUI/audio devices |

The newest main run [39824984937](https://github.com/fruitapps/limeshot/actions/runs/39824984937), at `dd53fb3`,
was pending with no jobs when inspected. Older workflows still had queued Intel jobs.
GitHub [retired macOS 13 runners](https://github.blog/changelog/2025-09-19-github-actions-macos-13-runner-image-is-closing-down/);
the Intel matrix now uses the supported `macos-15-intel` label. A new run must confirm the replacement.

Live checks were deferred at the owner's request while the Mac was locked. PLG-006 stays IMPLEMENTED.
