//! The Playlist's ⏵ options menu (FL Studio's Playlist menu: Edit, Tools, Select, Zoom, Time
//! markers, Center) and the commands behind it. The Playlist's keys run the same [`Act`]s.

use crate::app::{ClipOp, LimeShotApp};
use crate::piano_menu::shift_digit;
use crate::shortcuts::label as key;
use egui::{Button, Context, Key, Modifiers, Ui};
use limeshot_core::*;
use limeshot_engine::PlayMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Act {
    Cut,
    Copy,
    Paste,
    Duplicate,
    Delete,
    ShiftLeft,
    ShiftRight,
    ShiftUp,
    ShiftDown,
    MakeUnique,
    Mute,
    Unmute,
    InsertSpace,
    SliceInsertSpace,
    DeleteSpace,
    MergeClips,
    GroupTracks,
    UngroupTracks,
    CollapseGroupedTracks,
    ExpandGroupedTracks,
    QuickQuantizeStarts,
    Deselect,
    SelectAll,
    SelectSource,
    SelectMuted,
    SelectOverlapping,
    SelectStacked,
    InvertSelection,
    SelectTimeAround,
    ZoomIn,
    ZoomOut,
    /// Quick zoom 1–3, Zoom out far, Zoom on selection (Shift+1 … 5).
    QuickZoom(u8),
    AddMarker,
    AddMarkerAuto,
    DeleteMarkers,
    Center,
}

const EDIT: &[(&str, &str, Act)] = &[
    ("Cut", "Cmd+X", Act::Cut),
    ("Copy", "Cmd+C", Act::Copy),
    ("Paste", "Cmd+V", Act::Paste),
    ("Duplicate", "Cmd+B", Act::Duplicate),
    ("Delete", "Delete", Act::Delete),
    ("Shift left", "Shift+←", Act::ShiftLeft),
    ("Shift right", "Shift+→", Act::ShiftRight),
    ("Shift up", "Shift+↑", Act::ShiftUp),
    ("Shift down", "Shift+↓", Act::ShiftDown),
    ("Make unique", "Shift+Cmd+V", Act::MakeUnique),
    ("Mute", "Alt+M", Act::Mute),
    ("Unmute", "Shift+Alt+M", Act::Unmute),
    ("Insert space", "Cmd+Insert", Act::InsertSpace),
    ("Slice & insert space", "Cmd+Alt+Insert", Act::SliceInsertSpace),
    ("Delete space", "Cmd+Delete", Act::DeleteSpace),
    ("Merge selected clips", "Cmd+G", Act::MergeClips),
    ("Group", "Shift+G", Act::GroupTracks),
    ("Ungroup", "Alt+G", Act::UngroupTracks),
];

const VIEW: &[(&str, &str, Act)] = &[
    ("Collapse all grouped tracks", "", Act::CollapseGroupedTracks),
    ("Expand all grouped tracks", "", Act::ExpandGroupedTracks),
];

const TOOLS: &[(&str, &str, Act)] = &[("Quick quantize start times", "Shift+Q", Act::QuickQuantizeStarts)];

const SELECT: &[(&str, &str, Act)] = &[
    ("Deselect", "Cmd+D", Act::Deselect),
    ("Select all", "Cmd+A", Act::SelectAll),
    ("Select by selected source", "Shift+C", Act::SelectSource),
    ("Select muted", "", Act::SelectMuted),
    ("Select overlapping clips", "", Act::SelectOverlapping),
    ("Select stacked clips", "", Act::SelectStacked),
    ("Invert selection", "Shift+I", Act::InvertSelection),
    ("Select time around selection", "Cmd+Enter", Act::SelectTimeAround),
];

const ZOOM: &[(&str, &str, Act)] = &[
    ("Zoom in", "PgUp", Act::ZoomIn),
    ("Zoom out", "PgDn", Act::ZoomOut),
    ("Quick zoom 1", "Shift+1", Act::QuickZoom(1)),
    ("Quick zoom 2", "Shift+2", Act::QuickZoom(2)),
    ("Quick zoom 3", "Shift+3", Act::QuickZoom(3)),
    ("Zoom out far", "Shift+4", Act::QuickZoom(4)),
    ("Zoom on selection", "Shift+5", Act::QuickZoom(5)),
];

