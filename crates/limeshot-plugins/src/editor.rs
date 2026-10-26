//! A plugin's own editor (`IPlugView`), shown in a window of the host's.
//!
//! The host makes the window (LimeShot: an egui viewport, so the window also has LimeShot's own
//! tabs above the editor) and finds it with [`NativeWindow::find`]. [`Vst3Editor::attach`] puts a
//! container view into the window's content view, `top_inset` points below the top, and hands it
//! to the plugin. When the plugin asks for another size (`IPlugFrame::resizeView`), the container
//! and the window are resized at once, so the window always fits the editor.
//!
//! Everything here runs on the main thread, as VST3 editors require. macOS (`NSView`) and Windows
//! (`HWND`, an owned popup over the editor area) have native implementations; elsewhere
//! [`NativeWindow::find`] finds nothing and the host shows its own parameter list instead. Linux
//! needs only its platform module (`X11EmbedWindowID`, plus `IRunLoop` on the frame).

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
    host_resizing: bool,
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
            s.resized |= s.size != size && !s.host_resizing;
            s.size = size;
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

    /// Show a host menu above native plugin views. Coordinates are from the content layout’s
    /// top left, in platform points. Returns the chosen entry, or None on cancel/unavailable.
    /// Current pointer from the OS (AppKit, Win32), including OS drags which do not send ordinary
    /// mouse moves.
    pub fn pointer_position(&self) -> Option<(f64, f64)> {
        self.0.pointer_position()
    }

    pub fn popup_menu(&self, labels: &[String], x: f64, y: f64) -> Option<usize> {
        self.0.popup_menu(labels, x, y)
    }

    /// Make it a tool window, as FL Studio's plugin windows are: it floats above the main window
    /// and hides while another application is active.
    pub fn make_tool_window(&self) {
        self.0.make_tool_window();
    }
}

