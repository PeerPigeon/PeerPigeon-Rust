use std::sync::Arc;

#[derive(Clone)]
pub struct DebugLogger {
    name: Arc<String>,
    level: Arc<DebugLevel>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DebugLevel {
    Off,
    Error,
    Warn,
    Info,
    Log,
}

impl DebugLogger {
    pub fn create(name: &str) -> Self {
        Self { name: Arc::new(name.to_string()), level: Arc::new(DebugLevel::Log) }
    }
    pub fn set_level(&self, level: DebugLevel) {
        unsafe { *(Arc::as_ptr(&self.level) as *mut DebugLevel) = level; }
    }
    pub fn error(&self, msg: impl AsRef<str>) { if *self.level as u8 >= DebugLevel::Error as u8 { eprintln!("❌ {}: {}", self.name, msg.as_ref()); } }
    pub fn warn(&self, msg: impl AsRef<str>) { if *self.level as u8 >= DebugLevel::Warn as u8 { println!("⚠️ {}: {}", self.name, msg.as_ref()); } }
    pub fn info(&self, msg: impl AsRef<str>) { if *self.level as u8 >= DebugLevel::Info as u8 { println!("ℹ️ {}: {}", self.name, msg.as_ref()); } }
    pub fn log(&self, msg: impl AsRef<str>) { if *self.level as u8 >= DebugLevel::Log as u8 { println!("{}: {}", self.name, msg.as_ref()); } }
}

