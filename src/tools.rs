use crate::FFMpeg;
use std::time;

/// screenshot of input video
///
/// alias of FFMpeg::input(file).start_time(time).output().args(vec!["-frames:v", "1"])
/// samples:  
/// ```
/// use ffmpeg_cli_utils::tools;
/// use std::time;
/// # use ffmpeg_cli_utils::FFMpeg;
/// # FFMpeg::set_ffmpeg_bin("./ffmpeg");
/// tools::screenshot("./sample.mp4", &time::Duration::from_secs(30))
///    .resize(-2, 320)
///    .save("./output/screenshot.jpg")
///    .unwrap();
/// ```
pub fn screenshot(file: &str, time: &time::Duration) -> crate::output::FFmpegOutput {
    FFMpeg::input(file)
        .start_time(time)
        .output()
        .args(vec!["-frames:v", "1"])
}

/// capture screen from input device
///
/// samples:  
/// ```
/// use ffmpeg_cli_utils::tools;
/// use std::time;
/// # use ffmpeg_cli_utils::FFMpeg;
/// # FFMpeg::set_ffmpeg_bin("./ffmpeg");
/// tools::capture_screen().timeout(3).save("./output/capture.mkv").unwrap();
/// ```
pub fn capture_screen() -> crate::output::FFmpegOutput {
    let input_device = if cfg!(target_os = "windows") {
        r#"video="screen-capture-recorder""#
    } else if cfg!(target_os = "macos") {
        "Capture screen"
    } else if cfg!(target_os = "linux") {
        ":0.0+100,200"
    } else {
        panic!("unsupport platform");
    };

    let input_format = if cfg!(target_os = "windows") {
        "dshow"
    } else if cfg!(target_os = "macos") {
        "avfoundation"
    } else if cfg!(target_os = "linux") {
        "x11grab"
    } else {
        panic!("unsupport platform");
    };

    FFMpeg::input(input_device)
        .format(input_format)
        .output()
}

