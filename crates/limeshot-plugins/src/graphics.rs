//! Preserve the host's graphics context across embedded native editor callbacks.
//!
//! A plugin's AppKit/OpenGL timer can clear the thread's current context. Glutin's macOS
//! surface check only compares the context's bound view, so eframe may skip making its
//! context current again. Capture the host context before the first editor attaches and
//! restore it around every host frame; plugin contexts are never substituted for it.
#![cfg_attr(
    target_os = "macos",
    expect(deprecated, reason = "The host and plugin editors use macOS OpenGL")
)]

#[cfg(target_os = "macos")]
use objc2::{MainThreadMarker, rc::Retained};
#[cfg(target_os = "macos")]
use objc2_app_kit::NSOpenGLContext;

/// The main host viewport's context. Main-thread use only; a no-op on other platforms.
#[derive(Default)]
pub struct HostGraphicsContext {
    #[cfg(target_os = "macos")]
    context: Option<Retained<NSOpenGLContext>>,
}

/// Restores the host context after UI code that may attach or invoke a native editor.
#[must_use]
pub struct HostGraphicsGuard {
    #[cfg(target_os = "macos")]
    context: Option<Retained<NSOpenGLContext>>,
}

impl HostGraphicsContext {
    /// Call at the start of the main host viewport's UI, before attaching any editors.
    pub fn guard(&mut self) -> HostGraphicsGuard {
        #[cfg(target_os = "macos")]
        {
            if MainThreadMarker::new().is_none() {
                return HostGraphicsGuard { context: None };
            }
            if self.context.is_none() {
                self.context = NSOpenGLContext::currentContext();
            }
            let guard = HostGraphicsGuard {
                context: self.context.clone(),
            };
            guard.restore();
            guard
        }
        #[cfg(not(target_os = "macos"))]
        HostGraphicsGuard {}
    }
}

impl HostGraphicsGuard {
    fn restore(&self) {
        #[cfg(target_os = "macos")]
        if let Some(context) = &self.context {
            context.makeCurrentContext();
        }
    }
}

impl Drop for HostGraphicsGuard {
    fn drop(&mut self) {
        self.restore();
    }
}
