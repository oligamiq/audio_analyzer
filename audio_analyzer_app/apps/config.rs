/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)]
pub struct Config {}

impl Default for Config {
    fn default() -> Self {
        Self {}
    }
}
