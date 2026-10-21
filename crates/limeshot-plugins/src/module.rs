//! Loading a VST3 module (a `.vst3` bundle) and reading its plugin factory.
//!
//! A module stays loaded while any [`Vst3Module`] handle or plugin made from it is alive; loading
//! the same bundle twice shares one module. On macOS the executable is never unloaded: several
//! plugin frameworks register Objective-C classes that crash the host when their code goes away,
//! so mainstream hosts keep it loaded too.

use std::ffi::{c_char, c_void};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, Weak};

use vst3::Steinberg::*;
use vst3::{ComPtr, Interface};

use crate::util::{from_cstr, hex_to_tuid, tuid_to_hex};
use crate::{PluginError, PluginInfo, PluginKind};

type GetFactory = unsafe extern "system" fn() -> *mut IPluginFactory;

/// A loaded VST3 module and its factory.
pub struct Vst3Module {
    // Declared before `_lib` so the factory is released before the module's exit function runs.
    factory: ComPtr<IPluginFactory>,
    path: PathBuf,
    _lib: Option<platform::Library>,
}

// SAFETY: the factory is only used for `getClassInfo*` and `createInstance`, which the VST3
// specification allows from any thread as long as calls are not concurrent; LimeShot makes them
// from the control thread. The library handle is plain data.
unsafe impl Send for Vst3Module {}
unsafe impl Sync for Vst3Module {}

static LOADED: Mutex<Vec<(PathBuf, Weak<Vst3Module>)>> = Mutex::new(Vec::new());

impl Vst3Module {
    /// Load the bundle at `path`, or return the module already loaded from it.
    pub fn load(path: &Path) -> Result<Arc<Self>, PluginError> {
        let mut loaded = LOADED.lock().map_err(|_| PluginError::Load {
            path: path.to_owned(),
            reason: "module table poisoned".into(),
        })?;
        loaded.retain(|(_, m)| m.strong_count() > 0);
        if let Some(m) = loaded.iter().find(|(p, _)| p == path).and_then(|(_, m)| m.upgrade()) {
            return Ok(m);
        }
        let lib = platform::Library::open(path)?;
        let get = lib
            .symbol("GetPluginFactory")
            .ok_or_else(|| PluginError::NoFactory(path.to_owned()))?;
        // SAFETY: `GetPluginFactory` has this signature in every VST3 module.
        let get: GetFactory = unsafe { std::mem::transmute::<*const c_void, GetFactory>(get) };
        let module = Arc::new(Self::with_factory(get, path, Some(lib))?);
        loaded.push((path.to_owned(), Arc::downgrade(&module)));
        Ok(module)
    }

    /// A module whose factory is compiled into this program (LimeShot's own test plugins).
    pub(crate) fn from_factory(get: GetFactory) -> Result<Arc<Self>, PluginError> {
        Ok(Arc::new(Self::with_factory(get, Path::new(""), None)?))
    }

