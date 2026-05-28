//! libX11 dynamic bindings via dlopen.
//!
//! Loads `libX11.so.6` (or `libX11.so`) at runtime and resolves the symbols
//! used by the engine. Contains no vendor-specific or architecture-specific
//! logic — Xlib's protocol is identical on every Linux GPU and CPU.

use core::ffi::{c_char, c_int, c_long, c_uchar, c_uint, c_ulong, c_void};
use core::ptr;
use std::sync::Mutex;

pub(crate) type Display = c_void;
pub(crate) type Window = c_ulong;
pub(crate) type Atom = c_ulong;
pub(crate) type Visual = c_void;
pub(crate) type Colormap = c_ulong;
pub(crate) type VisualId = c_ulong;

pub(crate) const RTLD_NOW: c_int = 2;
pub(crate) const RTLD_GLOBAL: c_int = 0x100;

pub(crate) const KEY_PRESS_MASK: c_long = 1 << 0;
pub(crate) const KEY_RELEASE_MASK: c_long = 1 << 1;
pub(crate) const BUTTON_PRESS_MASK: c_long = 1 << 2;
pub(crate) const BUTTON_RELEASE_MASK: c_long = 1 << 3;
pub(crate) const POINTER_MOTION_MASK: c_long = 1 << 6;
pub(crate) const STRUCTURE_NOTIFY_MASK: c_long = 1 << 17;
pub(crate) const EXPOSURE_MASK: c_long = 1 << 15;

pub(crate) const KEY_PRESS_EVENT: c_int = 2;
pub(crate) const KEY_RELEASE_EVENT: c_int = 3;
pub(crate) const BUTTON_PRESS_EVENT: c_int = 4;
pub(crate) const BUTTON_RELEASE_EVENT: c_int = 5;
pub(crate) const MOTION_NOTIFY_EVENT: c_int = 6;
pub(crate) const EXPOSE_EVENT: c_int = 12;
pub(crate) const SELECTION_NOTIFY_EVENT: c_int = 31;
pub(crate) const CONFIGURE_NOTIFY_EVENT: c_int = 22;
pub(crate) const CLIENT_MESSAGE_EVENT: c_int = 33;

pub(crate) const XA_ATOM: Atom = 4;
pub(crate) const PROP_MODE_REPLACE: c_int = 0;
pub(crate) const ANY_PROPERTY_TYPE: Atom = 0;
pub(crate) const XDND_VERSION: c_long = 5;
pub(crate) const NO_EVENT_MASK: c_long = 0;

pub(crate) const CW_BACK_PIXEL: c_ulong = 1 << 1;
pub(crate) const CW_BORDER_PIXEL: c_ulong = 1 << 3;
pub(crate) const CW_EVENT_MASK: c_ulong = 1 << 11;
pub(crate) const CW_COLORMAP: c_ulong = 1 << 13;

pub(crate) const ALLOC_NONE: c_int = 0;
pub(crate) const INPUT_OUTPUT: c_uint = 1;

#[repr(C)]
pub(crate) struct XSetWindowAttributes {
    pub background_pixmap: c_ulong,
    pub background_pixel: c_ulong,
    pub border_pixmap: c_ulong,
    pub border_pixel: c_ulong,
    pub bit_gravity: c_int,
    pub win_gravity: c_int,
    pub backing_store: c_int,
    pub backing_planes: c_ulong,
    pub backing_pixel: c_ulong,
    pub save_under: c_int,
    pub event_mask: c_long,
    pub do_not_propagate_mask: c_long,
    pub override_redirect: c_int,
    pub colormap: Colormap,
    pub cursor: c_ulong,
}

#[repr(C)]
pub(crate) struct XVisualInfo {
    pub visual: *mut Visual,
    pub visualid: VisualId,
    pub screen: c_int,
    pub depth: c_int,
    pub class: c_int,
    pub red_mask: c_ulong,
    pub green_mask: c_ulong,
    pub blue_mask: c_ulong,
    pub colormap_size: c_int,
    pub bits_per_rgb: c_int,
}

