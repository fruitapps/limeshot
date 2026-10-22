//! A plugin's own editor (`IPlugView`), shown in a window of the host's.
//!
//! The host makes the window (LimeShot: an egui viewport, so the window also has LimeShot's own
//! tabs above the editor) and finds it with [`NativeWindow::find`]. [`Vst3Editor::attach`] puts a
//! container view into the window's content view, `top_inset` points below the top, and hands it
//! to the plugin. When the plugin asks for another size (`IPlugFrame::resizeView`), the container
//! and the window are resized at once, so the window always fits the editor.
//!
//! Everything here runs on the main thread, as VST3 editors require. Only macOS (`NSView`) has a
//! native implementation so far; on other platforms [`NativeWindow::find`] finds nothing and the
//! host shows its own parameter list instead. The platform module is the only part to add for
//! Windows (`HWND`, a child window) and Linux (`X11EmbedWindowID`, plus `IRunLoop` on the frame).

use std::cell::RefCell;
use std::sync::Arc;

use vst3::Steinberg::*;
use vst3::{Class, ComPtr, ComRef, ComWrapper};

use crate::PluginError;

/// The platform type string VST3 uses for this platform's views.
#[cfg(target_os = "macos")]
const PLATFORM_TYPE: FIDString = kPlatformTypeNSView;
#[cfg(target_os = "windows")]
const PLATFORM_TYPE: FIDString = kPlatformTypeHWND;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const PLATFORM_TYPE: FIDString = kPlatformTypeX11EmbedWindowID;

/// An editor's size in the platform's view units (points on macOS).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditorSize {
    pub width: u32,
    pub height: u32,
}

impl EditorSize {
    fn from_rect(r: &ViewRect) -> Option<Self> {
        let (w, h) = (r.right - r.left, r.bottom - r.top);
        (w > 0 && h > 0).then_some(Self {
            width: w as u32,
            height: h as u32,
        })
    }

    fn rect(self) -> ViewRect {
        ViewRect {
            left: 0,
            top: 0,
            right: self.width.min(i32::MAX as u32) as i32,
            bottom: self.height.min(i32::MAX as u32) as i32,
        }
    }
}

struct FrameState {
    /// The view, so `resizeView` can answer with `onSize` (not owned; `Vst3Editor` owns it).
    view: *mut IPlugView,
    container: Option<platform::Container>,
    top_inset: f64,
    size: EditorSize,
    /// The host's last embedded size request, before the plugin applies its constraints.
    embedded_request: EditorSize,
    /// The plugin resized itself since the host last asked.
    resized: bool,
}

/// The host's `IPlugFrame`: the plugin calls it to resize its editor.
struct PlugFrame {
    state: RefCell<FrameState>,
}

impl Class for PlugFrame {
    type Interfaces = (IPlugFrame,);
}

impl IPlugFrameTrait for PlugFrame {
    unsafe fn resizeView(&self, _view: *mut IPlugView, new_size: *mut ViewRect) -> tresult {
        if new_size.is_null() {
            return kInvalidArgument;
        }
        // SAFETY: checked non-null; the plugin passes a rectangle it owns for the call.
        let Some(size) = EditorSize::from_rect(unsafe { &*new_size }) else {
            return kInvalidArgument;
        };
        // The borrow ends before `onSize`, which may call back into the frame.
        let view = {
            let Ok(mut s) = self.state.try_borrow_mut() else {
                return kResultFalse;
            };
            s.size = size;
            s.resized = true;
            let inset = s.top_inset;
            if let Some(c) = &s.container {
                c.resize(size, inset);
            }
            s.view
        };
        // SAFETY: `view` is the live view this frame was given to (cleared before it is
        // released); `r` lives for the call.
        unsafe {
            if let Some(v) = ComRef::from_raw(view) {
                let mut r = size.rect();
                v.onSize(&mut r);
            }
        }
        kResultOk
    }
}

/// A window the host made, for [`Vst3Editor::attach`].
pub struct NativeWindow(platform::Window);

impl NativeWindow {
    /// The application's window titled `title` (the newest if several are). Main thread only.
    pub fn find(title: &str) -> Option<Self> {
        platform::Window::find(title).map(Self)
    }

    /// Make it a tool window, as FL Studio's plugin windows are: it floats above the main window
    /// and hides while another application is active.
    pub fn make_tool_window(&self) {
        self.0.make_tool_window();
    }
}