const MARKERS: &[(&str, &str, Act)] = &[
    ("Add one…", "Alt+T", Act::AddMarker),
    ("Add auto", "Cmd+T", Act::AddMarkerAuto),
    ("Delete", "", Act::DeleteMarkers),
];

fn items(app: &mut LimeShotApp, ui: &mut Ui, list: &[(&str, &str, Act)]) {
    for &(text, keys, act) in list {
        if ui.add(Button::new(text).shortcut_text(key(keys))).clicked() {
            ui.close();
            run(app, act);
        }
    }
}

/// The ⏵ menu at the left of the Playlist's header.
pub fn menu(app: &mut LimeShotApp, ui: &mut Ui) {
    ui.menu_button("Edit", |ui| items(app, ui, EDIT));
    ui.menu_button("Tools", |ui| items(app, ui, TOOLS));
    ui.menu_button("View", |ui| items(app, ui, VIEW));
    ui.menu_button("Select", |ui| items(app, ui, SELECT));
    ui.menu_button("Zoom", |ui| items(app, ui, ZOOM));
    ui.menu_button("Time markers", |ui| items(app, ui, MARKERS));
    ui.menu_button("Arrangement", |ui| arrangement_menu(app, ui));
    if ui.add(Button::new("Center").shortcut_text(key("Shift+0"))).clicked() {
        ui.close();
        run(app, Act::Center);
    }
}

/// The Playlist's Arrangement menu (also shown from the header bar's arrangement selector).
pub fn arrangement_menu(app: &mut LimeShotApp, ui: &mut Ui) {
    let cur = app.project().current_arrangement;
    let count = app.project().arrangement_count();
    let names: Vec<String> = (0..count)
        .filter_map(|i| app.project().arrangement(i).map(|a| a.name.clone()))
        .collect();
    for (i, name) in names.iter().enumerate() {
        if ui.selectable_label(i == cur, name).clicked() {
            app.exec(Command::SelectArrangement { index: i });
            ui.close();
        }
    }
    ui.separator();
    if ui.button("Add one…").clicked() {
        app.dialog = Some(crate::dialogs::Dialog::new_arrangement(app));
        ui.close();
    }
    if ui.button("Clone").clicked() {
        let base = names.get(cur).cloned().unwrap_or_else(|| "Arrangement".into());
        let name = format!("{base} (copy)");
        app.exec(Command::CloneArrangement {
            index: cur,
            name: Some(name),
        });
        ui.close();
    }
    if ui.button("Rename…").clicked() {
        app.dialog = Some(crate::dialogs::Dialog::rename_arrangement(app, cur));
        ui.close();
    }
    if count > 1 {
        ui.menu_button("Merge with", |ui| {
            for (i, name) in names.iter().enumerate() {
                if i != cur && ui.button(name).clicked() {
                    app.exec(Command::MergeArrangement { source: i });
                    ui.close();
                }
            }
        });
    } else {
        ui.add_enabled(false, Button::new("Merge with"));
    }
    if ui
        .add_enabled(
            count > 1,
            Button::new(egui::RichText::new("Delete").color(crate::theme::Theme::current().danger)),
        )
        .clicked()
    {
        app.exec(Command::DeleteArrangement { index: cur });
        ui.close();
    }
}

fn select_new(app: &mut LimeShotApp, before: &[ClipId]) {
    app.ui.playlist.selection = app
        .project()
        .playlist
        .clips
        .iter()
        .map(|c| c.id)
        .filter(|id| !before.contains(id))
        .collect();
}

fn clip_ids(app: &LimeShotApp) -> Vec<ClipId> {
    app.project().playlist.clips.iter().map(|c| c.id).collect()
}

/// The song position in Song mode, otherwise the time selection's start (or 0).
fn position(app: &LimeShotApp) -> Tick {
    if app.ui.mode == PlayMode::Song {
        app.player.status().position()
    } else {
        app.ui.playlist.time_sel.map_or(0, |s| s.0)
    }
}

