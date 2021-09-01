#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Input {
    pub up: bool,
    pub down: bool,
    pub reset: bool,
}
