//! Preserve the host's graphics context across embedded native editor callbacks.
//!
//! A plugin's AppKit/OpenGL timer can clear the thread's current context. Glutin's macOS
//! surface check only compares the context's bound view, so eframe may skip making its
//! context current again. Capture the host context before the first editor attaches and
//! restore it around every host frame; plugin contexts are never substituted for it.
//! On Windows, OpenGL editors call `wglMakeCurrent` on the host's UI thread in the same way.
#![cfg_attr(
    target_os = "macos",
    expect(deprecated, reason = "The host and plugin editors use macOS OpenGL")
)]

#[cfg(target_os = "macos")]
use objc2::{MainThreadMarker, rc::Retained};
#[cfg(target_os = "macos")]
use objc2_app_kit::NSOpenGLContext;
#[cfg(target_os = "windows")]
use windows_sys::Win32::Graphics::{
    Gdi::HDC,
    OpenGL::{HGLRC, wglGetCurrentContext, wglGetCurrentDC, wglMakeCurrent},
};

/// The thread's current WGL device context and rendering context, if any.
#[cfg(target_os = "windows")]
fn current_wgl() -> Option<(HDC, HGLRC)> {
    // SAFETY: plain queries of this thread's current context.
    let (dc, rc) = unsafe { (wglGetCurrentDC(), wglGetCurrentContext()) };
    (!rc.is_null()).then_some((dc, rc))
}

/// The main host viewport's context. Main-thread use only; a no-op on other platforms.
#[derive(Default)]
pub struct HostGraphicsContext {
    #[cfg(target_os = "macos")]
    context: Option<Retained<NSOpenGLContext>>,
    #[cfg(target_os = "windows")]
    context: Option<(HDC, HGLRC)>,
}

/// Restores the host context after UI code that may attach or invoke a native editor.
#[must_use]
pub struct HostGraphicsGuard {
    #[cfg(target_os = "macos")]
    context: Option<Retained<NSOpenGLContext>>,
    #[cfg(target_os = "windows")]
    context: Option<(HDC, HGLRC)>,
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
        #[cfg(target_os = "windows")]
        {
            if self.context.is_none() {
                self.context = current_wgl();
            }
            let guard = HostGraphicsGuard { context: self.context };
            guard.restore();
            guard
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        HostGraphicsGuard {}
    }
}

impl HostGraphicsGuard {
    /// Prevent a plugin teardown from deleting GL names in the host’s current context.
    /// Some editor backends destroy their painter without binding their own context.
    pub(crate) fn suspend() -> Self {
        #[cfg(target_os = "macos")]
        {
            let context = NSOpenGLContext::currentContext();
            NSOpenGLContext::clearCurrentContext();
            Self { context }
        }
        #[cfg(target_os = "windows")]
        {
            let context = current_wgl();
            if context.is_some() {
                // SAFETY: releases this thread's current context; `restore` makes it current again.
                unsafe { wglMakeCurrent(std::ptr::null_mut(), std::ptr::null_mut()) };
            }
            Self { context }
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        Self {}
    }

    /// Restore the host OpenGL context after a plugin editor callback changes it.
    fn restore(&self) {
        #[cfg(target_os = "macos")]
        if let Some(context) = &self.context {
            context.makeCurrentContext();
        }
        #[cfg(target_os = "windows")]
        if let Some((dc, rc)) = self.context
            && current_wgl().is_none_or(|(_, current)| current != rc)
        {
            // SAFETY: the host's own device and rendering contexts, captured on this thread.
            unsafe { wglMakeCurrent(dc, rc) };
        }
    }
}

impl Drop for HostGraphicsGuard {
    /// Restore the host context even when the editor callback exits early.
    fn drop(&mut self) {
        self.restore();
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    use objc2::AnyThread;
    use objc2_app_kit::{NSOpenGLPFAAllowOfflineRenderers, NSOpenGLPixelFormat};

    #[test]
    fn teardown_suspends_and_restores_the_callers_gl_context() {
        let mut attributes = [NSOpenGLPFAAllowOfflineRenderers, 0];
        // SAFETY: the non-null, zero-terminated attributes array lives throughout initialization.
        let format = unsafe {
            NSOpenGLPixelFormat::initWithAttributes(
                NSOpenGLPixelFormat::alloc(),
                std::ptr::NonNull::from(&mut attributes[0]),
            )
        }
        .expect("offscreen GL pixel format");
        let context = NSOpenGLContext::initWithFormat_shareContext(NSOpenGLContext::alloc(), &format, None)
            .expect("offscreen GL context");
        context.makeCurrentContext();
        assert_eq!(NSOpenGLContext::currentContext().as_deref(), Some(context.as_ref()));
        {
            let _guard = HostGraphicsGuard::suspend();
            assert!(
                NSOpenGLContext::currentContext().is_none(),
                "plugin teardown cannot delete host GL names"
            );
        }
        assert_eq!(NSOpenGLContext::currentContext().as_deref(), Some(context.as_ref()));
        NSOpenGLContext::clearCurrentContext();
    }
}
