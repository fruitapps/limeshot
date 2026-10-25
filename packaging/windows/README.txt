LimeShot - alpha release
========================

LimeShot is an open-source, offline-first DAW with a pattern workflow (Channel Rack, Piano Roll,
Playlist, Mixer). This is an alpha build: save often and expect rough edges.

Start:     limeshot.exe (or the LimeShot Start menu entry); limeshot.exe song.lime opens a project.
Headless:  limeshot-cli.exe demo out.lime --render out.wav

Instruments and effects
-----------------------
LimeShot's instruments and effects are the twelve Lime Native VST3 plugins (LimeSynth, Lime Sampler,
Lime EQ, ...). LimeShot looks for them, in order, in:
  * the folder named by the LIMESHOT_LIME_PLUGINS environment variable,
  * plugins\ next to limeshot.exe (portable ZIP, or installer with the shared VST3 option unticked),
  * C:\Program Files\Common Files\VST3 and %LOCALAPPDATA%\Programs\Common\VST3 (installer default).
Plugins in the shared VST3 folders can also be loaded by other VST3 hosts.

Known alpha limitations
-----------------------
  * The executables are not code-signed; Windows SmartScreen may warn on first start
    ("More info" > "Run anyway").
  * See docs/roadmap/known-limitations.md in the source repository for unfinished features.

Licences: LICENSE, LICENSE-MIT, LICENSE-APACHE, ATTRIBUTION.md and the licenses folder.
Source and issues: https://github.com/fruitapps/limeshot
