# Screenshots and visual review

These images record earlier offscreen `ui_shot` renders and live macOS captures. They illustrate the application,
but do not establish that later interaction changes or native plugin embedding have been verified.
FL Studio research evidence is kept separately in `docs/research/evidence/`.

Regenerate a review image with:

```sh
cargo run -p limeshot-app --example ui_shot -- out/review.png --piano --mixer
cargo run -p limeshot-app --example ui_shot -- out/dialogs.png --dialog=export --tapper
```

Other scenarios include `--fx`, `--settings`, `--melody`, `--empty`, `--audio`, `--automation`, `--synth`,
`--scale`, `--plugin` and `--plugin-window`. Dialog choices are `export`, `quit`, `marker` and `control`. Read the resulting PNG
after UI edits. Offscreen plugin captures exercise the parameter fallback; native views require a live application
and working capture/input permissions. See the latest [progress log](../development/progress.md) for review evidence.
