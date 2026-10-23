//! The Channel Settings window: the selected channel's instrument parameters (FL Studio opens
//! this window when you click a channel button in the Channel Rack).

use crate::app::LimeShotApp;
use crate::theme::{Theme, mono, rgb, semibold};
use crate::widgets::Hint as _;
use egui::{CornerRadius, DragValue, RichText, Sense, Ui, vec2};
use limeshot_core::*;

/// Body of the Channel Settings window for the selected channel.
pub fn show(app: &mut LimeShotApp, ui: &mut Ui) {
    page(app, ui, app.selected_channel());
}

/// The Channel Settings of channel `cid` (also the Settings tab of a plugin channel's window).
pub fn channel_page(app: &mut LimeShotApp, ui: &mut Ui, cid: ChannelId) {
    page(app, ui, Some(cid));
}

fn page(app: &mut LimeShotApp, ui: &mut Ui, cid: Option<ChannelId>) {
    let t = Theme::current();
    let Some(ch) = cid.and_then(|c| app.project().channel(c).cloned()) else {
        ui.add_space(16.0);
        ui.vertical_centered(|ui| ui.label(RichText::new("Add a channel to edit its sound.").color(t.text_dim)));
        return;
    };
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        egui::Frame::new().inner_margin(egui::Margin::same(12)).show(ui, |ui| {
            ui.horizontal(|ui| {
                let (r, _) = ui.allocate_exact_size(vec2(12.0, 12.0), Sense::hover());
                ui.painter().rect_filled(r, CornerRadius::same(3), rgb(ch.color));
                let name = ui
                    .add(egui::Label::new(RichText::new(&ch.name).font(semibold(15.0))).sense(Sense::click()))
                    .hint("Click to rename");
                if name.clicked() {
                    app.dialog = Some(crate::dialogs::Dialog::rename_channel(app, ch.id));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!(
                            "→ Mixer {}",
                            if ch.mixer_track == 0 {
                                "M".to_string()
                            } else {
                                ch.mixer_track.to_string()
                            }
                        ))
                        .font(mono(10.5))
                        .color(t.text_dim),
                    );
                });
            });
            ui.add_space(8.0);
            channel_body(app, ui, &ch);
        });
    });
}

fn channel_body(app: &mut LimeShotApp, ui: &mut Ui, ch: &Channel) {
    let t = Theme::current();
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let mut key = ch.root_key;
            let r = ui.add(
                DragValue::new(&mut key)
                    .range(0..=127)
                    .custom_formatter(|v, _| crate::piano_roll::key_name(v as u8)),
            );
            if crate::widgets::user_changed(&r) {
                app.exec(Command::SetChannelRootKey { channel: ch.id, key });
            }
            ui.label(RichText::new("Root").color(t.text_dim));
            crate::presets_ui::menu(app, ui, crate::presets_ui::Target::Channel(ch.id));
        });
    });
    ui.add_space(6.0);
    match &ch.instrument {
        Instrument::Sampler(_) | Instrument::Synth(_) => {
            ui.label(RichText::new("This legacy instrument needs conversion to a Lime plugin.").color(t.warning));
        }
        Instrument::Plugin(p) => {
            if let Some(params) = lime::sampler_params(&ch.instrument) {
                sampler_source(app, ui, ch, &params);
            }
            crate::plugins::channel_settings(app, ui, ch.id, p);
        }
        Instrument::Unsupported(_) => {
            ui.label(RichText::new("This channel uses an instrument this version of LimeShot does not support. It is kept in the project but stays silent.").color(t.warning));
        }
    }
}

/// Host file actions remain available beside the plugin's own sound controls.
fn sampler_source(app: &mut LimeShotApp, ui: &mut Ui, ch: &Channel, params: &SamplerParams) {
    ui.horizontal(|ui| {
        ui.label(
            params
                .sample
                .as_ref()
                .map(|s| crate::browser::file_label(std::path::Path::new(&s.path)))
                .unwrap_or_else(|| {
                    if app.sampler_source(ch.id).is_some() {
                        "Embedded sample".into()
                    } else {
                        "No sample".into()
                    }
                }),
        );
        if ui.button("Load…").clicked()
            && let Some(path) = rfd::FileDialog::new()
                .add_filter("Audio files", &limeshot_dsp::AUDIO_EXTENSIONS)
                .pick_file()
        {
            crate::browser::assign_sample(app, ch.id, &path);
        }
    });
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 64.0), Sense::click_and_drag());
    ui.painter().rect_filled(rect, 3.0, Theme::current().sunken);
    if let Some(path) = app.sampler_source(ch.id) {
        crate::browser::paint_sample_wave(app, ui.painter(), rect, &path, Theme::current().accent);
        if response.drag_started() {
            if let Some(sample) = &params.sample
                && std::path::Path::new(&sample.path).is_file()
            {
                response.dnd_set_drag_payload(crate::browser::DraggedSample(app.samples.resolve(&sample.path)));
            } else {
                response.dnd_set_drag_payload(ClipSource::Audio(ch.id));
            }
        }
        if response.is_pointer_button_down_on()
            && ui.input(|i| i.pointer.primary_pressed())
            && let Some(limeshot_audio::assets::SampleStatus::Ready(sample)) = app.samples.status_of_path(&path)
        {
            let duration = sample.duration_secs() * f64::from((params.end - params.start).max(0.0))
                / 2.0f64.powf(f64::from(params.pitch) / 12.0);
            app.preview_for(ch.id, ch.root_key, 0.78, duration);
            app.sampler_preview = Some((ch.id, std::time::Instant::now(), duration));
        }
    }
}
