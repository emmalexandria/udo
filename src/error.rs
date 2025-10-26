#[repr(i32)]
pub enum UdoReturn {
    NoError = 0,
    GenericError = 1,
    CacheFailure = 2,
    ElevateFailure = 3,
    AuthenticateFailure = 4,
}
