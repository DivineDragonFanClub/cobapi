use std::cell::RefCell;

use engage_il2cpp::app::procinst::ProcInst;

#[repr(C)]
pub struct ProcInstBindArgs {
    pub proc: RefCell<ProcInst>,
    pub parent: RefCell<ProcInst>,
}

#[cobapi_macros::service]
pub trait ProcInstBindService {
    fn subscribe(&self, handler: extern "C" fn(&ProcInstBindArgs));
    fn unsubscribe(&self, handler: extern "C" fn(&ProcInstBindArgs));
}
