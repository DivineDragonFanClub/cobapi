use engage_il2cpp::app::procinst::ProcInst;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ProcInstJumpArgs {
    pub proc: ProcInst,
    pub label: i32,
}

#[cobapi_macros::service]
pub trait ProcInstJumpService {
    fn subscribe(&self, handler: extern "C" fn(&ProcInstJumpArgs));
    fn unsubscribe(&self, handler: extern "C" fn(&ProcInstJumpArgs));
}
