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
            self.view
                .setFrame(NSRect::new(NSPoint::new(x, y), NSSize::new(width, height)));
        }

        pub(super) fn set_input_passthrough(&self, passthrough: bool) {
            self.view.ivars().passthrough.set(passthrough);
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

#[cfg(target_os = "windows")]
mod platform {
    //! Plugin views as child windows (`HWND`) of the host's main window. Coordinates passed in
    //! are logical points from the client area's top left; plugin sizes are physical pixels, as
    //! VST3 uses on Windows.
    use super::EditorSize;
    use crate::PluginError;
    use std::ffi::c_void;
    use std::sync::OnceLock;
    use windows_sys::Win32::Foundation::{BOOL, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
    use windows_sys::Win32::Graphics::Gdi::{ClientToScreen, CreateSolidBrush, ScreenToClient};
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::System::Threading::GetCurrentThreadId;
    use windows_sys::Win32::UI::HiDpi::GetDpiForWindow;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::EnableWindow;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        AppendMenuW, CS_HREDRAW, CS_VREDRAW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu,
        DestroyWindow, EnumThreadWindows, GW_CHILD, GetClientRect, GetCursorPos, GetWindow, GetWindowTextW, HWND_TOP,
        IsWindowVisible, MF_STRING, RegisterClassW, SW_HIDE, SW_SHOWNA, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOZORDER,
        SetForegroundWindow, SetWindowPos, ShowWindow, TPM_NONOTIFY, TPM_RETURNCMD, TrackPopupMenu, WNDCLASSW,
        WS_CHILD, WS_CLIPCHILDREN, WS_CLIPSIBLINGS, WS_VISIBLE,
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

    /// The container's window class, registered once. The system `STATIC` class paints through
    /// its parent's device context (`CS_PARENTDC`), which never reaches the screen over the
    /// host's OpenGL surface; this class paints its own dark background.
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

    /// A plain child window of the host's window that the plugin's view goes into.
    pub(super) struct Container {
        hwnd: HWND,
        host: HWND,
        embedded: std::cell::Cell<bool>,
    }

    impl Container {
        pub(super) fn new(window: &Window, size: EditorSize, top_inset: f64) -> Result<Self, PluginError> {
            let class = container_class().ok_or(PluginError::Call("register the editor container class"))?;
            // SAFETY: creates a child of the live host window with our registered class; destroyed in Drop.
            let hwnd = unsafe {
                CreateWindowExW(
                    0,
                    class.as_ptr(),
                    std::ptr::null(),
                    WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
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
            if hwnd.is_null() {
                return Err(PluginError::Call("create a child window for its editor"));
            }
            let c = Self {
                hwnd,
                host: window.0,
                embedded: std::cell::Cell::new(false),
            };
            c.place(size, top_inset);
            Ok(c)
        }

        pub(super) fn parent(&self) -> *mut c_void {
            self.hwnd
        }

        fn set_rect(&self, x: f64, y: f64, size: EditorSize) {
            let s = scale(self.host);
            // SAFETY: moves our own child window.
            unsafe {
                SetWindowPos(
                    self.hwnd,
                    HWND_TOP,
                    (x * s).round() as i32,
                    (y * s).round() as i32,
                    size.width as i32,
                    size.height as i32,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
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
            // SAFETY: resizes our own child window.
            unsafe {
                SetWindowPos(
                    self.hwnd,
                    HWND_TOP,
                    0,
                    0,
                    size.width as i32,
                    size.height as i32,
                    SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
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
            // SAFETY: a plain query of our own window's first child.
            let child = unsafe { GetWindow(self.hwnd, GW_CHILD) };
            let plugin = if child.is_null() {
                "missing".to_owned()
            } else {
                format!("{child:?} {}", rect(child))
            };
            format!(
                "container {:?} {} at scale {:.2}; plugin window {plugin}",
                self.hwnd,
                rect(self.hwnd),
                scale(self.host)
            )
        }

        /// Place the child at `(x, y)` points from the client area's top left at its natural size.
        pub(super) fn place_rect(&self, x: f64, y: f64, _width: f64, _height: f64, natural: EditorSize) {
            self.set_rect(x, y, natural);
        }

        pub(super) fn set_hidden(&self, hidden: bool) {
            // SAFETY: shows or hides our own child window without activating it.
            unsafe { ShowWindow(self.hwnd, if hidden { SW_HIDE } else { SW_SHOWNA }) };
        }

        /// A disabled child is skipped by hit testing, so drags reach the host window.
        pub(super) fn set_input_passthrough(&self, passthrough: bool) {
            // SAFETY: enables or disables our own child window.
            unsafe { EnableWindow(self.hwnd, BOOL::from(!passthrough)) };
        }
    }

    impl Drop for Container {
        fn drop(&mut self) {
            // SAFETY: our own child window; the plugin's view was removed before this.
            unsafe { DestroyWindow(self.hwnd) };
        }
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod platform {
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
        pub(super) fn set_input_passthrough(&self, _passthrough: bool) {}
    }
}