#[repr(C)]
pub(crate) struct XClientMessageEvent {
    pub type_: c_int,
    pub serial: c_ulong,
    pub send_event: c_int,
    pub display: *mut Display,
    pub window: Window,
    pub message_type: Atom,
    pub format: c_int,
    pub data: [c_long; 5],
}

#[repr(C)]
pub(crate) struct XSelectionEvent {
    pub type_: c_int,
    pub serial: c_ulong,
    pub send_event: c_int,
    pub display: *mut Display,
    pub requestor: Window,
    pub selection: Atom,
    pub target: Atom,
    pub property: Atom,
    pub time: c_ulong,
}

#[repr(C)]
pub(crate) struct XConfigureEvent {
    pub type_: c_int,
    pub serial: c_ulong,
    pub send_event: c_int,
    pub display: *mut Display,
    pub event: Window,
    pub window: Window,
    pub x: c_int,
    pub y: c_int,
    pub width: c_int,
    pub height: c_int,
    pub border_width: c_int,
    pub above: Window,
    pub override_redirect: c_int,
}

#[repr(C)]
pub(crate) struct XKeyEvent {
    pub type_: c_int,
    pub serial: c_ulong,
    pub send_event: c_int,
    pub display: *mut Display,
    pub window: Window,
    pub root: Window,
    pub subwindow: Window,
    pub time: c_ulong,
    pub x: c_int,
    pub y: c_int,
    pub x_root: c_int,
    pub y_root: c_int,
    pub state: c_uint,
    pub keycode: c_uint,
    pub same_screen: c_int,
}

#[repr(C)]
pub(crate) struct XButtonEvent {
    pub type_: c_int,
    pub serial: c_ulong,
    pub send_event: c_int,
    pub display: *mut Display,
    pub window: Window,
    pub root: Window,
    pub subwindow: Window,
    pub time: c_ulong,
    pub x: c_int,
    pub y: c_int,
    pub x_root: c_int,
    pub y_root: c_int,
    pub state: c_uint,
    pub button: c_uint,
    pub same_screen: c_int,
}

#[repr(C)]
pub(crate) struct XMotionEvent {
    pub type_: c_int,
    pub serial: c_ulong,
    pub send_event: c_int,
    pub display: *mut Display,
    pub window: Window,
    pub root: Window,
    pub subwindow: Window,
    pub time: c_ulong,
    pub x: c_int,
    pub y: c_int,
    pub x_root: c_int,
    pub y_root: c_int,
    pub state: c_uint,
    pub is_hint: c_char,
    pub same_screen: c_int,
}

#[repr(C)]
pub(crate) union XEvent {
    pub type_: c_int,
    pub key: core::mem::ManuallyDrop<XKeyEvent>,
    pub button: core::mem::ManuallyDrop<XButtonEvent>,
    pub motion: core::mem::ManuallyDrop<XMotionEvent>,
    pub configure: core::mem::ManuallyDrop<XConfigureEvent>,
    pub client_message: core::mem::ManuallyDrop<XClientMessageEvent>,
    pub selection: core::mem::ManuallyDrop<XSelectionEvent>,
    pub _pad: [c_long; 24],
}

