use std::cell::RefCell;
use std::ffi::c_void;

use engage::{
    proc::ProcInst,
    script::EventScript,
    menu::config::ConfigBasicMenuItem,
};

// 0.1.0

// Lua scripting
pub type EventScriptRegistrationCallback = extern "C" fn(&EventScript);
// Custom in-game settings
pub type GameSettingRegistrationCallback = extern "C" fn() -> &'static mut ConfigBasicMenuItem;

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
    ProcInstJump { proc: &'static ProcInst, label: i32 },
    // 0.4.0
    ProcInstBind {
        proc: RefCell<&'static mut ProcInst>,
        parent: RefCell<Option<&'static mut ProcInst>>,
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
pub type GlobalConfigMenuItemRegistrationCallback = extern "C" fn() -> &'static mut ConfigBasicMenuItem;

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
}

// 0.1.0

/// Install a new setting in the "Plugin Settings" sub-menu during gameplay.
/// 
/// Expects a function returning an instance of ConfigBasicMenuItem to be appended to the list of settings.
pub fn install_game_setting(callback: GameSettingRegistrationCallback) {
    unsafe { cobapi_register_configmenuitem_cb(callback as _)}
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
    unsafe { cobapi_register_global_configmenuitem_cb(callback as _)}
}

/// Returns Cobalt's versioning as a struct.
pub fn cobalt_version() -> CobApiVerison {
    unsafe { cobapi_cobalt_version() }
}