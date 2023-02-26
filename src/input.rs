use std::{path::Path, time};

use crate::{error::Error, output::FFmpegOutput, owned, utils::format_time};

#[derive(Clone, PartialEq, Debug)]
pub enum StreamType {
    Audio,
    Video,
    Both,
}

#[derive(Clone)]
pub struct FFMpegInput {
    pub(crate) input_file: Option<String>,
    pub(crate) stream_type: StreamType,
    pub(crate) vcodec: Option<String>,
    pub(crate) acodec: Option<String>,
    pub(crate) custom_args: Vec<String>,
    pub(crate) start_time: Option<String>,
    pub(crate) end_time: Option<String>,
    pub(crate) stream_index: Option<u64>,
    pub(crate) format: Option<String>,
}

impl FFMpegInput {
    pub fn new() -> Self {
        return Self {
            input_file: None,
            stream_type: StreamType::Both,
            vcodec: None,
            acodec: None,
            custom_args: owned![],
            start_time: None,
            end_time: None,
            stream_index: None,
            format: None,
        };
    }

    pub fn format(mut self, format: &str) -> Self {
        self.format = Some(format.to_owned());
        self
    }

    pub fn arg(mut self, arg: &str) -> Self {
        self.custom_args.push(arg.to_owned());
        self
    }

    pub fn args(mut self, args: Vec<impl AsRef<str>>) -> Self {
        for arg in args {
            let arg = arg.as_ref();
            self.custom_args.push(arg.to_owned());
        }
        self
    }

    pub fn input(file: impl AsRef<Path>) -> Self {
        let ffmpeg = Self::new();
        ffmpeg.input_file(file)
    }

    pub fn input_file(mut self, file: impl AsRef<Path>) -> Self {
        let file = Some(file.as_ref().to_string_lossy().into_owned());
        self.input_file = file;
        self
    }

    pub fn only_audio(mut self) -> Self {
        self.stream_type = StreamType::Audio;
        self
    }

    pub fn only_video(mut self) -> Self {
        self.stream_type = StreamType::Video;
        self
    }

    pub fn take_stream(mut self, stream_index: u64) -> Self {
        assert_ne!(self.stream_type, StreamType::Both, "must specify video/audio stream by using .only_audio() or .only_video() before take stream");
        self.stream_index = Some(stream_index);
        self
    }

    /// set stream type for input file:
    /// StreamType::Audio: take only audio stream from input file
    /// StreamType::Video: take only video stream from input file
    /// StreamType::Both: take video and audio stream from input file
    pub fn stream_type(mut self, stream_type: StreamType) -> Self {
        self.stream_type = stream_type;
        self
    }

    /// set video encoder for input file
    /// run: ffmpeg -encoders to list all supported encoders
    pub fn video_codec(mut self, codec: String) -> Self {
        self.vcodec = Some(codec);
        self
    }

    /// set audio encoder for input file
    /// run: ffmpeg -encoders to list all supported encoders
    pub fn audio_codec(mut self, codec: String) -> Self {
        self.acodec = Some(codec);
        self
    }

    pub(crate) fn get_input_file(&self) -> Result<String, Error> {
        let file = self
            .input_file
            .as_ref()
            .ok_or(Error {
                msg: "input file is not set".to_owned(),
            })?
            .clone();
        Ok(file)
    }

    /// set start time of input file
    pub fn start_time(mut self, start_time: &time::Duration) -> Self {
        let t = format_time(start_time);
        self.start_time = Some(t);
        self
    }

    /// set end time of input file
    pub fn end_time(mut self, end_time: &time::Duration) -> Self {
        let t = format_time(end_time);
        self.end_time = Some(t);
        self
    }

    pub fn build_args(&self) -> Result<Vec<String>, Error> {
        let abs_file = self.get_input_file()?;

        let mut input_args = owned![];

        if let Some(ref start_time) = self.start_time {
            input_args.append(&mut owned!["-ss", start_time]);
        }

        if let Some(ref end_time) = self.end_time {
            input_args.append(&mut owned!["-to", end_time]);
        }

        if let Some(ref format) = self.format {
            input_args.append(&mut owned!["-f", format]);
        }

        for arg in &self.custom_args {
            input_args.push(arg.to_owned());
        }

        let mut args = owned!["-i", &abs_file];

        input_args.append(&mut args);

        Ok(input_args)
    }

    pub fn output(&self) -> FFmpegOutput {
        let input = FFMpegMultipleInput::new(self);
        return FFmpegOutput::new(input);
    }

    pub fn merge(&self, anthor_input: &FFMpegInput) -> FFMpegMultipleInput {
        FFMpegMultipleInput::merge(self, anthor_input)
    }
}

pub enum MergeStrategy {
    Merge,
    Concat,
}

pub struct FFMpegMultipleInput {
    pub(crate) inputs: Vec<FFMpegInput>,
    pub(crate) merge_strategy: MergeStrategy,
}

impl FFMpegMultipleInput {
    pub fn new(input: &FFMpegInput) -> Self {
        Self {
            inputs: vec![input.clone()],
            merge_strategy: MergeStrategy::Merge,
        }
    }

    /// concat inputs by order to one output
    ///
    /// samples:
    /// ```
    /// use ffmpeg_cli_utils::FFMpegMultipleInput;
    /// # use ffmpeg_cli_utils::FFMpeg;
    /// # FFMpeg::set_ffmpeg_bin("./ffmpeg");
    /// let videos = ["./sample.mp4", "./sample1.mp4"];
    /// FFMpegMultipleInput::concat(&videos)
    ///     .output()
    ///     .save("./output/concat_output.mp4");
    /// ```
    pub fn concat(inputs: &[&str]) -> Self {
        let inputs: Vec<FFMpegInput> = inputs.into_iter().map(|i| FFMpegInput::input(*i)).collect();
        Self {
            inputs,
            merge_strategy: MergeStrategy::Concat,
        }
    }

    pub fn merge(one: &FFMpegInput, two: &FFMpegInput) -> Self {
        Self {
            inputs: vec![one.clone(), two.clone()],
            merge_strategy: MergeStrategy::Merge,
        }
    }
    pub fn append(&mut self, inputs: Vec<&FFMpegInput>) {
        let mut inputs: Vec<FFMpegInput> = inputs.into_iter().map(|input| input.clone()).collect();
        self.inputs.append(&mut inputs);
    }

    pub fn output(self) -> FFmpegOutput {
        return FFmpegOutput::new(self);
    }
}
