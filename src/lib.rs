//! ffmpeg_cli_utils
//! Provide rust style ffmpeg control APIs
//!
//! ## Intro

//! ### Simple input/output
//! ffmpeg_utils_rs depends on tokio runtime
//! before running codes below, ffmpeg should be placed in $PATH,
//! or you can either configure ffmpeg static binary by using `set_binary_path`
//! or set an env FFMEPG_BIN=path/to/ffmpeg

//! ```rust
//! fn main() {
//!   let ffmpeg = FFMpeg::new();
//!   ffmpeg
//!       .input_file("./sample.mp4")
//!       .output()
//!       .save("./output/output.mp4")
//!       .unwrap();
//! }
//! ```

//! ### Resize to (width, height)

//! ```rust
//! fn main() {
//!   let ffmpeg = FFMpeg::new();
//!   ffmpeg
//!       .input_file("./sample.mp4")
//!       .resize(1280, 720)
//!       .save("./output_720p.mp4")
//!       .unwrap();
//! }
//! ```

//! ### Return AsyncRead and use as stream

//! ```rust
//! #[tokio::main]
//! async fn main() {
//!   let ffmpeg = FFMpeg::new();
//!   let mut reader = ffmpeg.input_file("./sample.mp4").output().resize(-2, 320).stream().unwrap();
//!   let mut output_file = tokio::fs::File::create("./output-stream.mp4")
//!       .await
//!       .unwrap();
//!   tokio::io::copy(&mut reader, &mut output_file)
//!       .await
//!       .unwrap();
//! }
//! ```

//! stream is useful in some realtime cases, e.g. http response:

//! ```rust
//! fn some_route() -> HttpResponse {
//!   let mut reader = ffmpeg
//!       .set_binary_path("./ffmpeg")
//!       .input_file("./sample.mp4")
//!       .stream()
//!       .unwrap();
//!   let reader_stream = tokio_util::io::ReaderStream::new(reader);
//!   HttpResponse::Ok().streaming(reader_stream)
//! }
//! ```

//! ## Other APIs:

//! ### Set bitrate

//! ```rust
//! fn some_route() -> HttpResponse {
//!   let ffmpeg = FFMpeg::new();
//!   ffmpeg
//!       .input_file("./sample.mp4")
//!       .bitrate(1000)
//!       .save("./output_720p.mp4")
//!       .unwrap();
//! }
//! ```

//! ### Inspect ffmpeg args

//! ```rust
//! fn some_route() -> HttpResponse {
//!   let ffmpeg = FFMpeg::new();
//!   let args = ffmpeg
//!       .input_file("./sample.mp4")
//!       .bitrate(1000)
//!       .build_args(Some("/path/to/output_file"));
//! }
//! ```

//! ### Combine multiple input

//! ```rust
//! fn main() {
//!   let start_time = time::Duration::from_secs(30);
//!   let end_time = time::Duration::from_secs(60);

//!   let input1 = FFMpeg::new()
//!       .input_file("./audio.mp3")
//!       .only_audio()
//!       .start_time(&start_time)
//!       .end_time(&end_time);

//!   let input2 = FFMpeg::new().input_file("./sample.mp4").only_video();

//!   input1
//!       .concat(&input2)
//!       .output()
//!       .resize(-2, 480)
//!       .save("./combination_output.mp4")
//!       .unwrap();
//! }
//! ```

mod error;
mod input;
mod macros;
mod output;
mod utils;

use std::sync::Mutex;

pub use input::FFMpegInput;
pub use input::FFMpegMultipleInput;
pub mod tools;

pub struct FFMpeg {}
use lazy_static::lazy_static;

lazy_static! {
    pub static ref BIN_PATH: Mutex<String> = {
        let default = String::from("ffmpeg");
        let mutex = Mutex::new(default);
        mutex
    };
}

