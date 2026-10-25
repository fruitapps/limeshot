# Local egui-baseview 0.9.0 patch

Source: crates.io egui-baseview 0.9.0, MIT OR Apache-2.0; upstream licenses retained.
The only source change retains the renderer’s GL context and binds it before destroying
its painter. Upstream destroys GL object names in whichever context the host left current,
which can delete the host’s textures/programs and black out its window when an editor closes.
Failed context binding skips explicit deletion; context destruction releases remaining resources.
Remove this patch when an upstream release provides equivalent teardown behavior.