unsafe extern "C" {
    fn dlopen(filename: *const c_char, flag: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    fn dlerror() -> *const c_char;
}

type FnXOpenDisplay = unsafe extern "C" fn(*const c_char) -> *mut Display;
type FnXCloseDisplay = unsafe extern "C" fn(*mut Display) -> c_int;
type FnXDefaultScreen = unsafe extern "C" fn(*mut Display) -> c_int;
type FnXRootWindow = unsafe extern "C" fn(*mut Display, c_int) -> Window;
type FnXCreateColormap =
    unsafe extern "C" fn(*mut Display, Window, *mut Visual, c_int) -> Colormap;
type FnXCreateWindow = unsafe extern "C" fn(
    *mut Display,
    Window,
    c_int,
    c_int,
    c_uint,
    c_uint,
    c_uint,
    c_int,
    c_uint,
    *mut Visual,
    c_ulong,
    *mut XSetWindowAttributes,
) -> Window;
type FnXDestroyWindow = unsafe extern "C" fn(*mut Display, Window) -> c_int;
type FnXMapWindow = unsafe extern "C" fn(*mut Display, Window) -> c_int;
type FnXStoreName = unsafe extern "C" fn(*mut Display, Window, *const c_char) -> c_int;
type FnXSelectInput = unsafe extern "C" fn(*mut Display, Window, c_long) -> c_int;
type FnXSync = unsafe extern "C" fn(*mut Display, c_int) -> c_int;
type FnXFlush = unsafe extern "C" fn(*mut Display) -> c_int;
type FnXPending = unsafe extern "C" fn(*mut Display) -> c_int;
type FnXNextEvent = unsafe extern "C" fn(*mut Display, *mut XEvent) -> c_int;
type FnXInternAtom = unsafe extern "C" fn(*mut Display, *const c_char, c_int) -> Atom;
type FnXSetWMProtocols = unsafe extern "C" fn(*mut Display, Window, *mut Atom, c_int) -> c_int;
type FnXFree = unsafe extern "C" fn(*mut c_void) -> c_int;
type FnXLookupKeysym = unsafe extern "C" fn(*mut XKeyEvent, c_int) -> c_ulong;
type FnXInitThreads = unsafe extern "C" fn() -> c_int;
type FnXChangeProperty = unsafe extern "C" fn(
    *mut Display,
    Window,
    Atom,
    Atom,
    c_int,
    c_int,
    *const c_uchar,
    c_int,
) -> c_int;
type FnXGetWindowProperty = unsafe extern "C" fn(
    *mut Display,
    Window,
    Atom,
    c_long,
    c_long,
    c_int,
    Atom,
    *mut Atom,
    *mut c_int,
    *mut c_ulong,
    *mut c_ulong,
    *mut *mut c_uchar,
) -> c_int;
type FnXSendEvent =
    unsafe extern "C" fn(*mut Display, Window, c_int, c_long, *mut XEvent) -> c_int;
type FnXConvertSelection = unsafe extern "C" fn(
    *mut Display,
    Atom,
    Atom,
    Atom,
    Window,
    c_ulong,
) -> c_int;
type FnXDeleteProperty = unsafe extern "C" fn(*mut Display, Window, Atom) -> c_int;

#[allow(non_snake_case, dead_code)]
pub(crate) struct Xlib {
    pub handle: *mut c_void,
    pub XOpenDisplay: FnXOpenDisplay,
    pub XCloseDisplay: FnXCloseDisplay,
    pub XDefaultScreen: FnXDefaultScreen,
    pub XRootWindow: FnXRootWindow,
    pub XCreateColormap: FnXCreateColormap,
    pub XCreateWindow: FnXCreateWindow,
    pub XDestroyWindow: FnXDestroyWindow,
    pub XMapWindow: FnXMapWindow,
    pub XStoreName: FnXStoreName,
    pub XSelectInput: FnXSelectInput,
    pub XSync: FnXSync,
    pub XFlush: FnXFlush,
    pub XPending: FnXPending,
    pub XNextEvent: FnXNextEvent,
    pub XInternAtom: FnXInternAtom,
    pub XSetWMProtocols: FnXSetWMProtocols,
    pub XFree: FnXFree,
    pub XLookupKeysym: FnXLookupKeysym,
    pub XInitThreads: FnXInitThreads,
    pub XChangeProperty: FnXChangeProperty,
    pub XGetWindowProperty: FnXGetWindowProperty,
    pub XSendEvent: FnXSendEvent,
    pub XConvertSelection: FnXConvertSelection,
    pub XDeleteProperty: FnXDeleteProperty,
}

unsafe impl Send for Xlib {}
unsafe impl Sync for Xlib {}

static XLIB: Mutex<Option<&'static Xlib>> = Mutex::new(None);

unsafe fn load_sym<T: Copy>(handle: *mut c_void, name: &[u8]) -> Option<T> {
    debug_assert_eq!(name.last(), Some(&0));
    let sym = unsafe { dlsym(handle, name.as_ptr() as *const c_char) };
    if sym.is_null() {
        return None;
    }
    debug_assert_eq!(core::mem::size_of::<T>(), core::mem::size_of::<*mut c_void>());
    Some(unsafe { core::mem::transmute_copy::<*mut c_void, T>(&sym) })
}

fn try_open(name: &[u8]) -> Option<*mut c_void> {
    debug_assert_eq!(name.last(), Some(&0));
    let handle = unsafe { dlopen(name.as_ptr() as *const c_char, RTLD_NOW | RTLD_GLOBAL) };
    if handle.is_null() {
        unsafe { dlerror() };
        None
    } else {
        Some(handle)
    }
}

pub(crate) fn xlib() -> Option<&'static Xlib> {
    let mut guard = XLIB.lock().ok()?;
    if let Some(lib) = *guard {
        return Some(lib);
    }
    let handle = try_open(b"libX11.so.6\0").or_else(|| try_open(b"libX11.so\0"))?;
    let lib = unsafe {
        Xlib {
            handle,
            XOpenDisplay: load_sym(handle, b"XOpenDisplay\0")?,
            XCloseDisplay: load_sym(handle, b"XCloseDisplay\0")?,
            XDefaultScreen: load_sym(handle, b"XDefaultScreen\0")?,
            XRootWindow: load_sym(handle, b"XRootWindow\0")?,
            XCreateColormap: load_sym(handle, b"XCreateColormap\0")?,
            XCreateWindow: load_sym(handle, b"XCreateWindow\0")?,
            XDestroyWindow: load_sym(handle, b"XDestroyWindow\0")?,
            XMapWindow: load_sym(handle, b"XMapWindow\0")?,
            XStoreName: load_sym(handle, b"XStoreName\0")?,
            XSelectInput: load_sym(handle, b"XSelectInput\0")?,
            XSync: load_sym(handle, b"XSync\0")?,
            XFlush: load_sym(handle, b"XFlush\0")?,
            XPending: load_sym(handle, b"XPending\0")?,
            XNextEvent: load_sym(handle, b"XNextEvent\0")?,
            XInternAtom: load_sym(handle, b"XInternAtom\0")?,
            XSetWMProtocols: load_sym(handle, b"XSetWMProtocols\0")?,
            XFree: load_sym(handle, b"XFree\0")?,
            XLookupKeysym: load_sym(handle, b"XLookupKeysym\0")?,
            XInitThreads: load_sym(handle, b"XInitThreads\0")?,
            XChangeProperty: load_sym(handle, b"XChangeProperty\0")?,
            XGetWindowProperty: load_sym(handle, b"XGetWindowProperty\0")?,
            XSendEvent: load_sym(handle, b"XSendEvent\0")?,
            XConvertSelection: load_sym(handle, b"XConvertSelection\0")?,
            XDeleteProperty: load_sym(handle, b"XDeleteProperty\0")?,
        }
    };
    unsafe { (lib.XInitThreads)() };
    let leaked: &'static Xlib = Box::leak(Box::new(lib));
    *guard = Some(leaked);
    Some(leaked)
}