/// A plugin's editor. Create it with [`Vst3Instance::create_editor`](crate::Vst3Instance::create_editor),
/// attach it to a window, and drop it (or [`detach`](Self::detach)) before the window closes.
/// Main thread only.
pub struct Vst3Editor {
    view: ComPtr<IPlugView>,
    frame: ComWrapper<PlugFrame>,
    attached: bool,
    builtin: bool,
    /// Keeps the plugin alive while its editor is open.
    _keep: Arc<dyn Send + Sync>,
}

impl Vst3Editor {
    /// Wrap a view the controller created. `keep` holds the plugin.
    ///
    /// # Safety
    /// `view` must be a live `IPlugView` of a plugin that `keep` keeps alive.
    pub(crate) unsafe fn new(view: ComPtr<IPlugView>, keep: Arc<dyn Send + Sync>, builtin: bool) -> Option<Self> {
        // SAFETY: plain queries and `setFrame` on the live view (function contract).
        unsafe {
            if view.isPlatformTypeSupported(PLATFORM_TYPE) != kResultTrue {
                return None;
            }
            let mut r = ViewRect {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            view.getSize(&mut r);
            let size = EditorSize::from_rect(&r).unwrap_or(EditorSize {
                width: 400,
                height: 300,
            });
            let frame = ComWrapper::new(PlugFrame {
                state: RefCell::new(FrameState {
                    view: view.as_ptr(),
                    container: None,
                    top_inset: 0.0,
                    size,
                    embedded_request: size,
                    resized: false,
                }),
            });
            let frame_ptr = frame.as_com_ref::<IPlugFrame>()?.as_ptr();
            view.setFrame(frame_ptr);
            Some(Self {
                view,
                frame,
                attached: false,
                builtin,
                _keep: keep,
            })
        }
    }

    /// The editor's current size.
    pub fn size(&self) -> EditorSize {
        self.frame.state.borrow().size
    }

    /// Whether the user may resize the editor (the plugin's `canResize`).
    pub fn can_resize(&self) -> bool {
        // SAFETY: a plain query on the live view.
        unsafe { self.view.canResize() == kResultTrue }
    }

    pub fn is_attached(&self) -> bool {
        self.attached
    }

    /// Put the editor into `window`, `top_inset` points below the top of its content, and resize
    /// the window to fit it.
    pub fn attach(&mut self, window: &NativeWindow, top_inset: f64) -> Result<(), PluginError> {
        if self.attached {
            return Ok(());
        }
        let size = self.size();
        let container = platform::Container::new(&window.0, size, top_inset)?;
        container.resize(size, top_inset);
        // SAFETY: the container's native view stays alive until after `removed` (see `detach`).
        let r = unsafe { self.view.attached(container.parent(), PLATFORM_TYPE) };
        if r != kResultOk {
            return Err(PluginError::Call("open its editor"));
        }
        if let Ok(mut s) = self.frame.state.try_borrow_mut() {
            s.container = Some(container);
            s.top_inset = top_inset;
        }
        self.attached = true;
        Ok(())
    }

    /// Attach as a child of the main window without changing the OS window's size.
    pub fn attach_embedded(&mut self, window: &NativeWindow) -> Result<(), PluginError> {
        if self.attached {
            return Ok(());
        }
        let container = platform::Container::new(&window.0, self.size(), 0.0)?;
        container.embed();
        // SAFETY: the container stays alive until after the editor is removed in detach.
        if unsafe { self.view.attached(container.parent(), PLATFORM_TYPE) } != kResultOk {
            return Err(PluginError::Call("open its embedded editor"));
        }
        if let Ok(mut state) = self.frame.state.try_borrow_mut() {
            state.container = Some(container);
        }
        self.attached = true;
        Ok(())
    }

    /// Position a native child view in the main window's content coordinates. Resize-capable
    /// plugins receive onSize; fixed-size plugins retain their natural dimensions.
    pub fn layout_embedded(&self, x: f64, y: f64, width: f64, height: f64) {
        let requested = EditorSize {
            width: width.max(1.0) as u32,
            height: height.max(1.0) as u32,
        };
        if self.can_resize() && self.frame.state.borrow().embedded_request != requested {
            // Cache the request before calling the plugin, which may constrain it or resize itself.
            self.frame.state.borrow_mut().embedded_request = requested;
            let mut rect = requested.rect();
            // SAFETY: plain size negotiation with a live plugin; the rectangle lives for both calls.
            unsafe {
                self.view.checkSizeConstraint(&mut rect);
                self.view.onSize(&mut rect);
            }
            if let Some(size) = EditorSize::from_rect(&rect)
                && let Ok(mut state) = self.frame.state.try_borrow_mut()
            {
                state.size = size;
            }
        }
        if let Ok(state) = self.frame.state.try_borrow()
            && let Some(c) = &state.container
        {
            c.place_rect(x, y, width, height, state.size);
        }
    }

    /// Attach without a window. Only LimeShot's built-in test plugins accept this (headless tests).
    pub fn attach_headless(&mut self) -> Result<(), PluginError> {
        if !self.builtin {
            return Err(PluginError::Unsupported("an editor without a window"));
        }
        // SAFETY: the built-in test plugins never dereference the parent.
        if unsafe { self.view.attached(std::ptr::null_mut(), PLATFORM_TYPE) } != kResultOk {
            return Err(PluginError::Call("open its editor"));
        }
        self.attached = true;
        Ok(())
    }

    /// Keep the editor where it belongs after the window changed size (or the host's tabs moved).
    pub fn layout(&self, top_inset: f64) {
        if let Ok(mut s) = self.frame.state.try_borrow_mut() {
            s.top_inset = top_inset;
            let size = s.size;
            if let Some(c) = &s.container {
                c.place(size, top_inset);
            }
        }
    }

    /// Show or hide the editor (the host's other tab is showing).
    pub fn set_visible(&self, visible: bool) {
        if let Ok(s) = self.frame.state.try_borrow()
            && let Some(c) = &s.container
        {
            c.set_hidden(!visible);
        }
    }

    /// The editor's new size if the plugin resized it since the last call.
    pub fn take_resized(&mut self) -> Option<EditorSize> {
        let mut s = self.frame.state.try_borrow_mut().ok()?;
        std::mem::take(&mut s.resized).then_some(s.size)
    }

    /// Forward a key press to the view (`IPlugView::onKeyDown`), as hosts do where the platform
    /// does not deliver keys to the view itself.
    pub fn key_down(&self, key: char) -> bool {
        let mut buf = [0u16; 2];
        let Some(&k) = key.encode_utf16(&mut buf).first() else {
            return false;
        };
        // SAFETY: a plain call on the live view.
        unsafe { self.view.onKeyDown(k, 0, 0) == kResultTrue }
    }

    /// Take the editor out of its window. Called by `Drop` too.
    pub fn detach(&mut self) {
        if self.attached {
            // SAFETY: `removed` on the attached view, before its parent goes.
            unsafe { self.view.removed() };
            self.attached = false;
        }
        if let Ok(mut s) = self.frame.state.try_borrow_mut() {
            s.container = None;
        }
    }
}

impl Drop for Vst3Editor {
    fn drop(&mut self) {
        self.detach();
        if let Ok(mut s) = self.frame.state.try_borrow_mut() {
            s.view = std::ptr::null_mut();
        }
        // SAFETY: the view is still alive; it lets go of the frame before it is released.
        unsafe { self.view.setFrame(std::ptr::null_mut()) };
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use super::EditorSize;
    use crate::PluginError;
    use objc2::rc::Retained;
    use objc2::{MainThreadMarker, MainThreadOnly};
    use objc2_app_kit::{NSApplication, NSFloatingWindowLevel, NSView, NSWindow};
    use objc2_foundation::{NSPoint, NSRect, NSSize};

    pub(super) struct Window(Retained<NSWindow>);

    impl Window {
        /// Above the main window, hidden while another application is active.
        pub(super) fn make_tool_window(&self) {
            self.0.setLevel(NSFloatingWindowLevel);
            self.0.setHidesOnDeactivate(true);
        }

        pub(super) fn find(title: &str) -> Option<Self> {
            let mtm = MainThreadMarker::new()?;
            let app = NSApplication::sharedApplication(mtm);
            let windows = app.windows();
            let all: Vec<Retained<NSWindow>> = (0..windows.count()).map(|i| windows.objectAtIndex(i)).collect();
            all.into_iter().rev().find(|w| w.title().to_string() == title).map(Self)
        }
    }

    /// A plain view in the host's window that the plugin's view goes into.
    pub(super) struct Container {
        view: Retained<NSView>,
        window: Retained<NSWindow>,
        embedded: std::cell::Cell<bool>,
    }

    impl Container {
        pub(super) fn new(window: &Window, size: EditorSize, top_inset: f64) -> Result<Self, PluginError> {
            let mtm = MainThreadMarker::new().ok_or(PluginError::Unsupported("editors off the main thread"))?;
            let window = window.0.clone();
            let content = window
                .contentView()
                .ok_or(PluginError::Unsupported("a window without a content view"))?;
            let frame = NSRect::new(
                NSPoint::new(0.0, 0.0),
                NSSize::new(size.width as f64, size.height as f64),
            );
            let view = NSView::initWithFrame(NSView::alloc(mtm), frame);
            content.addSubview(&view);
            let c = Self {
                view,
                window,
                embedded: std::cell::Cell::new(false),
            };
            c.place(size, top_inset);
            Ok(c)
        }

        pub(super) fn parent(&self) -> *mut std::ffi::c_void {
            Retained::as_ptr(&self.view) as *mut std::ffi::c_void
        }

        /// Put the container at the top left of the content view, below the inset.
        pub(super) fn place(&self, size: EditorSize, top_inset: f64) {
            let Some(content) = self.window.contentView() else {
                return;
            };
            let (w, h) = (size.width as f64, size.height as f64);
            let y = if content.isFlipped() {
                top_inset
            } else {
                content.bounds().size.height - top_inset - h
            };
            self.view.setFrame(NSRect::new(NSPoint::new(0.0, y), NSSize::new(w, h)));
        }

        /// Resize the window so its content is the editor plus the inset, keeping its top left.
        pub(super) fn resize(&self, size: EditorSize, top_inset: f64) {
            if self.embedded.get() {
                return;
            }
            let old = self.window.frame();
            let content = NSRect::new(
                old.origin,
                NSSize::new(size.width as f64, size.height as f64 + top_inset),
            );
            let mut new = self.window.frameRectForContentRect(content);
            new.origin.y = old.origin.y + old.size.height - new.size.height;
            if new != old {
                self.window.setFrame_display(new, true);
            }
            self.place(size, top_inset);
        }

        pub(super) fn embed(&self) {
            self.embedded.set(true);
        }

        pub(super) fn place_rect(&self, x: f64, y: f64, _width: f64, _height: f64, natural: EditorSize) {
            let Some(content) = self.window.contentView() else {
                return;
            };
            let width = f64::from(natural.width);
            let height = f64::from(natural.height);
            let title_inset = (content.bounds().size.height - self.window.contentLayoutRect().size.height).max(0.0);
            let top = y + title_inset;
            let y = if content.isFlipped() {
                top
            } else {
                content.bounds().size.height - top - height
            };
            self.view
                .setFrame(NSRect::new(NSPoint::new(x, y), NSSize::new(width, height)));
        }

        pub(super) fn set_hidden(&self, hidden: bool) {
            self.view.setHidden(hidden);
        }
    }

    impl Drop for Container {
        fn drop(&mut self) {
            self.view.removeFromSuperview();
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod platform {
    use super::EditorSize;
    use crate::PluginError;

    pub(super) struct Window;

    impl Window {
        pub(super) fn make_tool_window(&self) {}

        pub(super) fn find(_title: &str) -> Option<Self> {
            None
        }
    }

    pub(super) struct Container;

    impl Container {
        pub(super) fn new(_window: &Window, _size: EditorSize, _top_inset: f64) -> Result<Self, PluginError> {
            Err(PluginError::Unsupported("plugin editors on this platform yet"))
        }
        pub(super) fn parent(&self) -> *mut std::ffi::c_void {
            std::ptr::null_mut()
        }
        pub(super) fn place(&self, _size: EditorSize, _top_inset: f64) {}
        pub(super) fn resize(&self, _size: EditorSize, _top_inset: f64) {}
        pub(super) fn embed(&self) {}
        pub(super) fn place_rect(&self, _x: f64, _y: f64, _width: f64, _height: f64, _natural: EditorSize) {}
        pub(super) fn set_hidden(&self, _hidden: bool) {}
    }
}
