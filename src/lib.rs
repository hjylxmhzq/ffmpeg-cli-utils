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
    #[tokio::test]
    async fn output_to_file() {
        let ffmpeg = FFMpeg::new();
        ffmpeg
            .input_file("./sample.mp4")
            .output_async()
            .save("./output.mp4")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn output_to_stream() {
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