#[allow(dead_code)]
pub(crate) struct XdndAtoms {
    pub aware: Atom,
    pub enter: Atom,
    pub position: Atom,
    pub status: Atom,
    pub drop: Atom,
    pub finished: Atom,
    pub leave: Atom,
    pub selection: Atom,
    pub action_copy: Atom,
    pub type_list: Atom,
    pub uri_list: Atom,
    pub target_property: Atom,
}

#[allow(dead_code)]
pub(crate) struct X11Window {
    pub display: *mut Display,
    pub window: Window,
    pub wm_delete_window: Atom,
    pub width: u32,
    pub height: u32,
    pub closed: bool,
    pub visual: *mut Visual,
    pub colormap: Colormap,
    pub depth: c_int,
    pub screen: c_int,
    pub xdnd: XdndAtoms,
    pub xdnd_source: Window,
    pub xdnd_version: c_long,
    pub xdnd_format: Atom,
    pub xdnd_timestamp: c_ulong,
    pub dropped_files: Vec<String>,
}

unsafe impl Send for X11Window {}
unsafe impl Sync for X11Window {}

impl X11Window {
    /// Creates an X11 window on an already-open `Display`. The window takes
    /// ownership of the display only when `owns_display` is true.
    pub fn create_on_display(
        display: *mut Display,
        owns_display: bool,
        width: u32,
        height: u32,
        title: &str,
        visual_info: Option<XVisualInfo>,
    ) -> Option<Self> {
        let lib = xlib()?;
        let screen = unsafe { (lib.XDefaultScreen)(display) };
        let root = unsafe { (lib.XRootWindow)(display, screen) };
        let _ = owns_display;

        let (visual_ptr, depth) = match visual_info.as_ref() {
            Some(vi) => (vi.visual, vi.depth),
            None => (ptr::null_mut::<Visual>(), 24),
        };

        let colormap = if !visual_ptr.is_null() {
            unsafe { (lib.XCreateColormap)(display, root, visual_ptr, ALLOC_NONE) }
        } else {
            0
        };

        let mut attrs: XSetWindowAttributes = unsafe { core::mem::zeroed() };
        attrs.background_pixel = 0;
        attrs.border_pixel = 0;
        attrs.colormap = colormap;
        attrs.event_mask = STRUCTURE_NOTIFY_MASK
            | EXPOSURE_MASK
            | KEY_PRESS_MASK
            | KEY_RELEASE_MASK
            | BUTTON_PRESS_MASK
            | BUTTON_RELEASE_MASK
            | POINTER_MOTION_MASK;

        let valuemask = if colormap != 0 {
            CW_BACK_PIXEL | CW_BORDER_PIXEL | CW_EVENT_MASK | CW_COLORMAP
        } else {
            CW_BACK_PIXEL | CW_BORDER_PIXEL | CW_EVENT_MASK
        };

        let window = unsafe {
            (lib.XCreateWindow)(
                display,
                root,
                0,
                0,
                width,
                height,
                0,
                depth,
                INPUT_OUTPUT,
                visual_ptr,
                valuemask,
                &mut attrs,
            )
        };
        if window == 0 {
            unsafe { (lib.XCloseDisplay)(display) };
            return None;
        }

        let mut title_bytes: Vec<u8> = title.as_bytes().to_vec();
        title_bytes.push(0);
        unsafe { (lib.XStoreName)(display, window, title_bytes.as_ptr() as *const c_char) };

        let _wm_protocols = unsafe {
            (lib.XInternAtom)(display, c"WM_PROTOCOLS".as_ptr(), 0)
        };
        let wm_delete_window = unsafe {
            (lib.XInternAtom)(
                display,
                c"WM_DELETE_WINDOW".as_ptr(),
                0,
            )
        };
        if wm_delete_window != 0 {
            let mut atoms = [wm_delete_window];
            unsafe {
                (lib.XSetWMProtocols)(display, window, atoms.as_mut_ptr(), 1);
            }
        }

        let xdnd = intern_xdnd_atoms(lib, display);
        let version: c_long = XDND_VERSION;
        let value_ptr = &version as *const c_long as *const c_uchar;
        unsafe {
            (lib.XChangeProperty)(
                display,
                window,
                xdnd.aware,
                XA_ATOM,
                32,
                PROP_MODE_REPLACE,
                value_ptr,
                1,
            );
        }

        unsafe {
            (lib.XMapWindow)(display, window);
            (lib.XFlush)(display);
        }

        Some(Self {
            display,
            window,
            wm_delete_window,
            width,
            height,
            closed: false,
            visual: visual_ptr,
            colormap,
            depth,
            screen,
            xdnd,
            xdnd_source: 0,
            xdnd_version: 0,
            xdnd_format: 0,
            xdnd_timestamp: 0,
            dropped_files: Vec::new(),
        })
    }

