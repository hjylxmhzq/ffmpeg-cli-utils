### ffmpeg_utils_rs

#### samples

##### from input file to output file

before running codes below, ffmpeg should be placed in $PATH,
or you can configure ffmpeg static binary path by using `set_binary_path`

```rust
#[tokio::main]
async fn main() {
  let ffmpeg = FFMpeg::new();
  ffmpeg
      .set_binary_path("./ffmpeg")
      .input_file("./sample.mp4")
      .output("./output.mp4")
      .await
      .unwrap();
}
```

##### resize

```rust
#[tokio::main]
async fn main() {
  let ffmpeg = FFMpeg::new();
  ffmpeg
      .input_file("./sample.mp4")
      .resize((1280, 720))
      .output("./output_720p.mp4")
      .await
      .unwrap();
}
```

##### return AsyncRead and use as stream

```rust
#[tokio::main]
async fn main() {
  let ffmpeg = FFMpeg::new();
  let mut stdout = ffmpeg
      .set_binary_path("./ffmpeg")
      .input_file("./sample.mp4")
      .stream()
      .unwrap();
  let mut output_file = tokio::fs::File::create("./output-stream.mp4")
      .await
      .unwrap();
  tokio::io::copy(&mut stdout, &mut output_file)
      .await
      .unwrap();
}
```

