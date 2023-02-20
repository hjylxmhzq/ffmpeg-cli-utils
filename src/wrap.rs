use std::{env, path::Path, process::Stdio};

use tokio::{
    io::{duplex, AsyncReadExt, AsyncWriteExt, DuplexStream},
    process,
};

#[derive(Clone)]
pub struct FFMpeg {
    bin_path: Option<String>,
    input_file: Option<String>,
    buffer_size: usize,
    output_option: OutputOption,
}

#[derive(Clone)]
struct OutputOption {
    size: Option<(u64, u64)>,
    bitrate: Option<u64>,
}

#[derive(Debug)]
pub struct Error {
    pub msg: String,
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self { msg: e.to_string() }
    }
}

impl FFMpeg {
    pub fn new() -> Self {
        let bin_path = env::var("FFMPEG_BINARY").ok();
        return Self {
            buffer_size: 1024,
            bin_path,
            input_file: None,
            output_option: OutputOption {
                size: None,
                bitrate: None,
            },
        };
    }
    pub fn set_binary_path(mut self, bin_path: &str) -> Self {
        self.bin_path = Some(bin_path.to_owned());
        self
    }
    pub fn input_file(mut self, file: impl AsRef<Path>) -> Self {
        let file = Some(file.as_ref().to_string_lossy().into_owned());
        self.input_file = file;
        self
    }
    pub fn set_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }
    pub fn set_bitrate(mut self, bitrate: u64) -> Self {
        self.output_option.bitrate = Some(bitrate);
        self
    }
    pub fn resize(mut self, size: (u64, u64)) -> Self {
        self.output_option.size = Some(size);
        self
    }
    pub fn stream(&self) -> Result<DuplexStream, Error> {
        let (mut w, r) = duplex(self.buffer_size);
        let ffmpeg_bin = self.bin_path.clone().unwrap();
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
    fn get_input_file(&self) -> Result<String, Error> {
        let file = self
            .input_file
            .as_ref()
            .ok_or(Error {
                msg: "input file is not set".to_owned(),
            })?
            .clone();
        Ok(file)
    }
    fn get_ffmpeg_bin(&self) -> Result<String, Error> {
        let file = self
            .bin_path
            .as_ref()
            .ok_or(Error {
                msg: "ffmpeg binary is not set".to_owned(),
            })?
            .clone();
        Ok(file)
    }
    pub fn build_args(&self, output_file: Option<String>) -> Result<Vec<String>, Error> {
        use crate::owned;

        let abs_file = self.get_input_file()?;
        let mut output_args = if let Some(output_file) = output_file {
            vec![output_file]
        } else {
            vec!["pipe:1".to_owned()]
        };

        let mut args = owned!(vec!["-y", "-i", &abs_file]);

        if let Some(size) = self.output_option.size {
            args.append(&mut owned!(vec![
                "-filter:v",
                &format!("scale={}:{}", size.0, size.1)
            ]));
        }

        if let Some(bitrate) = self.output_option.bitrate {
            args.append(&mut owned!(vec!["-b:v", &bitrate.to_string()]))
        }

        let mut format_args = owned!(vec!["-movflags", "frag_keyframe+empty_moov", "-f", "mp4"]);

        args.append(&mut format_args);

        args.append(&mut output_args);

        Ok(args)
    }
    pub async fn output(&self, file: &str) -> Result<String, Error> {
        let ffmpeg_bin = self.get_ffmpeg_bin()?;

        let args = self.build_args(Some(file.to_owned()))?;

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
}

#[macro_export]
macro_rules! owned {
    ($key:expr) => {
        $key.into_iter()
            .map(|item| item.to_owned())
            .collect::<Vec<_>>()
    };
}
