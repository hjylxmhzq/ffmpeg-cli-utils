use std::{path::Path, time};

use crate::{error::Error, output::FFmpegOutput, owned, utils::format_time};

#[derive(Clone)]
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
        };
    }

    pub fn args(mut self, args: Vec<impl AsRef<str>>) -> Self {
        for arg in args {
            let arg = arg.as_ref();
            self.custom_args.push(arg.to_owned());
        }
        self
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

        let mut args = owned!["-i", &abs_file];

        input_args.append(&mut args);
        
        Ok(input_args)
    }

    pub fn output(&self) -> FFmpegOutput {
        let input = FFMpegMultipleInput::new(self);
        return FFmpegOutput::new(input);
    }

    pub fn concat(&self, anthor_input: &FFMpegInput) -> FFMpegMultipleInput {
        FFMpegMultipleInput::combine(self, anthor_input)
    }
}

pub struct FFMpegMultipleInput {
    pub(crate) inputs: Vec<FFMpegInput>,
}

impl FFMpegMultipleInput {
    pub fn new(input: &FFMpegInput) -> Self {
        Self {
            inputs: vec![input.clone()],
        }
    }

    pub fn combine(one: &FFMpegInput, two: &FFMpegInput) -> Self {
        Self {
            inputs: vec![one.clone(), two.clone()],
        }
    }
    pub fn append(&mut self, inputs: Vec<&FFMpegInput>) {
        let mut inputs: Vec<FFMpegInput> = inputs.into_iter().map(|input| input.clone()).collect();
        self.inputs.append(&mut inputs);
    }

    pub fn output(self) -> FFmpegOutput {
        return FFmpegOutput::new(self)
    }
}