pub fn run(app: &mut LimeShotApp, act: Act) {
    let sig = app.project().time_signature;
    let snap = app.ui.snap.ticks(sig).unwrap_or(STEP_TICKS) as i64;
    let sel = app.ui.playlist.selection.clone();
    let clips: Vec<Clip> = app.project().playlist.clips.clone();
    let mv = |app: &mut LimeShotApp, dt: i64, dtrack: i32| {
        if !sel.is_empty() {
            app.exec(Command::MoveClips {
                clips: sel.clone(),
                dt,
                dtrack,
            });
        }
    };
    match act {
        Act::Cut => crate::playlist::clipboard(app, ClipOp::Cut),
        Act::Copy => crate::playlist::clipboard(app, ClipOp::Copy),
        Act::Paste => crate::playlist::clipboard(app, ClipOp::Paste),
        Act::Duplicate => {
            let sel = if sel.is_empty() && app.ui.playlist.time_sel.is_none() {
                clip_ids(app)
            } else {
                sel
            };
            let before = clip_ids(app);
            if let Some((from, to)) = app.ui.playlist.time_sel.filter(|(a, b)| b > a) {
                app.exec(Command::DuplicateClipsBy {
                    clips: sel,
                    ticks: to - from,
                });
                app.ui.playlist.time_sel = Some((to, to.saturating_add(to - from)));
            } else {
                app.exec(Command::DuplicateClips(sel));
            }
            select_new(app, &before);
        }
        Act::Delete if !sel.is_empty() => {
            app.exec(Command::RemoveClips(sel));
            app.ui.playlist.selection.clear();
        }
        Act::Delete => {}
        Act::ShiftLeft => mv(app, -snap, 0),
        Act::ShiftRight => mv(app, snap, 0),
        Act::ShiftUp => mv(app, 0, -1),
        Act::ShiftDown => mv(app, 0, 1),
        Act::MakeUnique => {
            let cmds: Vec<Command> = clips
                .iter()
                .filter(|c| sel.contains(&c.id) && c.pattern().is_some())
                .map(|c| Command::MakeClipUnique(c.id))
                .collect();
            if !cmds.is_empty() {
                app.exec(Command::Batch(cmds));
            }
        }
        Act::Mute | Act::Unmute => {
            let cmds: Vec<Command> = sel
                .iter()
                .map(|&clip| Command::SetClipMuted {
                    clip,
                    muted: act == Act::Mute,
                })
                .collect();
            if !cmds.is_empty() {
                app.exec(Command::Batch(cmds));
            }
        }
        Act::InsertSpace | Act::SliceInsertSpace => {
            // The time selection's length at its start; without one, a bar at the position.
            let (at, length) = match app.ui.playlist.time_sel {
                Some((a, b)) if b > a => (a, b - a),
                _ => (position(app), sig.ticks_per_bar()),
            };
            app.exec(Command::InsertSpace {
                at,
                length,
                slice: act == Act::SliceInsertSpace,
            });
        }
        Act::DeleteSpace => match app.ui.playlist.time_sel {
            Some((from, to)) if to > from => {
                app.exec(Command::DeleteSpace { from, to });
                let left = clip_ids(app);
                app.ui.playlist.selection.retain(|id| left.contains(id));
            }
            _ => app.hint("Delete space: make a time selection first (Cmd+drag on the ruler)"),
        },
        Act::MergeClips => {
            if let Some(id) = app.exec(Command::MergeClips(sel)) {
                app.ui.playlist.selection = vec![ClipId(id)];
            }
        }
        Act::GroupTracks | Act::UngroupTracks => {
            let mut tracks = app.ui.playlist.selected_tracks.clone();
            if tracks.is_empty() {
                tracks = clips.iter().filter(|c| sel.contains(&c.id)).map(|c| c.track).collect();
            }
            tracks.sort_unstable();
            tracks.dedup();
            if act == Act::GroupTracks {
                if !tracks.is_empty() {
                    app.exec(Command::SetPlaylistTracksGrouped { tracks, grouped: true });
                }
            } else if !tracks.is_empty() {
                app.exec(Command::SetPlaylistTracksGrouped { tracks, grouped: false });
            }
        }
        Act::CollapseGroupedTracks | Act::ExpandGroupedTracks => {
            let collapsed = act == Act::CollapseGroupedTracks;
            let tracks = &app.project().playlist.tracks;
            let parents: Vec<u32> = (0..tracks.len())
                .filter(|&i| {
                    !tracks[i].grouped_with_above
                        && tracks.get(i + 1).is_some_and(|next| next.grouped_with_above)
                        && tracks[i].group_collapsed != collapsed
                })
                .map(|i| i as u32)
                .collect();
            let cmds: Vec<Command> = parents
                .into_iter()
                .map(|track| Command::SetPlaylistTrackGroupCollapsed { track, collapsed })
                .collect();
            if !cmds.is_empty() {
                app.exec(Command::Batch(cmds));
            }
        }
        Act::QuickQuantizeStarts => {
            let grid = snap.max(1) as Tick;
            let cmds: Vec<Command> = clips
                .iter()
                .filter(|c| sel.is_empty() || sel.contains(&c.id))
                .filter_map(|c| {
                    let target = (c.start + grid / 2) / grid * grid;
                    (target != c.start).then(|| Command::MoveClips {
                        clips: vec![c.id],
                        dt: target as i64 - c.start as i64,
                        dtrack: 0,
                    })
                })
                .collect();
            if !cmds.is_empty() {
                app.exec(Command::Batch(cmds));
            }
        }
        Act::Deselect => {
            app.ui.playlist.selection.clear();
            app.ui.playlist.time_sel = None;
        }
        Act::SelectAll => app.ui.playlist.selection = clip_ids(app),
        Act::SelectSource => {
            let sources: Vec<ClipSource> = clips.iter().filter(|c| sel.contains(&c.id)).map(|c| c.source).collect();
            let source = if sources.is_empty() {
                vec![crate::playlist::place_source(app)]
            } else {
                sources
            };
            app.ui.playlist.selection = clips
                .iter()
                .filter(|c| source.contains(&c.source))
                .map(|c| c.id)
                .collect();
        }
        Act::SelectMuted => app.ui.playlist.selection = clips.iter().filter(|c| c.muted).map(|c| c.id).collect(),
        Act::SelectOverlapping => {
            app.ui.playlist.selection = clips
                .iter()
                .filter(|c| {
                    clips
                        .iter()
                        .any(|m| m.id != c.id && m.track == c.track && m.start < c.end() && c.start < m.end())
                })
                .map(|c| c.id)
                .collect()
        }
        Act::SelectStacked => {
            app.ui.playlist.selection = clips
                .iter()
                .filter(|c| clips.iter().any(|m| m.id != c.id && m.start == c.start))
                .map(|c| c.id)
                .collect()
        }
        Act::InvertSelection => {
            app.ui.playlist.selection = clips.iter().map(|c| c.id).filter(|id| !sel.contains(id)).collect()
        }
        Act::SelectTimeAround => {
            let chosen = clips.iter().filter(|c| sel.contains(&c.id));
            if let (Some(a), Some(b)) = (chosen.clone().map(|c| c.start).min(), chosen.map(Clip::end).max()) {
                app.ui.playlist.time_sel = Some((a, b));
            }
        }
        Act::ZoomIn => app.ui.playlist.zoom_x = (app.ui.playlist.zoom_x * 1.25).min(MAX_ZOOM),
        Act::ZoomOut => app.ui.playlist.zoom_x = (app.ui.playlist.zoom_x / 1.25).max(MIN_ZOOM),
        Act::QuickZoom(n) => quick_zoom(app, n),
        Act::AddMarker => {
            let at = position(app);
            if let Some(id) = crate::playlist::add_marker(app, at) {
                app.dialog = Some(crate::dialogs::Dialog::rename_marker(app, id));
            }
        }
        Act::AddMarkerAuto => {
            let at = position(app);
            crate::playlist::add_marker(app, at);
        }
        Act::DeleteMarkers => {
            // The markers in the time selection, or the one at (or last before) the position.
            let markers = &app.project().playlist.markers;
            let ids: Vec<MarkerId> = match app.ui.playlist.time_sel {
                Some((a, b)) => markers
                    .iter()
                    .filter(|m| (a..b).contains(&m.tick))
                    .map(|m| m.id)
                    .collect(),
                None => {
                    let at = position(app);
                    markers
                        .iter()
                        .rev()
                        .find(|m| m.tick <= at)
                        .map(|m| m.id)
                        .into_iter()
                        .collect()
                }
            };
            let cmds: Vec<Command> = ids.into_iter().map(Command::RemoveMarker).collect();
            if !cmds.is_empty() {
                app.exec(Command::Batch(cmds));
            }
        }
        Act::Center => {
            let Some(grid) = app.ui.playlist.geometry.as_ref().map(|g| g.grid) else {
                return;
            };
            let pos = position(app);
            let st = &mut app.ui.playlist;
            st.scroll.x = (pos as f32 / PPQ as f32 * st.zoom_x - grid.width() / 2.0).max(0.0);
        }
    }
}

