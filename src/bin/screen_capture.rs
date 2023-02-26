use ffmpeg_cli_utils::FFMpeg;

fn main() {
  FFMpeg::set_ffmpeg_bin("./ffmpeg");
  ffmpeg_cli_utils::tools::capture_screen().timeout(3).save("./output/capture.mkv").unwrap();
}
