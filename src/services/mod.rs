use std::cell::RefCell;

use engage::app::procinst::ProcInst;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SaveLoadedArgs {
    pub ty: i32,
    pub slot_id: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ProcInstJumpArgs {
    pub proc: ProcInst,
    pub label: i32,
}

#[repr(C)]
pub struct ProcInstBindArgs {
    pub proc: RefCell<ProcInst>,
    pub parent: RefCell<ProcInst>,
}

#[cobapi_macros::service]
pub trait SystemEventService {
    fn subscribe_save_loaded(&self, handler: extern "C" fn(&SaveLoadedArgs));
    fn unsubscribe_save_loaded(&self, handler: extern "C" fn(&SaveLoadedArgs));

    fn subscribe_catalog_loaded(&self, handler: extern "C" fn());
    fn unsubscribe_catalog_loaded(&self, handler: extern "C" fn());

    fn subscribe_gamedata_loaded(&self, handler: extern "C" fn());
    fn unsubscribe_gamedata_loaded(&self, handler: extern "C" fn());

    fn subscribe_msbt_loaded(&self, handler: extern "C" fn());
    fn unsubscribe_msbt_loaded(&self, handler: extern "C" fn());

    fn subscribe_language_changed(&self, handler: extern "C" fn());
    fn unsubscribe_language_changed(&self, handler: extern "C" fn());

    fn subscribe_proc_inst_jump(&self, handler: extern "C" fn(&ProcInstJumpArgs));
    fn unsubscribe_proc_inst_jump(&self, handler: extern "C" fn(&ProcInstJumpArgs));

    fn subscribe_proc_inst_bind(&self, handler: extern "C" fn(&ProcInstBindArgs));
    fn unsubscribe_proc_inst_bind(&self, handler: extern "C" fn(&ProcInstBindArgs));
}