    pub fn pump_events(&mut self) -> Vec<X11Event> {
        let mut events = Vec::new();
        let Some(lib) = xlib() else {
            return events;
        };
        loop {
            let pending = unsafe { (lib.XPending)(self.display) };
            if pending <= 0 {
                break;
            }
            let mut ev: XEvent = unsafe { core::mem::zeroed() };
            unsafe { (lib.XNextEvent)(self.display, &mut ev) };
            let ty = unsafe { ev.type_ };
            match ty {
                CLIENT_MESSAGE_EVENT => {
                    let cm = unsafe { &ev.client_message };
                    if cm.data[0] as Atom == self.wm_delete_window {
                        self.closed = true;
                        events.push(X11Event::CloseRequested);
                    } else if cm.message_type == self.xdnd.enter {
                        self.handle_xdnd_enter(lib, cm);
                    } else if cm.message_type == self.xdnd.position {
                        self.handle_xdnd_position(lib, cm);
                    } else if cm.message_type == self.xdnd.drop {
                        self.handle_xdnd_drop(lib, cm);
                    } else if cm.message_type == self.xdnd.leave {
                        self.xdnd_source = 0;
                        self.xdnd_format = 0;
                    }
                }
                SELECTION_NOTIFY_EVENT => {
                    let se = unsafe { &ev.selection };
                    if se.selection == self.xdnd.selection {
                        self.handle_xdnd_selection(lib, se);
                    }
                }
                CONFIGURE_NOTIFY_EVENT => {
                    let cn = unsafe { &ev.configure };
                    let new_w = cn.width.max(0) as u32;
                    let new_h = cn.height.max(0) as u32;
                    if new_w != self.width || new_h != self.height {
                        self.width = new_w;
                        self.height = new_h;
                        events.push(X11Event::Resized {
                            width: new_w,
                            height: new_h,
                        });
                    }
                }
                EXPOSE_EVENT => {
                    events.push(X11Event::Expose);
                }
                KEY_PRESS_EVENT => {
                    let key_ptr = core::ptr::addr_of_mut!(ev.key) as *mut XKeyEvent;
                    let keysym = unsafe { (lib.XLookupKeysym)(key_ptr, 0) };
                    events.push(X11Event::KeyPress { keysym });
                }
                KEY_RELEASE_EVENT => {
                    let key_ptr = core::ptr::addr_of_mut!(ev.key) as *mut XKeyEvent;
                    let keysym = unsafe { (lib.XLookupKeysym)(key_ptr, 0) };
                    events.push(X11Event::KeyRelease { keysym });
                }
                BUTTON_PRESS_EVENT => {
                    let b = unsafe { &ev.button };
                    events.push(X11Event::MouseButtonPress {
                        button: b.button,
                        x: b.x,
                        y: b.y,
                    });
                }
                BUTTON_RELEASE_EVENT => {
                    let b = unsafe { &ev.button };
                    events.push(X11Event::MouseButtonRelease {
                        button: b.button,
                        x: b.x,
                        y: b.y,
                    });
                }
                MOTION_NOTIFY_EVENT => {
                    let m = unsafe { &ev.motion };
                    events.push(X11Event::MouseMove { x: m.x, y: m.y });
                }
                _ => {}
            }
        }
        events
    }

