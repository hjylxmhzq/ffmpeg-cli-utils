use std::process::Stdio;

use crate::{error::Error, input::FFMpegInput, owned};

#[cfg(feature = "async")]
use tokio::{
    io::{duplex, AsyncReadExt, AsyncWriteExt, DuplexStream},
    process,
};

pub struct FFmpegOutput {
    output_option: OutputOption,
    input: FFMpegInput,
}

#[derive(Clone)]
struct OutputOption {
    stream_buffer_size: usize,
    size: Option<(i32, i32)>,
    bitrate: Option<u64>,
    framerate: Option<u64>,
}

pub struct SpawnResult {
    pub stdout: String,
    pub stderr: String,
}

impl FFmpegOutput {
    pub fn new(ffmpeg_input: FFMpegInput) -> Self {
        FFmpegOutput {
            output_option: OutputOption {
                size: None,
                bitrate: None,
                framerate: None,
                stream_buffer_size: 1024,
            },
            input: ffmpeg_input,
        }
    }
    pub fn save(&self, file: &str) -> Result<SpawnResult, Error> {
        let ffmpeg_bin = self.input.get_ffmpeg_bin()?;

        let args = self.build_args(Some(file.to_owned()))?;
        println!("{ffmpeg_bin} {args:?}");
        let child = std::process::Command::new(ffmpeg_bin)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .unwrap();

        let stdout = child.stdout;
        let stderr = child.stderr;

        let stdout = String::from_utf8_lossy(&stdout).into_owned();
        let stderr = String::from_utf8_lossy(&stderr).into_owned();

        Ok(SpawnResult { stderr, stdout })
    }
    #[cfg(feature = "async")]
    pub async fn async_save(&self, file: &str) -> Result<String, Error> {
        let ffmpeg_bin = self.input.get_ffmpeg_bin()?;

        let args = self.build_args(Some(file.to_owned()))?;
        println!("{ffmpeg_bin} {args:?}");
        let mut child = process::Command::new(ffmpeg_bin)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut err_buf = String::with_capacity(100);
        let mut out_buf = String::with_capacity(100);
        let mut out = String::new();
        let mut err = String::new();

        let stdout = child.stdout.as_mut().unwrap();
        let stderr = child.stderr.as_mut().unwrap();

        while let Ok(size) = stdout.read_to_string(&mut out_buf).await {
            if size == 0 {
                break;
            }
            out.push_str(&out_buf[0..size]);
        }

        while let Ok(size) = stderr.read_to_string(&mut err_buf).await {
            if size == 0 {
                break;
            }
            err.push_str(&err_buf[0..size]);
        }

        child.wait().await.map_err(|_| Error { msg: err })?;
        Ok(out)
    }

    pub fn build_args(&self, output_file: Option<String>) -> Result<Vec<String>, Error> {
        let mut args = self.input.build_args()?;

        if let Some(size) = self.output_option.size {
            args.append(&mut owned![
                "-filter:v",
                &format!("scale={}:{}", size.0, size.1)
            ]);
        }

        if let Some(bitrate) = self.output_option.bitrate {
            args.append(&mut owned!["-b:v", &bitrate.to_string()]);
        }

        if let Some(framerate) = self.output_option.framerate {
            args.append(&mut owned!["-r", &framerate.to_string()]);
        }

        let mut format_args = owned!["-movflags", "frag_keyframe+empty_moov", "-f", "mp4"];

        let mut output_method_args = if let Some(output_file) = output_file {
            owned![output_file]
        } else {
            owned!["pipe:1"]
        };

        args.append(&mut format_args);

        args.append(&mut output_method_args);

        Ok(args)
    }
    pub fn set_buffer_size(mut self, size: usize) -> Self {
        self.output_option.stream_buffer_size = size;
        self
    }
    pub fn set_bitrate(mut self, bitrate: u64) -> Self {
        self.output_option.bitrate = Some(bitrate);
        self
    }
    pub fn set_framerate(mut self, framerate: u64) -> Self {
        self.output_option.framerate = Some(framerate);
        self
    }
    pub fn resize(mut self, width: i32, height: i32) -> Self {
        self.output_option.size = Some((width, height));
        self
    }
    #[cfg(feature = "async")]
    pub fn stream(&self) -> Result<DuplexStream, Error> {
        let (mut w, r) = duplex(self.output_option.stream_buffer_size);
        let ffmpeg_bin = self.input.get_ffmpeg_bin()?;
        let args = self.build_args(Option::<String>::None)?;
        tokio::spawn(async move {
            let mut child = process::Command::new(ffmpeg_bin)
                .args(args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();
            let mut stdout = child.stdout.take().unwrap();
            let mut buf = [0; 1024];

            while let Ok(size) = stdout.read(&mut buf).await {
                if size == 0 {
                    break;
                }
                let r = w.write_all(&buf[0..size]).await;
                if let Err(_) = r {
                    child.kill().await.unwrap();
                    break;
                }
            }
        });
        Ok(r)
    }
}
