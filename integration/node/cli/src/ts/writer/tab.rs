use std::fmt;

pub struct Tab {
    size: u8,
    offset: u8,
}

impl Default for Tab {
    fn default() -> Self {
        Self { size: 4, offset: 0 }
    }
}
impl Tab {
    pub fn inc(&mut self) {
        self.offset = self.offset.saturating_add(1);
    }
    pub fn dec(&mut self) {
        self.offset = self.offset.saturating_sub(1);
    }
}

impl fmt::Display for Tab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", " ".repeat((self.size * self.offset) as usize))
    }
}
