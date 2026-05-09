use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScrcpyOptions {
    pub(crate) max_size: Option<u32>,
    pub(crate) max_fps: Option<u32>,
    pub(crate) video_bit_rate: Option<String>,
    pub(crate) video_codec: Option<String>,
    pub(crate) no_audio: Option<bool>,
    pub(crate) no_control: Option<bool>,
    pub(crate) stay_awake: Option<bool>,
    pub(crate) turn_screen_off: Option<bool>,
    pub(crate) show_touches: Option<bool>,
    pub(crate) always_on_top: Option<bool>,
    pub(crate) fullscreen: Option<bool>,
}

impl Default for ScrcpyOptions {
    fn default() -> Self {
        Self {
            max_size: Some(1920),
            max_fps: Some(60),
            video_bit_rate: None,
            video_codec: Some("default".to_string()),
            no_audio: Some(true),
            no_control: None,
            stay_awake: Some(true),
            turn_screen_off: None,
            show_touches: None,
            always_on_top: None,
            fullscreen: None,
        }
    }
}

pub(crate) fn build_scrcpy_args(serial: &str, options: &ScrcpyOptions) -> Vec<String> {
    let mut args = vec!["-s".to_string(), serial.to_string()];

    if let Some(max_size) = options.max_size {
        args.push(format!("--max-size={max_size}"));
    }
    if let Some(max_fps) = options.max_fps {
        args.push(format!("--max-fps={max_fps}"));
    }
    if let Some(video_bit_rate) = &options.video_bit_rate {
        if !video_bit_rate.trim().is_empty() {
            args.push(format!("--video-bit-rate={}", video_bit_rate.trim()));
        }
    }
    if let Some(video_codec) = &options.video_codec {
        if !video_codec.trim().is_empty() && video_codec != "default" {
            args.push(format!("--video-codec={}", video_codec.trim()));
        }
    }
    if options.no_audio.unwrap_or(false) {
        args.push("--no-audio".to_string());
    }
    if options.no_control.unwrap_or(false) {
        args.push("--no-control".to_string());
    }
    if options.stay_awake.unwrap_or(false) {
        args.push("--stay-awake".to_string());
    }
    if options.turn_screen_off.unwrap_or(false) {
        args.push("--turn-screen-off".to_string());
    }
    if options.show_touches.unwrap_or(false) {
        args.push("--show-touches".to_string());
    }
    if options.always_on_top.unwrap_or(false) {
        args.push("--always-on-top".to_string());
    }
    if options.fullscreen.unwrap_or(false) {
        args.push("--fullscreen".to_string());
    }

    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_scrcpy_args_in_stable_order() {
        let options = ScrcpyOptions {
            max_size: Some(1920),
            max_fps: Some(60),
            video_bit_rate: Some("4M".to_string()),
            video_codec: Some("h265".to_string()),
            no_audio: Some(true),
            no_control: None,
            stay_awake: Some(true),
            turn_screen_off: None,
            show_touches: None,
            always_on_top: Some(true),
            fullscreen: None,
        };

        assert_eq!(
            build_scrcpy_args("R9YT301WXXX", &options),
            vec![
                "-s".to_string(),
                "R9YT301WXXX".to_string(),
                "--max-size=1920".to_string(),
                "--max-fps=60".to_string(),
                "--video-bit-rate=4M".to_string(),
                "--video-codec=h265".to_string(),
                "--no-audio".to_string(),
                "--stay-awake".to_string(),
                "--always-on-top".to_string(),
            ]
        );
    }
}
