#[repr(i32)]
pub enum UdoReturn {
    NoError = 0,
    GenericError = 1,
    CacheFail = 2,
    ElevateFail = 3,
}
