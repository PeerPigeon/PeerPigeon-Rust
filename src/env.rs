#[derive(Clone, Debug)]
pub struct EnvironmentDetector {
    pub is_node: bool,
    pub is_browser: bool,
    pub has_websocket: bool,
}

impl EnvironmentDetector {
    pub fn detect() -> Self {
        Self { is_node: true, is_browser: false, has_websocket: true }
    }
}

