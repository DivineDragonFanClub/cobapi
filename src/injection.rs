use unity2::il2cpp::Il2CppClass;
use unity2::injection::{build, InjectedClass};
use unity2::Class;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum RegisterClassError {
    #[error("the built class pointer was null")]
    NullClass,
    #[error("a class with this namespace+name is already registered")]
    AlreadyRegistered,
    #[error("unrecognized class-registration error code {0}")]
    Unknown(i32),
}

extern "C" {
    fn cobapi_register_injected_class(class: *mut Il2CppClass) -> i32;
}

pub fn register<T: InjectedClass>() -> Result<Class, RegisterClassError> {
    let class = build::<T>();

    let code = unsafe { cobapi_register_injected_class(class.raw_mut()) };

    match code {
        0 => {
            T::fill_cache(class);
            Ok(class)
        }
        1 => Err(RegisterClassError::NullClass),
        2 => Err(RegisterClassError::AlreadyRegistered),
        other => Err(RegisterClassError::Unknown(other)),
    }
}
