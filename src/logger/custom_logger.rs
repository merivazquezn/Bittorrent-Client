use log::*;

pub struct CustomLogger {
    pub target: &'static str,
}

impl CustomLogger {
    pub const fn init(target: &'static str) -> CustomLogger {
        CustomLogger { target }
    }

    pub fn info(&self, message: String) {
        info!(target: self.target, "{}", message);
    }

    pub fn info_str(&self, message: &str) {
        info!(target: self.target, "{}", message);
    }

    pub fn error(&self, message: String) {
        error!(target: self.target, "{}", message);
    }
}