///! FFMpeg cli utils
///! samples:
///! ```
///! use ffmpeg_cli_utils::FFMpeg;
///! FFMpeg::set_ffmpeg_bin("./ffmpeg"); //! not necessary
///
///! let ffmpeg = FFMpeg::new();
///! ffmpeg
///!     .input_file("./sample.mp4")
///!     .output()
///!     .resize(-2, 480)
///!     .save("./output/output_480p.mp4")
///!     .unwrap();
///! ```
impl FFMpeg {
    pub fn new() -> FFMpegInput {
        FFMpegInput::new()
    }
    pub fn input(file: &str) -> FFMpegInput {
        FFMpegInput::input(file)
    }
    pub fn set_ffmpeg_bin(bin_path: &str) {
        let mut s = BIN_PATH.lock().unwrap();
        s.clone_from(&bin_path.to_owned());
    }
    pub(crate) fn get_ffmpeg_bin() -> String {
        let s = BIN_PATH.lock().unwrap();
        s.to_owned()
    }
}

#[cfg(test)]
mod tests {

    use crate::{input::FFMpegMultipleInput, tools, FFMpeg};
    use std::{fs, process, str::FromStr, sync::Once, time};

    static ONCE: Once = Once::new();

    fn init() {
        ONCE.call_once(|| {
            FFMpeg::set_ffmpeg_bin("./ffmpeg");
            std::fs::create_dir_all("./output").unwrap();
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
            .input_file("./sample.mp4")
            .output()
            .async_save("./output/output.mp4")
            .await
            .unwrap();
    }

    #[test]
    fn build_args() {
        init();
        let ffmpeg = FFMpeg::new();
        ffmpeg
            .input_file("./sample.mp4")
            .output()
            .build_args(None)
            .unwrap();
    }

    #[test]
    fn merge_videos() {
        init();
        let start_time = time::Duration::from_secs(30);
        let end_time = time::Duration::from_secs(60);

        let input1 = FFMpeg::new()
            .input_file("./audio.oggl")
            .only_audio()
            .start_time(&start_time)
            .end_time(&end_time);

        let input2 = FFMpeg::new().input_file("./sample.mp4").only_video();

        input1
            .merge(&input2)
            .output()
            .resize(-2, 480)
            .save("./output/combination_output.mp4")
            .unwrap();
    }

    #[test]
    fn concat_videos() {
        init();

        let input1 = "./sample.mp4";
        let input2 = "./sample1.mp4";

        let mut concat_output = FFMpegMultipleInput::concat(&[input1, input2]).output();
        concat_output.save("./output/concat_videos.mp4").unwrap();
    }

    #[test]
    fn screenshot() {
        init();
        tools::screenshot("./sample.mp4", &time::Duration::from_secs(30))
            .resize(-2, 320)
            .save("./output/screenshot.jpg")
            .unwrap();
    }

    #[test]
    fn capture_screen() {
        init();
        tools::capture_screen().timeout(3).save("./output/capture.mkv").unwrap();
    }

    #[test]
    fn concat_and_resize_videos() {
        init();

        let input1 = "./sample.mp4";
        let input2 = "./sample1.mp4";

        let concat_output = FFMpegMultipleInput::concat(&[input1, input2]).output();
        concat_output
            .resize(-2, 320)
            .save("./output/concat_320p.mp4")
            .unwrap();
    }

    #[test]
    fn concat_and_reformat_videos() {
        init();

        let input1 = "./sample.mp4";
        let input2 = "./sample1.mp4";

        let concat_output = FFMpegMultipleInput::concat(&[input1, input2]).output();
        concat_output
            .format("avi")
            .save("./output/concat_videos.avi")
            .unwrap();
    }

    #[tokio::test]
    async fn output_to_stream() {
        init();
        let now = time::Instant::now();
        let ffmpeg = FFMpeg::new();
        let mut stdout = ffmpeg
            .input_file("./sample.mp4")
            .output()
            .resize(-2, 320)
            .stream()
            .unwrap();
        let mut output_file = tokio::fs::File::create("./output/output-stream.mp4")
            .await
            .unwrap();

        tokio::io::copy(&mut stdout, &mut output_file)
            .await
            .unwrap();
        let d = now.elapsed();
        println!("stream time cost: {d:?}");
    }

    #[tokio::test]
    async fn resize_to_320p() {
        init();
        let now = time::Instant::now();
        let ffmpeg = FFMpeg::new();
        ffmpeg
            .input_file("./sample.mp4")
            .output()
            .resize(-2, 320)
            .save("./output/output_320p.mp4")
            .unwrap();
        let d = now.elapsed();
        println!("sync save time cost: {d:?}");
    }
}