pub const MIN_ZOOM: f32 = 0.25;
pub const MAX_ZOOM: f32 = 16_384.0;

fn fit(app: &mut LimeShotApp, from: Tick, to: Tick) {
    let Some(grid) = app.ui.playlist.geometry.as_ref().map(|g| g.grid) else {
        return;
    };
    let beats = to.saturating_sub(from).max(1) as f32 / PPQ as f32;
    let st = &mut app.ui.playlist;
    st.zoom_x = (grid.width() / beats).clamp(MIN_ZOOM, MAX_ZOOM);
    st.scroll.x = from as f32 / PPQ as f32 * st.zoom_x;
}

/// FL Studio's quick zooms (Shift+1 … 5). The levels are LimeShot's: 4, 16 and 64 bars from the
/// view's left edge, the whole song (Zoom out far) and the selection.
fn quick_zoom(app: &mut LimeShotApp, n: u8) {
    let sig = app.project().time_signature;
    let bar = sig.ticks_per_bar();
    let left = {
        let st = &app.ui.playlist;
        let t = (st.scroll.x / st.zoom_x * PPQ as f32).max(0.0) as Tick;
        t / bar * bar
    };
    match n {
        1 => fit(app, left, left + 4 * bar),
        2 => fit(app, left, left + 16 * bar),
        3 => fit(app, left, left + 64 * bar),
        4 => {
            let len = app.project().playlist.song_length(sig).max(bar);
            fit(app, 0, len);
        }
        _ => {
            let sel = app.ui.playlist.selection.clone();
            let clips = &app.project().playlist.clips;
            let chosen = clips.iter().filter(|c| sel.contains(&c.id));
            let span = (chosen.clone().map(|c| c.start).min(), chosen.map(Clip::end).max());
            match (span, app.ui.playlist.time_sel) {
                ((Some(a), Some(b)), _) | (_, Some((a, b))) => fit(app, a, b),
                _ => {}
            }
        }
    }
}