    pub fn take_dropped_files(&mut self) -> Vec<String> {
        core::mem::take(&mut self.dropped_files)
    }

    fn handle_xdnd_enter(&mut self, lib: &Xlib, cm: &XClientMessageEvent) {
        self.xdnd_source = cm.data[0] as Window;
        self.xdnd_version = (cm.data[1] >> 24) & 0xFF;
        self.xdnd_format = 0;
        let candidates: [Atom; 3] = [
            cm.data[2] as Atom,
            cm.data[3] as Atom,
            cm.data[4] as Atom,
        ];
        for atom in candidates {
            if atom == self.xdnd.uri_list {
                self.xdnd_format = atom;
                break;
            }
        }
        if self.xdnd_format == 0 && (cm.data[1] & 1) != 0 {
            self.xdnd_format = self.find_uri_list_in_type_list(lib);
        }
    }

    fn find_uri_list_in_type_list(&self, lib: &Xlib) -> Atom {
        let mut actual_type: Atom = 0;
        let mut actual_format: c_int = 0;
        let mut nitems: c_ulong = 0;
        let mut bytes_after: c_ulong = 0;
        let mut data: *mut c_uchar = ptr::null_mut();
        let status = unsafe {
            (lib.XGetWindowProperty)(
                self.display,
                self.xdnd_source,
                self.xdnd.type_list,
                0,
                65536,
                0,
                XA_ATOM,
                &mut actual_type,
                &mut actual_format,
                &mut nitems,
                &mut bytes_after,
                &mut data,
            )
        };
        if status != 0 || data.is_null() || actual_format != 32 {
            if !data.is_null() {
                unsafe { (lib.XFree)(data as *mut c_void) };
            }
            return 0;
        }
        let atoms = unsafe { core::slice::from_raw_parts(data as *const Atom, nitems as usize) };
        let found = atoms
            .iter()
            .copied()
            .find(|a| *a == self.xdnd.uri_list)
            .unwrap_or(0);
        unsafe { (lib.XFree)(data as *mut c_void) };
        found
    }

