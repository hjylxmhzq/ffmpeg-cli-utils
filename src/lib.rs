mod wrap;

pub use wrap::FFMpeg;

#[cfg(test)]
mod tests {
    use crate::wrap::FFMpeg;
    #[tokio::test]
    async fn output_to_file() {
        let ffmpeg = FFMpeg::new();
        ffmpeg
            .set_binary_path("./ffmpeg")
            .input_file("./sample.mp4")
            .output("./output.mp4")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn output_to_stream() {
        let ffmpeg = FFMpeg::new();
        let mut stdout = ffmpeg
            .set_binary_path("./ffmpeg")
            .input_file("./sample.mp4")
            .stream().unwrap();
        let mut output_file = tokio::fs::File::create("./output-stream.mp4").await.unwrap();
        tokio::io::copy(&mut stdout, &mut output_file).await.unwrap();
    }
    
    #[tokio::test]
    async fn resize_to_720() {
        let ffmpeg = FFMpeg::new();
        ffmpeg
            .set_binary_path("./ffmpeg")
            .input_file("./sample.mp4")
            .resize((1280, 720))
            .output("./output_720p.mp4")
            .await
            .unwrap();
    }
}

