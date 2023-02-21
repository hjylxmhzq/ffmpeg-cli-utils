use std::{env, path::Path};


use crate::{async_output::FFmpegOutput, error::Error, owned};


#[derive(Clone)]
pub struct FFMpegInput {
    bin_path: Option<String>,
    input_file: Option<String>,
}

impl FFMpegInput {
    pub fn new() -> Self {
        let bin_path = env::var("FFMPEG_BINARY").map_or("ffmpeg".to_owned(), |p| p);
        return Self {
            bin_path: Some(bin_path),
            input_file: None,
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
    pub(crate) fn get_ffmpeg_bin(&self) -> Result<String, Error> {
        let file = self
            .bin_path
            .as_ref()
            .ok_or(Error {
                msg: "ffmpeg binary is not set".to_owned(),
            })?
            .clone();
        Ok(file)
    }
    pub fn build_args(&self) -> Result<Vec<String>, Error> {

        let abs_file = self.get_input_file()?;

        let args = owned!["-y", "-i", &abs_file];

        Ok(args)
    }

    pub fn output_async(&self) -> FFmpegOutput {
        return FFmpegOutput::new(self.clone());
    }
    
    pub fn output(&self) -> FFmpegOutput {
        return FFmpegOutput::new(self.clone());
    }

}
