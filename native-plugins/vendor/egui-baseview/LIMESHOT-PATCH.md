# Local egui-baseview 0.9.0 patch

Source: crates.io egui-baseview 0.9.0, MIT OR Apache-2.0; upstream licenses retained.
The OpenGL renderer change retains the renderer’s GL context and binds it before destroying
its painter. Upstream destroys GL object names in whichever context the host left current,
which can delete the host’s textures/programs and black out its window when an editor closes.
Failed context binding skips explicit deletion; context destruction releases remaining resources.
Remove this patch when an upstream release provides equivalent teardown behavior.

Native modal dialogs can re-enter AppKit event, draw and resize callbacks during `user_app.ui()`.
The backend now defers nested draw/resize (retaining the latest requested size) and lets the modal
panel handle nested input, rather than borrowing the active frame’s RefCell again and aborting.
Live reproduction: Lime Sampler Load button, original panic at window.rs pointer-capture query.