/// Let the embedded editors catch up on timers and repaints. Call once per host frame, after the
/// editors were placed, while the host's graphics context is guarded. On Windows a host that
/// repaints every frame keeps its own `WM_PAINT` pending, which the message loop always returns
/// before the lower-priority `WM_TIMER`, so editors that redraw on a timer (baseview, JUCE) would
/// otherwise never update. Does nothing on other platforms.
pub fn service_editors() {
    platform::service();
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
                    host_resizing: false,
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
    /// Does nothing if already attached.
    ///
    /// Returns `PluginError::Unsupported` if a native container cannot be created (including off
    /// macOS), or `PluginError::Call` if the plugin rejects attachment.
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
        log::info!("embedded plugin editor {:?}: {}", self.size(), container.describe());
        if let Ok(mut state) = self.frame.state.try_borrow_mut() {
            state.container = Some(container);
        }
        self.attached = true;
        Ok(())
    }

    /// Position a native child view from the top left of the main window's content layout area,
    /// in platform view units (points on macOS). Resize-capable plugins receive a size request
    /// clamped to at least one unit per dimension, subject to their constraints; fixed-size
    /// plugins retain their natural dimensions.
    pub fn layout_embedded(&self, x: f64, y: f64, width: f64, height: f64) {
        let requested = EditorSize {
            width: width.max(1.0) as u32,
            height: height.max(1.0) as u32,
        };
        if self.can_resize() && self.frame.state.borrow().embedded_request != requested {
            // Cache the request before calling the plugin, which may constrain it or resize itself.
            {
                let mut state = self.frame.state.borrow_mut();
                state.embedded_request = requested;
                state.host_resizing = true;
            }
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
            self.frame.state.borrow_mut().host_resizing = false;
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

    /// Whether [`set_cutouts`](Self::set_cutouts) works here, so a partly covered editor can stay
    /// visible. Otherwise the host should hide an editor that its own windows overlap.
    pub fn can_cut_out() -> bool {
        platform::CAN_CUT_OUT
    }

    /// Host content drawn above the editor: rectangles `(x, y, width, height)` in the units and
    /// origin of [`layout_embedded`](Self::layout_embedded), left out of the editor's visible area.
    pub fn set_cutouts(&self, rects: &[(f64, f64, f64, f64)]) {
        if let Ok(s) = self.frame.state.try_borrow()
            && let Some(c) = &s.container
        {
            c.set_cutouts(rects);
        }
    }

    /// Let host drags reach the underlying workspace without hiding the native editor.
    pub fn set_input_passthrough(&self, passthrough: bool) {
        if let Ok(s) = self.frame.state.try_borrow()
            && let Some(c) = &s.container
        {
            c.set_input_passthrough(passthrough);
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
        let _graphics = crate::graphics::HostGraphicsGuard::suspend();
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
    /// AppKit runs timers and display on its own run loop.
    pub(super) fn service() {}

    pub(super) const CAN_CUT_OUT: bool = false;

    use super::EditorSize;
    use crate::PluginError;
    use objc2::rc::Retained;
    use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
    use objc2_app_kit::{NSApplication, NSFloatingWindowLevel, NSMenu, NSMenuItem, NSView, NSWindow};
    use objc2_foundation::{NSObject, NSObjectProtocol, NSPoint, NSRect, NSSize, NSString};

    pub(super) struct Window(Retained<NSWindow>);

    impl Window {
        pub(super) fn popup_menu(&self, labels: &[String], x: f64, y: f64) -> Option<usize> {
            let mtm = MainThreadMarker::new()?;
            let content = self.0.contentView()?;
            let target = PresetMenuTarget::alloc(mtm).set_ivars(MenuResult::default());
            // SAFETY: initialize an allocated NSObject subclass with its superclass initializer.
            let target: Retained<PresetMenuTarget> = unsafe { msg_send![super(target), init] };
            let menu = NSMenu::initWithTitle(NSMenu::alloc(mtm), &NSString::from_str("Presets"));
            menu.setAutoenablesItems(false);
            for (index, label) in labels.iter().enumerate() {
                // SAFETY: target implements selectPreset: with NSMenuItem’s action signature,
                // and remains retained throughout the synchronous menu tracking call below.
                let item = unsafe {
                    menu.addItemWithTitle_action_keyEquivalent(
                        &NSString::from_str(label),
                        Some(sel!(selectPreset:)),
                        &NSString::new(),
                    )
                };
                item.setTag(isize::try_from(index).ok()?);
                // SAFETY: the target has the action implementation and outlives menu tracking.
                unsafe { item.setTarget(Some(&target)) };
            }
            let inset = (content.bounds().size.height - self.0.contentLayoutRect().size.height).max(0.0);
            let top = y + inset;
            let point = NSPoint::new(
                x,
                if content.isFlipped() {
                    top
                } else {
                    content.bounds().size.height - top
                },
            );
            menu.popUpMenuPositioningItem_atLocation_inView(None, point, Some(&content));
            target.ivars().selected.get()
        }

        pub(super) fn pointer_position(&self) -> Option<(f64, f64)> {
            let content = self.0.contentView()?;
            let point = content.convertPoint_fromView(self.0.mouseLocationOutsideOfEventStream(), None);
            let inset = (content.bounds().size.height - self.0.contentLayoutRect().size.height).max(0.0);
            let y = if content.isFlipped() {
                point.y
            } else {
                content.bounds().size.height - point.y
            } - inset;
            Some((point.x, y))
        }

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

    #[derive(Default)]
    struct MenuResult {
        selected: std::cell::Cell<Option<usize>>,
    }

    define_class!(
        // SAFETY: NSObject imposes no subclassing requirements; this class has no Drop.
        #[unsafe(super = NSObject)]
        #[thread_kind = MainThreadOnly]
        #[ivars = MenuResult]
        struct PresetMenuTarget;

        // SAFETY: NSObjectProtocol has no additional requirements.
        unsafe impl NSObjectProtocol for PresetMenuTarget {}

        impl PresetMenuTarget {
            // SAFETY: matches NSMenuItem’s target/action signature (sender: id).
            #[unsafe(method(selectPreset:))]
            fn select_preset(&self, sender: &NSMenuItem) {
                self.ivars().selected.set(usize::try_from(sender.tag()).ok());
            }
        }
    );

    #[derive(Default)]
    struct ContainerInput {
        passthrough: std::cell::Cell<bool>,
    }

    define_class!(
        // SAFETY: NSView permits subclassing; this class only customizes hit testing.
        #[unsafe(super = NSView)]
        #[thread_kind = MainThreadOnly]
        #[ivars = ContainerInput]
        struct PluginContainerView;
        // SAFETY: NSObjectProtocol adds no requirements.
        unsafe impl NSObjectProtocol for PluginContainerView {}
        impl PluginContainerView {
            // SAFETY: signature matches NSView hitTest:, returning an optional retained view.
            #[unsafe(method_id(hitTest:))]
            fn hit_test(&self, point: NSPoint) -> Option<Retained<NSView>> {
                if self.ivars().passthrough.get() { None } else {
                    // SAFETY: calls the superclass implementation with its declared point argument.
                    unsafe { msg_send![super(self), hitTest: point] }
                }
            }
        }
    );

    /// A plain view in the host's window that the plugin's view goes into.
    pub(super) struct Container {
        view: Retained<PluginContainerView>,
        window: Retained<NSWindow>,
        embedded: std::cell::Cell<bool>,
    }

    impl Container {
        /// Add a child view sized for the editor, `top_inset` points below the content view's top.
        /// Returns `PluginError::Unsupported` off the main thread or if the window has no content view.
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
            let allocated = PluginContainerView::alloc(mtm).set_ivars(ContainerInput::default());
            // SAFETY: initialize the NSView subclass through its designated initializer.
            let view: Retained<PluginContainerView> = unsafe { msg_send![super(allocated), initWithFrame: frame] };
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
        /// Does nothing for an embedded container.
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

        /// Keep subsequent editor resize requests from resizing the host window.
        pub(super) fn embed(&self) {
            self.embedded.set(true);
        }

        pub(super) fn describe(&self) -> String {
            format!("container {:?}", self.view.frame())
        }

        /// Place the child at `(x, y)` points from the content layout area's top left.
        /// Use `natural` for its size; the requested width and height are ignored.
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
            let next = NSRect::new(NSPoint::new(x, y), NSSize::new(width, height));
            if self.view.frame() != next {
                self.view.setFrame(next);
            }
        }

        pub(super) fn set_input_passthrough(&self, passthrough: bool) {
            self.view.ivars().passthrough.set(passthrough);
        }

        pub(super) fn set_hidden(&self, hidden: bool) {
            self.view.setHidden(hidden);
        }

        pub(super) fn set_cutouts(&self, _rects: &[(f64, f64, f64, f64)]) {}
    }

    impl Drop for Container {
        fn drop(&mut self) {
            self.view.removeFromSuperview();
        }
    }
}

#[cfg(target_os = "windows")]
mod platform {
    //! Plugin views in popup windows (`HWND`) owned by the host's main window and kept over the
    //! editor area. Coordinates passed in are logical points from the client area's top left;
    //! plugin sizes are physical pixels, as VST3 uses on Windows.
    use super::EditorSize;
    use crate::PluginError;
    use std::cell::{Cell, RefCell};
    use std::ffi::c_void;
    use std::sync::OnceLock;
    use windows_sys::Win32::Foundation::{BOOL, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
    use windows_sys::Win32::Graphics::Gdi::{
        ClientToScreen, CombineRgn, CreateRectRgn, CreateSolidBrush, DeleteObject, RDW_ALLCHILDREN, RDW_UPDATENOW,
        RGN_DIFF, RedrawWindow, ScreenToClient, SetWindowRgn,
    };
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::System::Threading::GetCurrentThreadId;
    use windows_sys::Win32::UI::HiDpi::GetDpiForWindow;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::EnableWindow;
    use windows_sys::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        AppendMenuW, CS_HREDRAW, CS_VREDRAW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu,
        DestroyWindow, DispatchMessageW, EnumThreadWindows, GW_CHILD, GetClassNameW, GetClientRect, GetCursorPos,
        GetWindow, GetWindowTextW, IsWindowVisible, MF_STRING, MSG, PM_REMOVE, PeekMessageW, RegisterClassW, SW_HIDE,
        SW_SHOWNA, SWP_NOACTIVATE, SWP_NOOWNERZORDER, SWP_NOZORDER, SetForegroundWindow, SetWindowPos, ShowWindow,
        TPM_NONOTIFY, TPM_RETURNCMD, TrackPopupMenu, WM_NCDESTROY, WM_TIMER, WM_WINDOWPOSCHANGED, WNDCLASSW,
        WS_CLIPCHILDREN, WS_CLIPSIBLINGS, WS_EX_TOOLWINDOW, WS_POPUP,
    };

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    /// Physical pixels per logical point for `hwnd` (1.0 when the DPI is unknown).
    fn scale(hwnd: HWND) -> f64 {
        // SAFETY: a plain query; an invalid window yields 0.
        match unsafe { GetDpiForWindow(hwnd) } {
            0 => 1.0,
            dpi => f64::from(dpi) / 96.0,
        }
    }

    pub(super) struct Window(HWND);

    struct Search {
        title: Vec<u16>,
        found: HWND,
    }

    unsafe extern "system" fn match_title(hwnd: HWND, lparam: LPARAM) -> BOOL {
        // SAFETY: `lparam` is the `Search` that `find` passes for the duration of the enumeration.
        let search = unsafe { &mut *(lparam as *mut Search) };
        let mut buf = [0u16; 512];
        // SAFETY: the buffer and its length match; a window from the enumeration is valid.
        let n = unsafe { GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32) };
        // SAFETY: as above.
        let visible = unsafe { IsWindowVisible(hwnd) } != 0;
        if visible && usize::try_from(n).is_ok_and(|n| buf[..n] == search.title[..search.title.len() - 1]) {
            search.found = hwnd;
            return 0;
        }
        1
    }

    impl Window {
        pub(super) fn pointer_position(&self) -> Option<(f64, f64)> {
            let mut p = POINT { x: 0, y: 0 };
            // SAFETY: plain queries writing into `p`.
            if unsafe { GetCursorPos(&mut p) } == 0 || unsafe { ScreenToClient(self.0, &mut p) } == 0 {
                return None;
            }
            let s = scale(self.0);
            Some((f64::from(p.x) / s, f64::from(p.y) / s))
        }

        pub(super) fn popup_menu(&self, labels: &[String], x: f64, y: f64) -> Option<usize> {
            let s = scale(self.0);
            let mut p = POINT {
                x: (x * s).round() as i32,
                y: (y * s).round() as i32,
            };
            // SAFETY: a menu created, shown and destroyed within this call, owned by our window.
            unsafe {
                if ClientToScreen(self.0, &mut p) == 0 {
                    return None;
                }
                let menu = CreatePopupMenu();
                if menu.is_null() {
                    return None;
                }
                for (index, label) in labels.iter().enumerate() {
                    let text = wide(label);
                    AppendMenuW(menu, MF_STRING, index + 1, text.as_ptr());
                }
                SetForegroundWindow(self.0);
                let chosen = TrackPopupMenu(
                    menu,
                    TPM_RETURNCMD | TPM_NONOTIFY,
                    p.x,
                    p.y,
                    0,
                    self.0,
                    std::ptr::null(),
                );
                DestroyMenu(menu);
                usize::try_from(chosen).ok()?.checked_sub(1)
            }
        }

        pub(super) fn make_tool_window(&self) {}

        /// A visible top-level window of this (the UI) thread titled `title`.
        pub(super) fn find(title: &str) -> Option<Self> {
            let mut search = Search {
                title: wide(title),
                found: std::ptr::null_mut(),
            };
            // SAFETY: the callback only reads window text and writes into `search`, which
            // outlives the synchronous enumeration.
            unsafe {
                EnumThreadWindows(
                    GetCurrentThreadId(),
                    Some(match_title),
                    &mut search as *mut Search as LPARAM,
                );
            }
            (!search.found.is_null()).then_some(Self(search.found))
        }
    }

    unsafe extern "system" fn container_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        // SAFETY: default handling for our own window's messages.
        unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
    }

    pub(super) const CAN_CUT_OUT: bool = true;

    /// Dispatch the thread's due timers, then paint whatever the editors invalidated.
    pub(super) fn service() {
        // SAFETY: an all-zero `MSG` is valid; `PeekMessageW` fills it before it is dispatched.
        let mut msg: MSG = unsafe { std::mem::zeroed() };
        // A timer posts again only after its next period, so this ends; the bound is a guard.
        for _ in 0..64 {
            // SAFETY: standard message retrieval and dispatch on the UI thread.
            unsafe {
                if PeekMessageW(&mut msg, std::ptr::null_mut(), WM_TIMER, WM_TIMER, PM_REMOVE) == 0 {
                    break;
                }
                DispatchMessageW(&msg);
            }
        }
        // SAFETY: the callback only queries and repaints windows from the synchronous enumeration.
        unsafe { EnumThreadWindows(GetCurrentThreadId(), Some(repaint_container), 0) };
    }

    unsafe extern "system" fn repaint_container(hwnd: HWND, _: LPARAM) -> BOOL {
        let Some(class) = container_class() else {
            return 0;
        };
        let mut buf = [0u16; 64];
        // SAFETY: the buffer and its length match; a window from the enumeration is valid.
        let n = unsafe { GetClassNameW(hwnd, buf.as_mut_ptr(), buf.len() as i32) };
        // SAFETY: as above.
        if usize::try_from(n).is_ok_and(|n| buf[..n] == class[..class.len() - 1])
            && unsafe { IsWindowVisible(hwnd) } != 0
        {
            // SAFETY: paints the pending update regions of our popup and the plugin's windows in it.
            unsafe {
                RedrawWindow(
                    hwnd,
                    std::ptr::null(),
                    std::ptr::null_mut(),
                    RDW_UPDATENOW | RDW_ALLCHILDREN,
                )
            };
        }
        1
    }

    /// The container's window class, registered once, with a dark background for the moment
    /// before the plugin paints.
    fn container_class() -> Option<&'static [u16]> {
        static CLASS: OnceLock<Option<Vec<u16>>> = OnceLock::new();
        CLASS
            .get_or_init(|| {
                let name = wide("LimeShotPluginContainer");
                // SAFETY: the class name lives in the static and the procedure is a function; the
                // brush belongs to the class for the process's lifetime.
                let atom = unsafe {
                    let class = WNDCLASSW {
                        style: CS_HREDRAW | CS_VREDRAW,
                        lpfnWndProc: Some(container_proc),
                        cbClsExtra: 0,
                        cbWndExtra: 0,
                        hInstance: GetModuleHandleW(std::ptr::null()),
                        hIcon: std::ptr::null_mut(),
                        hCursor: std::ptr::null_mut(),
                        // Lime Dark surface, as COLORREF 0x00BBGGRR.
                        hbrBackground: CreateSolidBrush(0x0022_1F1E),
                        lpszMenuName: std::ptr::null(),
                        lpszClassName: name.as_ptr(),
                    };
                    RegisterClassW(&class)
                };
                (atom != 0).then_some(name)
            })
            .as_deref()
    }

    /// Where the container sits in the host's client area, in physical pixels. Shared with the
    /// host window's subclass procedure, which moves the container along with the host.
    struct Placement {
        popup: HWND,
        /// Left, top, width, height.
        rect: Cell<(i32, i32, i32, i32)>,
        /// Host content above the editor (left, top, right, bottom in client pixels).
        cutouts: RefCell<Vec<(i32, i32, i32, i32)>>,
        /// The last window region set, relative to the container: the part inside the client
        /// area, then the cutouts that overlap it. `None` is the whole window.
        region: RefCell<Option<Vec<(i32, i32, i32, i32)>>>,
    }

    impl Placement {
        /// Move the container over its place in the host's client area and clip it to that area.
        fn apply(&self, host: HWND) {
            let (x, y, w, h) = self.rect.get();
            let mut origin = POINT { x, y };
            let mut client = RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            // SAFETY: plain queries of the live host window writing into locals.
            if unsafe { ClientToScreen(host, &mut origin) } == 0 || unsafe { GetClientRect(host, &mut client) } == 0 {
                return;
            }
            // SAFETY: moves and sizes our own popup without changing its z-order or activation.
            unsafe {
                SetWindowPos(
                    self.popup,
                    std::ptr::null_mut(),
                    origin.x,
                    origin.y,
                    w,
                    h,
                    SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOOWNERZORDER,
                );
            }
            let visible = (
                (client.left - x).max(0),
                (client.top - y).max(0),
                (client.right - x).min(w).max(0),
                (client.bottom - y).min(h).max(0),
            );
            let (Ok(cutouts), Ok(mut last)) = (self.cutouts.try_borrow(), self.region.try_borrow_mut()) else {
                return;
            };
            let mut parts = vec![visible];
            parts.extend(
                cutouts
                    .iter()
                    .map(|&(l, t, r, b)| (l - x, t - y, r - x, b - y))
                    .filter(|&(l, t, r, b)| l < visible.2 && r > visible.0 && t < visible.3 && b > visible.1),
            );
            let region = (parts != [(0, 0, w, h)]).then_some(parts);
            if region == *last {
                return;
            }
            // SAFETY: regions made here are combined and then owned by the system once passed to
            // `SetWindowRgn` (the cutouts are deleted after use); a null region removes it.
            unsafe {
                let handle = match &region {
                    Some(parts) => {
                        let (l, t, r, b) = parts[0];
                        let handle = CreateRectRgn(l, t, r, b);
                        for &(l, t, r, b) in &parts[1..] {
                            let cut = CreateRectRgn(l, t, r, b);
                            CombineRgn(handle, handle, cut, RGN_DIFF);
                            DeleteObject(cut);
                        }
                        handle
                    }
                    None => std::ptr::null_mut(),
                };
                SetWindowRgn(self.popup, handle, 1);
            }
            *last = region;
        }
    }

    /// The host's window procedure, extended: when the host moves, its containers follow.
    unsafe extern "system" fn follow_host(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
        id: usize,
        data: usize,
    ) -> LRESULT {
        if msg == WM_WINDOWPOSCHANGED {
            // SAFETY: `data` is the container's boxed `Placement`, which outlives the subclass:
            // the container removes it before dropping the box.
            unsafe { (*(data as *const Placement)).apply(hwnd) };
        } else if msg == WM_NCDESTROY {
            // SAFETY: removes our own subclass from the window being destroyed.
            unsafe { RemoveWindowSubclass(hwnd, Some(follow_host), id) };
        }
        // SAFETY: the rest of the host's own handling.
        unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
    }

    /// The window the plugin's view goes into: a borderless popup owned by the host's window and
    /// kept over the editor area. A child window would be simpler, but common Windows OpenGL
    /// drivers present the host's surface over child windows, so a plugin view in one never
    /// reaches the screen. Owned popups stay above the host, minimise with it and stay off the
    /// taskbar.
    pub(super) struct Container {
        host: HWND,
        placement: Box<Placement>,
        embedded: Cell<bool>,
        /// Not shown before it is first placed, so it never flashes at the window's corner.
        placed: Cell<bool>,
        hidden: Cell<bool>,
    }

    impl Container {
        pub(super) fn new(window: &Window, size: EditorSize, _top_inset: f64) -> Result<Self, PluginError> {
            let class = container_class().ok_or(PluginError::Call("register the editor container class"))?;
            // SAFETY: creates a popup owned by the live host window with our registered class;
            // destroyed in Drop.
            let popup = unsafe {
                CreateWindowExW(
                    WS_EX_TOOLWINDOW,
                    class.as_ptr(),
                    std::ptr::null(),
                    WS_POPUP | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
                    0,
                    0,
                    size.width as i32,
                    size.height as i32,
                    window.0,
                    std::ptr::null_mut(),
                    GetModuleHandleW(std::ptr::null()),
                    std::ptr::null(),
                )
            };
            if popup.is_null() {
                return Err(PluginError::Call("create a window for its editor"));
            }
            let c = Self {
                host: window.0,
                placement: Box::new(Placement {
                    popup,
                    rect: Cell::new((0, 0, size.width as i32, size.height as i32)),
                    cutouts: RefCell::new(Vec::new()),
                    region: RefCell::new(None),
                }),
                embedded: Cell::new(false),
                placed: Cell::new(false),
                hidden: Cell::new(false),
            };
            let data = &*c.placement as *const Placement as usize;
            // SAFETY: the subclass reads the boxed placement, which stays at this address until
            // Drop removes the subclass. Its id (the same address) is unique per container.
            unsafe { SetWindowSubclass(c.host, Some(follow_host), data, data) };
            Ok(c)
        }

        pub(super) fn parent(&self) -> *mut c_void {
            self.placement.popup
        }

        fn set_rect(&self, x: f64, y: f64, size: EditorSize) {
            let s = scale(self.host);
            self.placement.rect.set((
                (x * s).round() as i32,
                (y * s).round() as i32,
                size.width as i32,
                size.height as i32,
            ));
            self.placement.apply(self.host);
            self.placed.set(true);
            self.show();
        }

        fn show(&self) {
            let show = self.placed.get() && !self.hidden.get();
            // SAFETY: shows or hides our own popup without activating it.
            unsafe { ShowWindow(self.placement.popup, if show { SW_SHOWNA } else { SW_HIDE }) };
        }

        pub(super) fn place(&self, size: EditorSize, top_inset: f64) {
            self.set_rect(0.0, top_inset, size);
        }

        /// Follow the editor's new size. The host window keeps its size; an embedded container
        /// also keeps its position until the next `place_rect`.
        pub(super) fn resize(&self, size: EditorSize, top_inset: f64) {
            if !self.embedded.get() {
                self.place(size, top_inset);
                return;
            }
            let (x, y, _, _) = self.placement.rect.get();
            self.placement.rect.set((x, y, size.width as i32, size.height as i32));
            self.placement.apply(self.host);
        }

        pub(super) fn embed(&self) {
            self.embedded.set(true);
        }

        /// The container and the plugin's first child window, for the log.
        pub(super) fn describe(&self) -> String {
            let rect = |hwnd: HWND| {
                let mut r = RECT {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                };
                // SAFETY: plain queries writing into `r`; a stale window only makes them fail.
                let ok = unsafe { GetClientRect(hwnd, &mut r) } != 0;
                // SAFETY: as above.
                let visible = unsafe { IsWindowVisible(hwnd) } != 0;
                let state = if ok && visible { "" } else { " (hidden)" };
                format!("{}x{}{state}", r.right - r.left, r.bottom - r.top)
            };
            let popup = self.placement.popup;
            // SAFETY: a plain query of our own window's first child.
            let child = unsafe { GetWindow(popup, GW_CHILD) };
            let plugin = if child.is_null() {
                "missing".to_owned()
            } else {
                format!("{child:?} {}", rect(child))
            };
            let (x, y, _, _) = self.placement.rect.get();
            format!(
                "container {popup:?} {} at {x},{y} px, scale {:.2}; plugin window {plugin}",
                rect(popup),
                scale(self.host)
            )
        }

        /// Place the child at `(x, y)` points from the client area's top left at its natural size.
        pub(super) fn place_rect(&self, x: f64, y: f64, _width: f64, _height: f64, natural: EditorSize) {
            self.set_rect(x, y, natural);
        }

        pub(super) fn set_hidden(&self, hidden: bool) {
            self.hidden.set(hidden);
            self.show();
        }

        pub(super) fn set_cutouts(&self, rects: &[(f64, f64, f64, f64)]) {
            let s = scale(self.host);
            let px = |v: f64| (v * s).round() as i32;
            let cutouts: Vec<_> = rects
                .iter()
                .map(|&(x, y, w, h)| (px(x), px(y), px(x + w), px(y + h)))
                .collect();
            let Ok(mut current) = self.placement.cutouts.try_borrow_mut() else {
                return;
            };
            if *current != cutouts {
                *current = cutouts;
                drop(current);
                self.placement.apply(self.host);
            }
        }

        /// A disabled window is skipped by hit testing (`WindowFromPoint`), so drags reach the
        /// host window.
        pub(super) fn set_input_passthrough(&self, passthrough: bool) {
            // SAFETY: enables or disables our own popup.
            unsafe { EnableWindow(self.placement.popup, BOOL::from(!passthrough)) };
        }
    }

    impl Drop for Container {
        fn drop(&mut self) {
            let data = &*self.placement as *const Placement as usize;
            // SAFETY: removes our subclass before the placement it reads is dropped, then
            // destroys our own popup; the plugin's view was removed before this.
            unsafe {
                RemoveWindowSubclass(self.host, Some(follow_host), data);
                DestroyWindow(self.placement.popup);
            }
        }
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod platform {
    pub(super) fn service() {}

    pub(super) const CAN_CUT_OUT: bool = false;

    use super::EditorSize;
    use crate::PluginError;

    pub(super) struct Window;

    impl Window {
        pub(super) fn pointer_position(&self) -> Option<(f64, f64)> {
            None
        }
        pub(super) fn popup_menu(&self, _labels: &[String], _x: f64, _y: f64) -> Option<usize> {
            None
        }
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
        /// Do nothing on platforms without native editor embedding.
        pub(super) fn embed(&self) {}
        pub(super) fn describe(&self) -> String {
            String::new()
        }
        /// Do nothing on platforms without native editor embedding.
        pub(super) fn place_rect(&self, _x: f64, _y: f64, _width: f64, _height: f64, _natural: EditorSize) {}
        pub(super) fn set_hidden(&self, _hidden: bool) {}
        pub(super) fn set_cutouts(&self, _rects: &[(f64, f64, f64, f64)]) {}
        pub(super) fn set_input_passthrough(&self, _passthrough: bool) {}
    }
}
