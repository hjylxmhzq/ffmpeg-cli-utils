### ffmpeg_utils_rs

#### Intro

##### Simple input/output

ffmpeg_utils_rs depends on tokio runtime

before running codes below, ffmpeg should be placed in $PATH,
or you can either configure ffmpeg static binary by using `set_binary_path`
or set an env FFMEPG_BIN=path/to/ffmpeg

```rust
fn main() {
  let ffmpeg = FFMpeg::new();
  ffmpeg
      .input_file("./sample.mp4")
      .output()
      .save("./output/output.mp4")
      .unwrap();
}
```

##### Resize to (width, height)

```rust
fn main() {
  let ffmpeg = FFMpeg::new();
  ffmpeg
      .input_file("./sample.mp4")
      .resize(1280, 720)
      .save("./output_720p.mp4")
      .unwrap();
}
```

##### Return AsyncRead and use as stream

```rust
#[tokio::main]
async fn main() {
  let ffmpeg = FFMpeg::new();
  let mut reader = ffmpeg.input_file("./sample.mp4").output().resize(-2, 320).stream().unwrap();
  let mut output_file = tokio::fs::File::create("./output-stream.mp4")
      .await
      .unwrap();
  tokio::io::copy(&mut reader, &mut output_file)
      .await
      .unwrap();
}
```

stream is useful in some realtime cases, e.g. http response:

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

#### Other APIs:

##### Set bitrate

```rust
fn some_route() -> HttpResponse {
  let ffmpeg = FFMpeg::new();
  ffmpeg
      .input_file("./sample.mp4")
      .bitrate(1000)
      .save("./output_720p.mp4")
      .unwrap();
}
```

##### Inspect ffmpeg args

```rust
fn some_route() -> HttpResponse {
  let ffmpeg = FFMpeg::new();
  let args = ffmpeg
      .input_file("./sample.mp4")
      .bitrate(1000)
      .build_args(Some("/path/to/output_file"));
}
```

##### Combine multiple input

```rust
fn main() {
  let start_time = time::Duration::from_secs(30);
  let end_time = time::Duration::from_secs(60);
  
  let input1 = FFMpeg::new()
      .input_file("./audio.mp3")
      .only_audio()
      .start_time(&start_time)
      .end_time(&end_time);

  let input2 = FFMpeg::new().input_file("./sample.mp4").only_video();

  input1
      .concat(&input2)
      .output()
      .resize(-2, 480)
      .save("./combination_output.mp4")
      .unwrap();
}
```
