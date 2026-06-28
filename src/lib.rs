extern crate self as cobapi;

use std::cell::RefCell;
use std::ffi::c_void;

use engage::app::eventscript::EventScript;
use engage::app::procinst::ProcInst;
use engage::root::configbasicmenuitem::ConfigBasicMenuItem;

pub mod injection;
pub mod services;

// 0.1.0

// Lua scripting
pub type EventScriptRegistrationCallback = extern "C" fn(&EventScript);
pub type GameSettingRegistrationCallback = extern "C" fn() -> ConfigBasicMenuItem;

// 0.2.0

#[repr(C)]
#[derive(Debug)]
pub enum Event<E> {
    Args(E),
    Missing,
}

/// Events related to actions performed by the game that are not related to things happening in-game.
#[repr(C)]
#[non_exhaustive] // Force people to handle a _ case so we can add entries later on if needed.
pub enum SystemEvent {
    // 0.2.0
    CatalogLoaded,
    GamedataLoaded,
    MsbtLoaded,
    LanguageChanged,
    SaveLoaded { ty: i32, slot_id: i32 },
    // 0.3.0
    ProcInstJump { proc: ProcInst, label: i32 },
    // 0.4.0
    ProcInstBind {
        proc: RefCell<ProcInst>,
        parent: RefCell<ProcInst>,
    },
}

#[repr(C)]
pub struct CobApiVerison {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

// Event system
pub type SystemEventHandler = extern "C" fn(&Event<SystemEvent>);
pub type GlobalConfigMenuItemRegistrationCallback = extern "C" fn() -> ConfigBasicMenuItem;

// 0.5.0 — typed-service registry

#[repr(C)]
pub struct VTableHeader {
    pub abi_version: u32,
    pub _reserved: u32,
    pub method_count: u32,
    pub _pad: u32,
    pub methods: *const MethodEntry,
    pub drop_fn: unsafe extern "C" fn(*mut c_void),
}

unsafe impl Sync for VTableHeader {}

pub const VTABLE_ABI_VERSION: u32 = 1;

#[repr(C)]
pub struct MethodEntry {
    pub name_hash: u64,
    pub sig_hash: u64,
    pub fn_ptr: *const c_void,
}

unsafe impl Sync for MethodEntry {}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum RegisterServiceError {
    #[error("a service with this id is already registered")]
    AlreadyRegistered,
    #[error("service vtable ABI version mismatch")]
    AbiVersionMismatch,
}

extern "C" {
    // 0.1.0
    fn cobapi_register_configmenuitem_cb(callback: *const c_void);
    fn cobapi_register_eventscript_cb(callback: EventScriptRegistrationCallback);
    // 0.2.0
    fn cobapi_register_system_event_listener(callback: SystemEventHandler);
    fn cobapi_unregister_system_event_listener(callback: SystemEventHandler);
    fn cobapi_register_global_configmenuitem_cb(callback: GlobalConfigMenuItemRegistrationCallback);
    // 0.4.0
    fn cobapi_cobalt_version() -> CobApiVerison;

    // 0.5.0
    fn cobapi_register_service(
        id_ptr: *const u8,
        id_len: usize,
        this: *mut c_void,
        vtable: *const VTableHeader,
    ) -> i32;
    fn cobapi_lookup_service(
        id_ptr: *const u8,
        id_len: usize,
        out_this: *mut *mut c_void,
        out_vtable: *mut *const VTableHeader,
    ) -> i32;
    fn cobapi_unregister_service(id_ptr: *const u8, id_len: usize);
}

// 0.1.0

/// Install a new setting in the "Plugin Settings" sub-menu during gameplay.
///
/// Expects a function returning an instance of ConfigBasicMenuItem to be appended to the list of settings.
pub fn install_game_setting(callback: GameSettingRegistrationCallback) {
    unsafe { cobapi_register_configmenuitem_cb(callback as _) }
}

/// Install a new command registerer for lua scripts.
///
/// The callback will be provided with the EventScript used by the game to add handlers.
/// Note that both the game and Cobalt's commands are already installed by the time your callback is called.
pub fn install_lua_command_registerer(callback: EventScriptRegistrationCallback) {
    unsafe { cobapi_register_eventscript_cb(callback) }
}

// 0.2.0

/// Register a event handler for System events.
///
/// The callback will be provided with every Event in the System category.
/// Match on the ones you want to listen to.
pub fn register_system_event_handler(callback: SystemEventHandler) {
    unsafe { cobapi_register_system_event_listener(callback) }
}

/// Unregister your event handler for System events.
pub fn unregister_system_event_handler(callback: SystemEventHandler) {
    unsafe { cobapi_unregister_system_event_listener(callback) }
}

/// Install a new global setting in the "Plugin Settings" sub-menu in the Cobalt settings menu.
///
/// Expects a function returning an instance of ConfigBasicMenuItem to be appended to the list of settings.
pub fn install_global_game_setting(callback: GlobalConfigMenuItemRegistrationCallback) {
    unsafe { cobapi_register_global_configmenuitem_cb(callback as _) }
}

/// Returns Cobalt's versioning as a struct.
pub fn cobalt_version() -> CobApiVerison {
    unsafe { cobapi_cobalt_version() }
}

// 0.5.0

pub fn register_service(
    id: &str,
    this: *mut c_void,
    vtable: &'static VTableHeader,
) -> Result<(), RegisterServiceError> {
    let rc = unsafe {
        cobapi_register_service(id.as_ptr(), id.len(), this, vtable as *const VTableHeader)
    };

    match rc {
        0 => Ok(()),
        1 => Err(RegisterServiceError::AlreadyRegistered),
        2 => Err(RegisterServiceError::AbiVersionMismatch),
        _ => Err(RegisterServiceError::AlreadyRegistered),
    }
}

pub fn lookup_service(id: &str) -> Option<(*mut c_void, *const VTableHeader)> {
    let mut this: *mut c_void = core::ptr::null_mut();
    let mut vtable: *const VTableHeader = core::ptr::null();

    let rc = unsafe {
        cobapi_lookup_service(id.as_ptr(), id.len(), &mut this, &mut vtable)
    };

    if rc == 0 {
        Some((this, vtable))
    } else {
        None
    }
}

pub fn unregister_service(id: &str) {
    unsafe { cobapi_unregister_service(id.as_ptr(), id.len()) }
}

pub use cobapi_macros::service;
