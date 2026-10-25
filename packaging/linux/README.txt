LimeShot - alpha release (Linux)
================================

LimeShot is an open-source, offline-first DAW with a pattern workflow (Channel Rack, Piano Roll,
Playlist, Mixer). This is an alpha build: save often and expect rough edges.

Run without installing:  ./limeshot            (the Lime VST3s are found in plugins/ next to it)
Install system-wide:     sudo ./install.sh     (/opt/limeshot; Lime VST3s in /usr/local/lib/vst3)
Install for you only:    ./install.sh --user   (~/.local/opt/limeshot; Lime VST3s in ~/.vst3)
Keep VST3s private:      add --private-plugins (other hosts will not see the Lime VST3s)
Uninstall:               run uninstall.sh in the installed folder

Instruments and effects
-----------------------
LimeShot's instruments and effects are the twelve Lime Native VST3 plugins. LimeShot looks for them,
in order, in $LIMESHOT_LIME_PLUGINS, plugins/ next to the binary, ~/.vst3, /usr/lib/vst3 and
/usr/local/lib/vst3.

Requirements
------------
ALSA (libasound2), OpenGL and libxkbcommon; X11 or Wayland. File dialogs use the XDG desktop portal
(xdg-desktop-portal). Built on Ubuntu 22.04; older distributions may lack the required glibc.

Known alpha limitations
-----------------------
  * Plugin editors are not embedded on Linux yet; Lime plugins show their parameters instead.
  * See docs/roadmap/known-limitations.md in the source repository for unfinished features.

Licences: LICENSE, LICENSE-MIT, LICENSE-APACHE, ATTRIBUTION.md and the licenses folder.
Source and issues: https://github.com/fruitapps/limeshot
