#[enum_ids::enum_ids(display_variant_snake)]
#[derive(Debug, Clone)]
pub enum Setting {
    NoDefaultPayload,
    PayloadsDerive(String),
}

pub struct Config(pub Vec<Setting>);

impl Config {
    pub fn is_no_default_payloads(&self) -> bool {
        self.0
            .iter()
            .any(|attr| matches!(attr, Setting::NoDefaultPayload))
    }
    pub fn get_payload_derive(&self) -> Vec<String> {
        let Some(Setting::PayloadsDerive(derives)) = self
            .0
            .iter()
            .find(|attr| matches!(attr, Setting::PayloadsDerive(..)))
        else {
            return Vec::new();
        };
        derives
            .split(",")
            .map(|s| s.trim().to_owned())
            .collect::<Vec<String>>()
    }
}
