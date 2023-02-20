### ffmpeg_utils_rs

#### samples

##### Simple input/output

ffmpeg_utils_rs depends on tokio runtime

before running codes below, ffmpeg should be placed in $PATH,
or you can configure ffmpeg static binary by using `set_binary_path`

```rust
#[tokio::main]
async fn main() {
  let ffmpeg = FFMpeg::new();
  ffmpeg
      .set_binary_path("./ffmpeg") // this line is not necessary if ffmpeg can be invoked globally
      .input_file("./sample.mp4")
      .output("./output.mp4")
      .await
      .unwrap();
}
```

##### Resize to (width, height)

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

##### Return AsyncRead and use as stream

```rust
#[tokio::main]
async fn main() {
  let ffmpeg = FFMpeg::new();
  let mut reader = ffmpeg
      .set_binary_path("./ffmpeg")
      .input_file("./sample.mp4")
      .stream()
      .unwrap();
  let mut output_file = tokio::fs::File::create("./output-stream.mp4")
      .await
      .unwrap();
  tokio::io::copy(&mut reader, &mut output_file)
      .await
      .unwrap();
}
```

stream is useful in some realtime cases, e.g. in actix-web:

```rust
fn some_route() -> HttpResponse {
  let mut reader = ffmpeg
      .set_binary_path("./ffmpeg")
      .input_file("./sample.mp4")
      .stream()
      .unwrap();
  let reader_stream = tokio_util::io::ReaderStream::new(reader);
  HttpResponse::Ok().streaming(reader_stream)
}
```

// TODO: support more options