    fn with_factory(get: GetFactory, path: &Path, lib: Option<platform::Library>) -> Result<Self, PluginError> {
        // SAFETY: the module is initialised; the returned pointer carries one reference.
        let factory = unsafe { ComPtr::from_raw(get()) }.ok_or_else(|| PluginError::NoFactory(path.to_owned()))?;
        Ok(Self {
            factory,
            path: path.to_owned(),
            _lib: lib,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The audio processor classes (instruments and effects) the module offers.
    pub fn classes(&self) -> Vec<PluginInfo> {
        let mut out = Vec::new();
        // SAFETY: plain factory queries into stack structs of the documented size.
        unsafe {
            let mut finfo: PFactoryInfo = std::mem::zeroed();
            let factory_vendor = if self.factory.getFactoryInfo(&mut finfo) == kResultOk {
                from_cstr(&finfo.vendor)
            } else {
                String::new()
            };
            let f2 = self.factory.cast::<IPluginFactory2>();
            for i in 0..self.factory.countClasses().max(0) {
                let mut info = PluginInfo {
                    path: self.path.clone(),
                    class_id: String::new(),
                    name: String::new(),
                    vendor: factory_vendor.clone(),
                    version: String::new(),
                    sub_categories: String::new(),
                    kind: PluginKind::Effect,
                };
                let category;
                let mut c2: PClassInfo2 = std::mem::zeroed();
                if let Some(f2) = &f2
                    && f2.getClassInfo2(i, &mut c2) == kResultOk
                {
                    category = from_cstr(&c2.category);
                    info.class_id = tuid_to_hex(&c2.cid);
                    info.name = from_cstr(&c2.name);
                    let vendor = from_cstr(&c2.vendor);
                    if !vendor.is_empty() {
                        info.vendor = vendor;
                    }
                    info.version = from_cstr(&c2.version);
                    info.sub_categories = from_cstr(&c2.subCategories);
                } else {
                    let mut c: PClassInfo = std::mem::zeroed();
                    if self.factory.getClassInfo(i, &mut c) != kResultOk {
                        continue;
                    }
                    category = from_cstr(&c.category);
                    info.class_id = tuid_to_hex(&c.cid);
                    info.name = from_cstr(&c.name);
                }
                if category != "Audio Module Class" {
                    continue;
                }
                if info.sub_categories.split('|').any(|s| s == "Instrument") {
                    info.kind = PluginKind::Instrument;
                }
                out.push(info);
            }
        }
        out
    }

    /// Create an object of class `class_id` (hex) and return its interface `I`.
    pub(crate) fn create<I: Interface>(&self, class_id: &str) -> Option<ComPtr<I>> {
        let cid = hex_to_tuid(class_id)?;
        self.create_tuid(&cid)
    }

    pub(crate) fn create_tuid<I: Interface>(&self, cid: &TUID) -> Option<ComPtr<I>> {
        let mut obj: *mut c_void = std::ptr::null_mut();
        // SAFETY: `cid` and the IID are 16-byte ids; on success `obj` carries one reference.
        unsafe {
            let r = self
                .factory
                .createInstance(cid.as_ptr(), I::IID.as_ptr() as *const c_char, &mut obj);
            if r != kResultOk {
                return None;
            }
            ComPtr::from_raw(obj as *mut I)
        }
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use std::ffi::c_void;
    use std::os::unix::ffi::OsStrExt;
    use std::path::Path;

    use core_foundation_sys::base::{CFRelease, kCFAllocatorDefault};
    use core_foundation_sys::bundle::{
        CFBundleCreate, CFBundleGetFunctionPointerForName, CFBundleLoadExecutable, CFBundleRef,
    };
    use core_foundation_sys::string::{CFStringCreateWithBytes, kCFStringEncodingUTF8};
    use core_foundation_sys::url::CFURLCreateFromFileSystemRepresentation;

    use crate::PluginError;

    pub struct Library {
        bundle: CFBundleRef,
    }

    type Entry = unsafe extern "system" fn(CFBundleRef) -> bool;
    type Exit = unsafe extern "system" fn() -> bool;

    impl Library {
        pub fn open(path: &Path) -> Result<Self, PluginError> {
            let fail = |reason: &str| PluginError::Load {
                path: path.to_owned(),
                reason: reason.into(),
            };
            let bytes = path.as_os_str().as_bytes();
            // SAFETY: CoreFoundation calls with a valid byte buffer; every object created here
            // is released or kept in `Library`.
            unsafe {
                let url =
                    CFURLCreateFromFileSystemRepresentation(kCFAllocatorDefault, bytes.as_ptr(), bytes.len() as _, 1);
                if url.is_null() {
                    return Err(fail("not a file path"));
                }
                let bundle = CFBundleCreate(kCFAllocatorDefault, url);
                CFRelease(url as *const c_void);
                if bundle.is_null() {
                    return Err(fail("not a bundle"));
                }
                if CFBundleLoadExecutable(bundle) == 0 {
                    CFRelease(bundle as *const c_void);
                    return Err(fail("the executable did not load"));
                }
                let lib = Self { bundle };
                let entry = lib.symbol("bundleEntry").or_else(|| lib.symbol("BundleEntry"));
                if let Some(entry) = entry {
                    let entry = std::mem::transmute::<*const c_void, Entry>(entry);
                    if !entry(bundle) {
                        return Err(fail("bundleEntry failed"));
                    }
                }
                Ok(lib)
            }
        }

        pub fn symbol(&self, name: &str) -> Option<*const c_void> {
            // SAFETY: a temporary CFString for the lookup, released before returning.
            unsafe {
                let s = CFStringCreateWithBytes(
                    kCFAllocatorDefault,
                    name.as_ptr(),
                    name.len() as _,
                    kCFStringEncodingUTF8,
                    0,
                );
                if s.is_null() {
                    return None;
                }
                let f = CFBundleGetFunctionPointerForName(self.bundle, s);
                CFRelease(s as *const c_void);
                (!f.is_null()).then_some(f)
            }
        }
    }

    impl Drop for Library {
        fn drop(&mut self) {
            let exit = self.symbol("bundleExit").or_else(|| self.symbol("BundleExit"));
            if let Some(exit) = exit {
                // SAFETY: the documented exit function, called once per successful entry.
                unsafe { std::mem::transmute::<*const c_void, Exit>(exit)() };
            }
            // The bundle reference is kept (see the module docs): releasing the last one may
            // unload the executable.
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod platform {
    use std::ffi::c_void;
    use std::path::{Path, PathBuf};

    use crate::PluginError;

    pub struct Library {
        lib: Option<libloading::Library>,
        #[cfg(not(target_os = "windows"))]
        handle: *mut c_void,
    }

    /// The module binary inside a bundle (a plain file is taken as the binary itself).
    fn binary(path: &Path) -> PathBuf {
        if path.is_file() {
            return path.to_owned();
        }
        let stem = path.file_stem().unwrap_or_default();
        #[cfg(target_os = "windows")]
        {
            let arch = if cfg!(target_arch = "aarch64") {
                "arm64"
            } else {
                "x86_64"
            };
            let mut p = path.join("Contents").join(format!("{arch}-win")).join(stem);
            p.set_extension("vst3");
            if !p.exists() && cfg!(target_arch = "aarch64") {
                let mut x = path.join("Contents").join("arm64x-win").join(stem);
                x.set_extension("vst3");
                return x;
            }
            p
        }
        #[cfg(not(target_os = "windows"))]
        {
            let arch = if cfg!(target_arch = "aarch64") {
                "aarch64"
            } else {
                "x86_64"
            };
            let mut p = path.join("Contents").join(format!("{arch}-linux")).join(stem);
            p.set_extension("so");
            p
        }
    }

    impl Library {
        pub fn open(path: &Path) -> Result<Self, PluginError> {
            let fail = |reason: String| PluginError::Load {
                path: path.to_owned(),
                reason,
            };
            // SAFETY: loading a plugin runs its initialisers; that is what hosting means.
            #[cfg(target_os = "windows")]
            let lib = Self {
                lib: Some(unsafe { libloading::Library::new(binary(path)) }.map_err(|e| fail(e.to_string()))?),
            };
            // SAFETY: as above; the raw handle is turned straight back into a library and kept
            // only to pass to ModuleEntry.
            #[cfg(not(target_os = "windows"))]
            let lib = unsafe {
                let unix = libloading::os::unix::Library::new(binary(path)).map_err(|e| fail(e.to_string()))?;
                let handle = unix.into_raw();
                let unix = libloading::os::unix::Library::from_raw(handle);
                Self {
                    lib: Some(unix.into()),
                    handle,
                }
            };
            #[cfg(target_os = "windows")]
            if let Some(entry) = lib.symbol("InitDll") {
                type Entry = unsafe extern "system" fn() -> bool;
                // SAFETY: the documented entry function.
                if !unsafe { std::mem::transmute::<*const c_void, Entry>(entry)() } {
                    return Err(fail("InitDll failed".into()));
                }
            }
            #[cfg(not(target_os = "windows"))]
            if let Some(entry) = lib.symbol("ModuleEntry") {
                type Entry = unsafe extern "system" fn(*mut c_void) -> bool;
                let handle = lib.handle();
                // SAFETY: the documented entry function, given the dlopen handle.
                if !unsafe { std::mem::transmute::<*const c_void, Entry>(entry)(handle) } {
                    return Err(fail("ModuleEntry failed".into()));
                }
            }
            Ok(lib)
        }

        #[cfg(not(target_os = "windows"))]
        fn handle(&self) -> *mut c_void {
            self.handle
        }

        pub fn symbol(&self, name: &str) -> Option<*const c_void> {
            let lib = self.lib.as_ref()?;
            let mut cname = name.as_bytes().to_vec();
            cname.push(0);
            // SAFETY: the symbol is only transmuted to its documented signature by the caller.
            unsafe { lib.get::<*const c_void>(&cname).ok().map(|s| *s) }
        }
    }

    impl Drop for Library {
        fn drop(&mut self) {
            let exit = if cfg!(target_os = "windows") {
                "ExitDll"
            } else {
                "ModuleExit"
            };
            if let Some(exit) = self.symbol(exit) {
                type Exit = unsafe extern "system" fn() -> bool;
                // SAFETY: the documented exit function, called once after a successful entry.
                unsafe { std::mem::transmute::<*const c_void, Exit>(exit)() };
            }
            drop(self.lib.take());
        }
    }
}
