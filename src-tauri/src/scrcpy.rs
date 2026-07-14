use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScrcpyCapabilities {
    pub(crate) supports_keep_active: bool,
    pub(crate) supports_background_color: bool,
    pub(crate) supports_window_aspect_ratio_lock: bool,
}

impl ScrcpyCapabilities {
    pub(crate) fn scrcpy_4() -> Self {
        Self {
            supports_keep_active: true,
            supports_background_color: true,
            supports_window_aspect_ratio_lock: true,
        }
    }
}

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
    pub(crate) keep_active: Option<bool>,
    pub(crate) background_color: Option<String>,
    pub(crate) window_aspect_ratio_lock: Option<bool>,
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
            stay_awake: Some(false),
            turn_screen_off: None,
            show_touches: None,
            always_on_top: None,
            fullscreen: None,
            keep_active: Some(true),
            background_color: None,
            window_aspect_ratio_lock: Some(true),
        }
    }
}

impl ScrcpyOptions {
    #[cfg(test)]
    pub(crate) fn empty() -> Self {
        Self {
            max_size: None,
            max_fps: None,
            video_bit_rate: None,
            video_codec: None,
            no_audio: None,
            no_control: None,
            stay_awake: None,
            turn_screen_off: None,
            show_touches: None,
            always_on_top: None,
            fullscreen: None,
            keep_active: None,
            background_color: None,
            window_aspect_ratio_lock: None,
        }
    }
}

pub(crate) fn normalize_background_color(value: &str) -> Option<String> {
    let color = value.trim().trim_start_matches('#');
    let valid = (color.len() == 3 || color.len() == 6)
        && color.chars().all(|character| character.is_ascii_hexdigit());
    if !valid {
        return None;
    }

    let color = color.to_ascii_lowercase();
    if color.len() == 3 {
        let mut expanded = String::from("#");
        for character in color.chars() {
            expanded.push(character);
            expanded.push(character);
        }
        Some(expanded)
    } else {
        Some(format!("#{color}"))
    }
}

pub(crate) fn build_scrcpy_args_with_capabilities(
    serial: &str,
    options: &ScrcpyOptions,
    capabilities: ScrcpyCapabilities,
) -> Result<Vec<String>, String> {
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
    if options.keep_active.unwrap_or(false) {
        if !capabilities.supports_keep_active {
            return Err("当前 scrcpy 版本低于 4.0，暂不支持保持活跃设置。请在工具配置中升级 scrcpy。".to_string());
        }
        args.push("--keep-active".to_string());
    }
    if let Some(background_color) = options.background_color.as_deref() {
        if !background_color.trim().is_empty() {
            if !capabilities.supports_background_color {
                return Err("当前 scrcpy 版本低于 4.0，暂不支持背景色设置。请在工具配置中升级 scrcpy。".to_string());
            }
            let color = normalize_background_color(background_color)
                .ok_or_else(|| "背景色格式不正确，请使用 #RGB 或 #RRGGBB。".to_string())?;
            args.push(format!("--background-color={color}"));
        }
    }
    if options.window_aspect_ratio_lock == Some(false) {
        if !capabilities.supports_window_aspect_ratio_lock {
            return Err("当前 scrcpy 版本低于 4.0，暂不支持窗口比例锁设置。请在工具配置中升级 scrcpy。".to_string());
        }
        args.push("--no-window-aspect-ratio-lock".to_string());
    }

    Ok(args)
}

#[cfg(test)]
pub(crate) fn build_scrcpy_args(serial: &str, options: &ScrcpyOptions) -> Vec<String> {
    build_scrcpy_args_with_capabilities(serial, options, ScrcpyCapabilities::default())
        .unwrap_or_else(|_| vec!["-s".to_string(), serial.to_string()])
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
            keep_active: None,
            background_color: None,
            window_aspect_ratio_lock: None,
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

    #[test]
    fn normalizes_background_colors() {
        assert_eq!(normalize_background_color("#abc"), Some("#aabbcc".to_string()));
        assert_eq!(normalize_background_color("567"), Some("#556677".to_string()));
        assert_eq!(normalize_background_color("#AABBCC"), Some("#aabbcc".to_string()));
        assert_eq!(normalize_background_color("red"), None);
        assert_eq!(normalize_background_color("#12"), None);
        assert_eq!(normalize_background_color("#12345g"), None);
    }

    #[test]
    fn builds_scrcpy_4_args_when_supported() {
        let options = ScrcpyOptions {
            keep_active: Some(true),
            background_color: Some("567".to_string()),
            window_aspect_ratio_lock: Some(false),
            ..ScrcpyOptions::empty()
        };

        assert_eq!(
            build_scrcpy_args_with_capabilities(
                "SERIAL",
                &options,
                ScrcpyCapabilities::scrcpy_4()
            )
            .unwrap(),
            vec![
                "-s".to_string(),
                "SERIAL".to_string(),
                "--keep-active".to_string(),
                "--background-color=#556677".to_string(),
                "--no-window-aspect-ratio-lock".to_string(),
            ]
        );
    }

    #[test]
    fn rejects_scrcpy_4_args_when_unsupported() {
        let options = ScrcpyOptions {
            keep_active: Some(true),
            ..ScrcpyOptions::empty()
        };

        assert!(build_scrcpy_args_with_capabilities(
            "SERIAL",
            &options,
            ScrcpyCapabilities::default()
        )
        .unwrap_err()
        .contains("scrcpy 版本低于 4.0"));
    }

    #[test]
    fn rejects_invalid_background_color() {
        let options = ScrcpyOptions {
            background_color: Some("red".to_string()),
            ..ScrcpyOptions::empty()
        };

        assert_eq!(
            build_scrcpy_args_with_capabilities(
                "SERIAL",
                &options,
                ScrcpyCapabilities::scrcpy_4()
            )
            .unwrap_err(),
            "背景色格式不正确，请使用 #RGB 或 #RRGGBB。"
        );
    }
}
