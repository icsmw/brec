use std::fmt;

/// Current indentation settings for generated source files.
///
/// `SourceWriter` owns line handling while `Tab` keeps indentation size and
/// global offset small and explicit.
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

    pub fn size(&self) -> u8 {
        self.size
    }

    pub fn offset(&self) -> u8 {
        self.offset
    }

    pub fn spaces(&self, offset: u8) -> String {
        " ".repeat(self.size as usize * offset as usize)
    }
}

impl fmt::Display for Tab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.spaces(self.offset))
    }
}