/// The menu's keys, checked before the Playlist's own (egui matches a shortcut with extra Shift
/// or Alt held, so the longer combinations come first).
pub fn keys(app: &mut LimeShotApp, ctx: &Context) {
    use crate::shortcuts::key as k;
    let cmd = Modifiers::COMMAND;
    let shift = Modifiers::SHIFT;
    let alt = Modifiers::ALT;
    for (mods, key, act) in [
        (cmd | shift, Key::V, Act::MakeUnique),
        (alt | shift, Key::M, Act::Unmute),
        (alt, Key::M, Act::Mute),
        (cmd | alt, Key::Insert, Act::SliceInsertSpace),
        (cmd, Key::Insert, Act::InsertSpace),
        (cmd, Key::Delete, Act::DeleteSpace),
        (cmd, Key::Backspace, Act::DeleteSpace),
        (alt, Key::T, Act::AddMarker),
        (cmd, Key::T, Act::AddMarkerAuto),
        (cmd, Key::G, Act::MergeClips),
        (shift, Key::G, Act::GroupTracks),
        (alt, Key::G, Act::UngroupTracks),
        (cmd, Key::Enter, Act::SelectTimeAround),
        (shift, Key::Q, Act::QuickQuantizeStarts),
        (shift, Key::C, Act::SelectSource),
        (Modifiers::NONE, Key::PageUp, Act::ZoomIn),
        (Modifiers::NONE, Key::PageDown, Act::ZoomOut),
    ] {
        if k(ctx, mods, key) {
            run(app, act);
        }
    }
    for (n, digit) in [Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::Num5]
        .into_iter()
        .enumerate()
    {
        if shift_digit(ctx, digit) {
            run(app, Act::QuickZoom(n as u8 + 1));
        }
    }
    if shift_digit(ctx, Key::Num0) {
        run(app, Act::Center);
    }
}