    fn handle_xdnd_position(&mut self, lib: &Xlib, cm: &XClientMessageEvent) {
        self.xdnd_timestamp = cm.data[3] as c_ulong;
        let accepted = self.xdnd_format != 0;
        let mut reply: XEvent = unsafe { core::mem::zeroed() };
        let cmsg = unsafe { &mut *core::ptr::addr_of_mut!(reply.client_message) };
        cmsg.type_ = CLIENT_MESSAGE_EVENT;
        cmsg.display = self.display;
        cmsg.window = self.xdnd_source;
        cmsg.message_type = self.xdnd.status;
        cmsg.format = 32;
        cmsg.data[0] = self.window as c_long;
        cmsg.data[1] = if accepted { 1 } else { 0 };
        cmsg.data[2] = 0;
        cmsg.data[3] = 0;
        cmsg.data[4] = if accepted {
            self.xdnd.action_copy as c_long
        } else {
            0
        };
        unsafe {
            (lib.XSendEvent)(
                self.display,
                self.xdnd_source,
                0,
                NO_EVENT_MASK,
                &mut reply,
            );
            (lib.XFlush)(self.display);
        }
    }

    fn handle_xdnd_drop(&mut self, lib: &Xlib, cm: &XClientMessageEvent) {
        if self.xdnd_format == 0 || self.xdnd_source == 0 {
            self.send_xdnd_finished(lib, false);
            return;
        }
        let timestamp = cm.data[2] as c_ulong;
        unsafe {
            (lib.XConvertSelection)(
                self.display,
                self.xdnd.selection,
                self.xdnd_format,
                self.xdnd.target_property,
                self.window,
                timestamp,
            );
            (lib.XFlush)(self.display);
        }
    }

    fn handle_xdnd_selection(&mut self, lib: &Xlib, se: &XSelectionEvent) {
        if se.property == 0 {
            self.send_xdnd_finished(lib, false);
            return;
        }
        let mut actual_type: Atom = 0;
        let mut actual_format: c_int = 0;
        let mut nitems: c_ulong = 0;
        let mut bytes_after: c_ulong = 0;
        let mut data: *mut c_uchar = ptr::null_mut();
        let status = unsafe {
            (lib.XGetWindowProperty)(
                self.display,
                self.window,
                self.xdnd.target_property,
                0,
                65536,
                0,
                ANY_PROPERTY_TYPE,
                &mut actual_type,
                &mut actual_format,
                &mut nitems,
                &mut bytes_after,
                &mut data,
            )
        };
        if status == 0 && !data.is_null() && nitems > 0 {
            let bytes = unsafe { core::slice::from_raw_parts(data, nitems as usize) };
            if let Ok(text) = core::str::from_utf8(bytes) {
                for line in text.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.starts_with('#') {
                        continue;
                    }
                    if let Some(path) = decode_file_uri(trimmed) {
                        self.dropped_files.push(path);
                    }
                }
            }
        }
        if !data.is_null() {
            unsafe { (lib.XFree)(data as *mut c_void) };
        }
        unsafe {
            (lib.XDeleteProperty)(self.display, self.window, self.xdnd.target_property);
        }
        self.send_xdnd_finished(lib, true);
    }

    fn send_xdnd_finished(&mut self, lib: &Xlib, accepted: bool) {
        if self.xdnd_source == 0 {
            return;
        }
        let mut reply: XEvent = unsafe { core::mem::zeroed() };
        let cmsg = unsafe { &mut *core::ptr::addr_of_mut!(reply.client_message) };
        cmsg.type_ = CLIENT_MESSAGE_EVENT;
        cmsg.display = self.display;
        cmsg.window = self.xdnd_source;
        cmsg.message_type = self.xdnd.finished;
        cmsg.format = 32;
        cmsg.data[0] = self.window as c_long;
        cmsg.data[1] = if accepted { 1 } else { 0 };
        cmsg.data[2] = if accepted {
            self.xdnd.action_copy as c_long
        } else {
            0
        };
        unsafe {
            (lib.XSendEvent)(
                self.display,
                self.xdnd_source,
                0,
                NO_EVENT_MASK,
                &mut reply,
            );
            (lib.XFlush)(self.display);
        }
        self.xdnd_source = 0;
        self.xdnd_format = 0;
    }

    pub fn close(&mut self) {
        if let Some(lib) = xlib() {
            unsafe {
                if self.window != 0 {
                    (lib.XDestroyWindow)(self.display, self.window);
                    self.window = 0;
                }
                if !self.display.is_null() {
                    (lib.XCloseDisplay)(self.display);
                    self.display = ptr::null_mut();
                }
            }
        }
    }
}

