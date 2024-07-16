use serde::Deserialize;

use super::util::empty_string_as_none;

pub struct Current {
    pub process_id: u32,
    pub erogame_scape_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct ScreenshotParams {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    src: Option<String>,
}

pub enum ScreenshotSource {
    ProcessId,
    TopWindow,
}

impl From<&ScreenshotParams> for ScreenshotSource {
    fn from(params: &ScreenshotParams) -> Self {
        match &params.src {
            Some(s) => match s.as_str() {
                "process_id" => ScreenshotSource::ProcessId,
                "top_window" => ScreenshotSource::TopWindow,
                _ => ScreenshotSource::TopWindow,
            },
            None => ScreenshotSource::TopWindow,
        }
    }
}
