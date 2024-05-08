pub(super) struct Controller {
    error_counter: u32,
}

impl Default for Controller {
    fn default() -> Self {
        Self {
            error_counter: 0,
        }
    }
}