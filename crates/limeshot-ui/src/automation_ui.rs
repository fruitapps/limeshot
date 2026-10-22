//! "Create automation clip" on controls. As in FL Studio, right-clicking an automatable control
//! offers it; the new clip starts as a flat line at the control's value and spans the song.

use crate::app::LimeShotApp;
use crate::theme::Theme;
use egui::{Response, RichText};
use limeshot_core::{AutomationTarget, ClipId, ClipSource, Command};

/// Attach the automation context menu to a control's response.
pub fn menu(app: &mut LimeShotApp, resp: &Response, target: AutomationTarget) {
    crate::widgets::clickable(resp).context_menu(|ui| {
        ui.label(
            RichText::new(target.label(app.project()))
                .small()
                .color(Theme::current().text_dim),
        );
        ui.separator();
        if ui.button("Create automation clip").clicked() {
            create(app, target);
            ui.close();
        }
    });
}

/// Create an automation clip for `target` on the first Playlist track that is free for the
/// length of the song, and show it.
pub fn create(app: &mut LimeShotApp, target: AutomationTarget) -> Option<ClipId> {
    create_with(app, target, None, None)
}

/// [`create`] for a control whose value the project does not hold (a plugin parameter):
/// `value` is its normalised value now and `name` the automation's name.
pub fn create_with(
    app: &mut LimeShotApp,
    target: AutomationTarget,
    value: Option<f32>,
    name: Option<String>,
) -> Option<ClipId> {
    let sig = app.project().time_signature;
    let length = app.project().playlist.song_length(sig).max(sig.ticks_per_bar());
    let playlist = &app.project().playlist;
    let free =
        (0..playlist.tracks.len() as u32).find(|t| !playlist.clips.iter().any(|c| c.track == *t && c.start < length));
    let track = match free {
        Some(t) => t,
        None => {
            app.exec(Command::AddPlaylistTrack)?;
            app.project().playlist.tracks.len() as u32 - 1
        }
    };
    let id = ClipId(app.exec(Command::AddAutomationClip {
        target,
        track,
        start: 0,
        length,
        value,
        name,
    })?);
    if let Some(a) = app.project().clip(id).and_then(|c| c.automation()) {
        app.ui.place = Some(ClipSource::Automation(a));
    }
    app.show_window(crate::workspace::Win::Playlist);
    let name = app
        .project()
        .playlist
        .tracks
        .get(track as usize)
        .map(|t| t.name.clone())
        .unwrap_or_default();
    app.toast(format!("Automation clip created on {name}"));
    Some(id)
}
