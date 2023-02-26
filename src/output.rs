use std::{cmp, io::Write, pin::Pin, process::Stdio, task::Poll, path::PathBuf, str::FromStr};

use crate::{
    error::Error,
    input::{FFMpegMultipleInput, MergeStrategy, StreamType},
    owned, FFMpeg,
};

use tempfile::NamedTempFile;
use tokio::{io::AsyncRead, sync::mpsc::Receiver};
#[cfg(feature = "async")]
use tokio::{io::AsyncReadExt, process};

pub struct FFmpegOutput {
    output_option: OutputOption,
    pub(crate) inputs: FFMpegMultipleInput,
}

struct OutputOption {
    stream_buffer_size: usize,
    size: Option<(i32, i32)>,
    bitrate: Option<u64>,
    framerate: Option<u64>,
    format: Option<String>,
    custom_args: Vec<String>,
    video_filters: Vec<String>,
    audio_filters: Vec<String>,
    temp_input_filelist: Option<NamedTempFile>,
}

pub struct SpawnResult {
    pub stdout: String,
    pub stderr: String,
}

impl FFmpegOutput {
    pub fn new(ffmpeg_input: FFMpegMultipleInput) -> Self {
        FFmpegOutput {
            output_option: OutputOption {
                size: None,
                bitrate: None,
                framerate: None,
                stream_buffer_size: 1024 * 10,
                format: Some("mp4".to_owned()),
                custom_args: owned!["-y", "-hide_banner", "-loglevel", "error"],
                video_filters: vec![],
                audio_filters: vec![],
                temp_input_filelist: None,
            },
            inputs: ffmpeg_input,
        }
    }

    pub fn args(mut self, args: Vec<impl AsRef<str>>) -> Self {
        for arg in args {
            let arg = arg.as_ref();
            self.output_option.custom_args.push(arg.to_owned());
        }
        self
    }

    pub fn save(&mut self, file: &str) -> Result<SpawnResult, Error> {
        let ffmpeg_bin = FFMpeg::get_ffmpeg_bin();

        let args = self.build_args(Some(file.to_owned()))?;
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
        if !stderr.is_empty() {
            return Err(Error { msg: stderr });
        }
        Ok(SpawnResult { stderr, stdout })
    }
    #[cfg(feature = "async")]
    pub async fn async_save(&mut self, file: &str) -> Result<String, Error> {
        let ffmpeg_bin = FFMpeg::get_ffmpeg_bin();

        let args = self.build_args(Some(file.to_owned()))?;
        println!("{ffmpeg_bin} {args:?}");
        let child = process::Command::new(ffmpeg_bin)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut err_buf = String::with_capacity(100);
        let mut out_buf = String::with_capacity(100);

        let mut stdout = child.stdout.unwrap();
        let mut stderr = child.stderr.unwrap();

        let out = tokio::spawn(async move {
            let mut out = String::new();
            while let Ok(size) = stdout.read_to_string(&mut out_buf).await {
                if size == 0 {
                    break;
                }
                out.push_str(&out_buf[0..size]);
            }
            out
        });
        let err = tokio::spawn(async move {
            let mut err = String::new();
            while let Ok(size) = stderr.read_to_string(&mut err_buf).await {
                if size == 0 {
                    break;
                }
                err.push_str(&err_buf[0..size]);
            }
            err
        });

        let err = err.await.unwrap();
        let out = out.await.unwrap();
        if !err.is_empty() {
            return Err(Error { msg: err });
        }
        Ok(out)
    }

    pub fn video_filter(mut self, f: &str) -> Self {
        self.output_option.video_filters.push(f.to_owned());
        self
    }

    pub fn video_filters(mut self, filters: &[&str]) -> Self {
        let mut filters: Vec<String> = filters.iter().map(|item| (*item).to_owned()).collect();
        self.output_option.video_filters.append(&mut filters);
        self
    }

