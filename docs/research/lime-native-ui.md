# Lime Native editor design references — 2026-10-22

Owner request: make the standalone Lime plugins visually closer to FL Studio's native lineup, distinct from each
other, and make Sampler resemble LimeShot's built-in sampler as well as FL Studio's Channel Sampler.

Reviewed Image-Line's current online manual, parameter descriptions and the interface images linked by those
pages. Reference screenshots were inspected from a temporary research directory; no Image-Line bitmap, icon,
font, source code or other bundle resource is included in Lime Native. Layout and control artwork are original
painter code. The existing DSP, parameter IDs, class IDs and preset/state schemas are retained.

| Lime editor | Official reference | Design decisions |
| --- | --- | --- |
| LimeSynth | [3x Osc](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/plugins/3x%20Osc.htm) | Waveform selectors, horizontal tuning bank, compact hardware dials; separate envelope/filter/LFO panels appropriate to LimeSynth's actual architecture. LimeSynth is not presented as three independent oscillators. |
| Lime Sampler | [Sampler Channel Settings](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/chansettings_sampler.htm) and `crates/limeshot-ui/src/inspector.rs::sampler_settings` | File bar with load/clear, playback/trim/release controls, reverse and one-shot switches, large lower waveform, colored start/end handles and root-key keyboard. The waveform and keyboard audition the loaded sample; right-click on a key sets root. No nonfunctional stretching/loop/effect controls. |
| Lime Utility | [Fruity Balance](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/plugins/Fruity%20Balance.htm), [Stereo Enhancer](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/plugins/Fruity%20Stereo%20Enhancer.htm) | Small panel, two large volume/width dials, polarity selector and stereo metering. No panning/phase-offset controls, since Utility does not implement them. |
| Lime EQ | [Parametric EQ 2](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/plugins/Fruity%20Parametric%20EQ%202.htm) | Dominant frequency graph with three colored, draggable nodes; three band gain faders; frequency/Q controls below. Curve is calculated by LimeShot's actual Eq3 response function, rather than a simulated spectrum. Exactly three supported bands. |
| Lime Filter | [Fruity Filter](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/plugins/Fruity%20Filter.htm) | Compact labeled control tiles, mode selector, cutoff/resonance and wet/dry; mint response guide. No oversampling/DC switches. |
| Lime Delay | [Fruity Delay 3](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/plugins/Fruity%20Delay%203.htm) | Distinct time, feedback/stereo and output sections; amber/teal accents, large time dial, tempo-division strip and wet/dry fader. Echo diagram follows feedback/mix settings; it is not an audio analyzer. |
| Lime Room | [Fruity Reeverb 2](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/plugins/Fruity%20Reeverb%202.htm) | Lavender wireframe acoustic space, 2×2 parameter bank and wet/dry fader. Original rectangular chamber drawing responds to size/width/damping rather than copying FL's cylinder artwork. |
| Lime Comp | [Fruity Compressor](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/plugins/Fruity%20Compressor.htm) | Compact 2×3 control/readout tiles with pink accents; sidechain detector selector, transfer graph, output scope and meters. |
| Lime Limiter | [Fruity Limiter](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/plugins/Fruity%20Limiter.htm) | Wide live input/output/gain-reduction history, grouped loudness/envelope controls, metering and transfer curve. Separate limiter identity; no inactive compressor/gate tabs. |

After verifying the FL-inspired layout revision (11 native tests, Clippy and all nine inspected editor renders),
the owner requested a cleaner, modern finish matching LimeShot itself. The final shared language uses the
application's Lime Dark neutral colors, flat rotary controls with thin value arcs, quiet inline section labels,
subtle rounded panels, a compact wrapper header, lime brand marker and plugin-specific accents. Plugin window sizes and proportions differ by purpose.
All controls call host automation setters; graphical EQ nodes and waveform trim handles edit the same parameters
as their numeric controls. Sample audition crosses to the audio thread through an atomic note request; no sample
loading or UI rendering runs on the audio thread.
