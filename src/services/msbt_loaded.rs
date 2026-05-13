#[cobapi_macros::service]
pub trait MsbtLoadedService {
    fn subscribe(&self, handler: extern "C" fn());
    fn unsubscribe(&self, handler: extern "C" fn());
}
