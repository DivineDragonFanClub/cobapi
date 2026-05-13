#[cobapi_macros::service]
pub trait LanguageChangedService {
    fn subscribe(&self, handler: extern "C" fn());
    fn unsubscribe(&self, handler: extern "C" fn());
}
