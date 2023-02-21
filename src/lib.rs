mod async_output;
mod error;
mod input;
mod macros;

pub use input::FFMpegInput;

pub struct FFMpeg {}

impl FFMpeg {
    pub fn new() -> FFMpegInput {
        FFMpegInput::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::FFMpeg;
    use std::{process::{self, Stdio}, str::FromStr, sync::Once, fs};

    static ONCE: Once = Once::new();

    fn init() {
        ONCE.call_once(|| {
            if !std::path::PathBuf::from_str("./sample.mp4")
                .unwrap()
                .exists()
            {
                let sample_video_url = "https://media.w3.org/2010/05/sintel/trailer.mp4";
                println!("sample video is not existed, start downloading from {sample_video_url}...");
                let mut child = process::Command::new("curl");
                child
                    .args(&[
                        sample_video_url,
                        "-o",
                        "sample.mp4",
                    ])
                    .output()
                    .unwrap();
                println!("done");
            }

            if !std::path::PathBuf::from_str("./ffmpeg")
                .unwrap()
                .exists()
            {
                let arch = if cfg!(target_arch = "x86") {
                    "ia32"
                } else if cfg!(target_arch = "x86_64") {
                    "x64"
                } else if cfg!(target_arch = "arm") {
                    "arm"
                } else if cfg!(target_arch = "aarch64") {
                    "arm64"
                } else {
                    return;
                };

                let os = if cfg!(target_os = "windows") {
                    "win32"
                } else if cfg!(target_os = "macos") {
                    "darwin"
                } else if cfg!(target_os = "linux") {
                    "linux"
                } else {
                    return;
                };

                let binary_download_url = format!("https://github.com/eugeneware/ffmpeg-static/releases/download/b5.0.1/{os}-{arch}");
                println!("start downloading ffmpeg static build from {}...", binary_download_url);

                let mut child = process::Command::new("curl");
                child
                    .args(&[
                        "-L",
                        &binary_download_url,
                        "-o",
                        "ffmpeg",
                    ])
                    .output()
                    .unwrap();
                println!("done");

                if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
                    use std::os::unix::prelude::PermissionsExt;
                    let mut perm = fs::metadata("./ffmpeg").unwrap().permissions();
                    perm.set_mode(0x744);
                    fs::set_permissions("./ffmpeg", perm).unwrap();
                }
            }
        });
    }

    #[tokio::test]
    async fn output_to_file() {
        init();
        let ffmpeg = FFMpeg::new();
        ffmpeg
            .set_binary_path("./ffmpeg")
            .input_file("./sample.mp4")
            .output_async()
            .save("./output.mp4")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn output_to_stream() {
        init();
        let ffmpeg = FFMpeg::new();
        let mut stdout = ffmpeg
            .set_binary_path("./ffmpeg")
            .input_file("./sample.mp4")
            .output_async()
            .stream()
            .unwrap();
        let mut output_file = tokio::fs::File::create("./output-stream.mp4")
            .await
            .unwrap();
        tokio::io::copy(&mut stdout, &mut output_file)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn resize_to_720() {
        init();
        let ffmpeg = FFMpeg::new();
        ffmpeg
            .set_binary_path("./ffmpeg")
            .input_file("./sample.mp4")
            .output_async()
            .resize(1280, 720)
            .save("./output_720p.mp4")
            .await
            .unwrap();
    }
}