    pub fn audio_filter(mut self, f: &str) -> Self {
        self.output_option.audio_filters.push(f.to_owned());
        self
    }

    pub fn audio_filters(mut self, filters: &[&str]) -> Self {
        let mut filters: Vec<String> = filters.iter().map(|item| (*item).to_owned()).collect();
        self.output_option.audio_filters.append(&mut filters);
        self
    }

    pub fn build_args(&mut self, output_file: Option<String>) -> Result<Vec<String>, Error> {
        let inputs = &self.inputs.inputs;
        let merge_strategy = &self.inputs.merge_strategy;

        let mut input_args: Vec<String> = owned![];
        let mut output_args: Vec<String> = owned![];

        if let MergeStrategy::Merge = merge_strategy {
            for (idx, input) in inputs.iter().enumerate() {
                let mut args = input.build_args().unwrap();
                let stream_index = if let Some(s_idx) = input.stream_index {
                    format!(":{s_idx}")
                } else {
                    "".to_owned()
                };
                match input.stream_type {
                    StreamType::Audio => {
                        output_args.append(&mut owned!["-map", &format!("{idx}:a{stream_index}")]);
                    }
                    StreamType::Video => {
                        output_args.append(&mut owned!["-map", &format!("{idx}:v{stream_index}")]);
                    }
                    _ => (),
                }
                input_args.append(&mut args);
            }
        } else {
            input_args.append(&mut owned!["-f", "concat", "-safe", "0", "-i"]);
            let mut tempfile = tempfile::NamedTempFile::new()?;
            for (_, input) in inputs.iter().enumerate() {
                let file = input.get_input_file()?;
                let file = PathBuf::from_str(&file).unwrap().canonicalize().unwrap().to_string_lossy().to_string();
                let file = "file '".to_owned() + &file + "'\n";
                tempfile.write_all(file.as_bytes()).unwrap();
            }
            tempfile.flush().unwrap();
            let tempfile_path = tempfile.path();
            let tempfile_path = tempfile_path.canonicalize().unwrap();
            input_args.push(format!(
                r#"{}"#,
                tempfile_path.to_string_lossy().to_string()
            ));
            self.output_option.temp_input_filelist = Some(tempfile);
        }

        if let Some(size) = self.output_option.size {
            input_args.append(&mut owned![
                "-filter:v",
                &format!("scale={}:{}", size.0, size.1)
            ]);
        }

        for filter in &self.output_option.video_filters {
            input_args.append(&mut owned!["-filter:v", filter]);
        }

        for filter in &self.output_option.video_filters {
            input_args.append(&mut owned!["-filter:a", filter]);
        }

        if let Some(bitrate) = self.output_option.bitrate {
            input_args.append(&mut owned!["-b:v", &bitrate.to_string()]);
        }

        if let Some(framerate) = self.output_option.framerate {
            input_args.append(&mut owned!["-r", &framerate.to_string()]);
        }

        let mut format_args = owned!["-movflags", "frag_keyframe+empty_moov"];
        output_args.append(&mut format_args);

        if let Some(ref format) = self.output_option.format {
            output_args.append(&mut owned!["-f", format]);
        }

        let mut ending_args = owned![];
        let mut output_target = if let Some(output_file) = output_file {
            owned![output_file]
        } else {
            ending_args.append(&mut owned!("pipe:1"));
            owned!()
        };

        output_args.append(&mut self.output_option.custom_args.clone());

        input_args.append(&mut output_target);

        input_args.append(&mut output_args);

        input_args.append(&mut ending_args);

        println!("{}", input_args.join(" "));
        Ok(input_args)
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
    pub fn stream(&mut self) -> Result<Reader, Error> {
        use tokio::sync::mpsc;

        let buffer_max = self.output_option.stream_buffer_size;
        let (w, r) = mpsc::channel::<ChannelData>(64);
        let ffmpeg_bin = FFMpeg::get_ffmpeg_bin();
        let args = self.build_args(Option::<String>::None)?;
        tokio::spawn(async move {
            let mut child = process::Command::new(ffmpeg_bin)
                .args(args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();
            let mut stdout = child.stdout.take().unwrap();
            let mut stderr = child.stderr.take().unwrap();
            let mut buf = vec![0; buffer_max];
            let mut err_str_buf = String::with_capacity(256);
            let mut err_str = String::new();

            loop {
                let a = stdout.read(&mut buf);
                let b = stderr.read_to_string(&mut err_str_buf);
                let mut out_size = 0;
                let mut err_size = 0;
                tokio::select!(
                    out = a => {
                        match out {
                            Ok(out_s) => {
                                out_size = out_s;
                            },
                            Err(e) => {
                                child.kill().await.unwrap();
                                w.send(ChannelData::Err(e.to_string())).await.unwrap();
                                break;
                            }
                        }
                    },
                    err = b => {
                        match err {
                            Ok(err_s) => {
                                err_size = err_s;
                            },
                            Err(e) => {
                                child.kill().await.unwrap();
                                w.send(ChannelData::Err(e.to_string())).await.unwrap();
                                break;
                            }
                        }
                    }
                );
                // println!("pppp {out_size} {err_size}");
                if out_size == 0 && err_size == 0 {
                    if !err_str.is_empty() {
                        w.send(ChannelData::Err(err_str)).await.unwrap();
                    } else {
                        w.send(ChannelData::End).await.unwrap();
                    }
                    break;
                }
                if out_size != 0 {
                    let bytes = buf[0..out_size].to_vec();
                    w.send(ChannelData::Data(bytes)).await.unwrap();
                }
                if err_size != 0 {
                    err_str.push_str(&err_str_buf[0..err_size]);
                }
            }
        });
        let r = Reader {
            r: Box::pin(r),
            cached: Box::pin(vec![]),
            read: 0,
        };
        Ok(r)
    }
}

#[derive(Debug)]
enum ChannelData {
    Data(Vec<u8>),
    Err(String),
    End,
}

pub struct Reader {
    r: Pin<Box<Receiver<ChannelData>>>,
    cached: Pin<Box<Vec<u8>>>,
    read: usize,
}

impl AsyncRead for Reader {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let cached_len = self.cached.len();
        if cached_len > 0 {
            let remain = buf.remaining();
            let min = cmp::min(cached_len, remain);
            let to_fill: Vec<u8> = self.cached.drain(0..min).collect();
            self.read += to_fill.len();
            buf.put_slice(&to_fill);
            return Poll::Ready(Ok(()));
        }
        let r = self.r.poll_recv(cx);
        let cached = &mut self.cached;
        match r {
            Poll::Ready(val) => match val {
                Some(val) => match val {
                    ChannelData::Data(mut data) => {
                        cached.append(&mut data);
                        let remain = buf.remaining();
                        let len = cached.len();
                        if len == 0 {
                            return Poll::Ready(Ok(()));
                        }
                        let min = cmp::min(len, remain);
                        let to_fill: Vec<u8> = cached.drain(0..min).collect();
                        self.read += to_fill.len();
                        buf.put_slice(&to_fill);
                        Poll::Ready(Ok(()))
                    }
                    ChannelData::Err(err) => Poll::Ready(Err(std::io::Error::new(
                        std::io::ErrorKind::Interrupted,
                        err,
                    ))),
                    ChannelData::End => {
                        let remain = buf.remaining();
                        let len = cached.len();
                        if len == 0 {
                            return Poll::Ready(Ok(()));
                        }
                        let min = cmp::min(len, remain);
                        let to_fill: Vec<u8> = cached.drain(0..min).collect();
                        self.read += to_fill.len();
                        buf.put_slice(&to_fill);
                        Poll::Ready(Ok(()))
                    }
                },
                None => Poll::Pending,
            },
            Poll::Pending => Poll::Pending,
        }
    }
}
