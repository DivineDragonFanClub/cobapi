#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SaveLoadedArgs {
    pub ty: i32,
    pub slot_id: i32,
}

#[cobapi_macros::service]
pub trait SaveLoadedService {
    fn subscribe(&self, handler: extern "C" fn(&SaveLoadedArgs));
    fn unsubscribe(&self, handler: extern "C" fn(&SaveLoadedArgs));
}