impl Drop for X11Window {
    fn drop(&mut self) {
        self.close();
    }
}

/// Subset of X11 events surfaced to the engine event loop.
#[derive(Debug, Clone, Copy)]
pub enum X11Event {
    CloseRequested,
    Resized { width: u32, height: u32 },
    Expose,
    KeyPress { keysym: c_ulong },
    KeyRelease { keysym: c_ulong },
    MouseButtonPress { button: c_uint, x: c_int, y: c_int },
    MouseButtonRelease { button: c_uint, x: c_int, y: c_int },
    MouseMove { x: c_int, y: c_int },
}

fn intern_xdnd_atoms(lib: &Xlib, display: *mut Display) -> XdndAtoms {
    XdndAtoms {
        aware: unsafe { (lib.XInternAtom)(display, c"XdndAware".as_ptr(), 0) },
        enter: unsafe { (lib.XInternAtom)(display, c"XdndEnter".as_ptr(), 0) },
        position: unsafe { (lib.XInternAtom)(display, c"XdndPosition".as_ptr(), 0) },
        status: unsafe { (lib.XInternAtom)(display, c"XdndStatus".as_ptr(), 0) },
        drop: unsafe { (lib.XInternAtom)(display, c"XdndDrop".as_ptr(), 0) },
        finished: unsafe { (lib.XInternAtom)(display, c"XdndFinished".as_ptr(), 0) },
        leave: unsafe { (lib.XInternAtom)(display, c"XdndLeave".as_ptr(), 0) },
        selection: unsafe { (lib.XInternAtom)(display, c"XdndSelection".as_ptr(), 0) },
        action_copy: unsafe { (lib.XInternAtom)(display, c"XdndActionCopy".as_ptr(), 0) },
        type_list: unsafe { (lib.XInternAtom)(display, c"XdndTypeList".as_ptr(), 0) },
        uri_list: unsafe { (lib.XInternAtom)(display, c"text/uri-list".as_ptr(), 0) },
        target_property: unsafe { (lib.XInternAtom)(display, c"RuxelXdndDrop".as_ptr(), 0) },
    }
}

fn decode_file_uri(input: &str) -> Option<String> {
    let body = input.strip_prefix("file://").unwrap_or(input);
    let body = body
        .strip_prefix("localhost")
        .map(|s| s.trim_start_matches('/'))
        .map(|s| {
            let mut owned = String::with_capacity(s.len() + 1);
            owned.push('/');
            owned.push_str(s);
            owned
        })
        .unwrap_or_else(|| body.to_string());
    let bytes = body.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'%' && i + 2 < bytes.len() {
            let hi = hex_value(bytes[i + 1])?;
            let lo = hex_value(bytes[i + 2])?;
            decoded.push((hi << 4) | lo);
            i += 3;
        } else {
            decoded.push(b);
            i += 1;
        }
    }
    String::from_utf8(decoded).ok().map(|s| s.trim().to_string())
}

fn hex_value(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}
