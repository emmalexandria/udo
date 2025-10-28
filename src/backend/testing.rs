use crate::backend::Backend;

/// This is a [Backend] used for testing udo. It in no way fully simulates a Unix system,
/// but it aims to simulate *enough* to verify that udo has the expected behaviour
pub struct TestingBackend {}

impl Backend for TestingBackend {}
