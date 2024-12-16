
#[derive(Debug, Clone)]
pub enum Message {
    ToBrowser,
    ToImage,
    ToVideo,
    ToAudio,
    ModelUpdate,
    ModelClear,
    Open(String),
    EndOfStream,
    NewFrame,
    NextFile,
    PreviousFile,
    RedrawRequested(f64),
    Seek(f64),
    SeekRelease,
    ToggleLoop,
    ToggleMute,
    TogglePause,
}

#[derive(Debug, Clone)]
pub enum Command {
    ModelUpdate(),
    ModelClear(),
    Open(String),
}

//single static page, is not supposed to do anything other than display a button that lets you move to the next page
#[derive(Debug, Default)]
pub struct AudioView {
    //pub audio_model: segmented_button::Model<segmented_button::SingleSelect>,
    pub audio: Option<super::audio::Audio>,
    pub audio_loaded: bool,
    pub audio_path_loaded: String,
    pub position: f64,
    pub duration: f64,
    pub audio_path: String,
}

impl AudioView {
    pub fn new() -> Self {
        AudioView {
            ..Default::default()
        }
    }
    //I don't know when this is called..'
    pub fn update(&mut self, message: Message){
        match message {
            Message::ToBrowser => {},
            Message::ToImage => {},
            Message::ToVideo => {},
            Message::ToAudio => {},
            Message::ModelUpdate => {},
            Message::ModelClear => {},
            Message::Open(path) => {
                self.audio_path = path.clone();
                match super::audio::Audio::new(&self.audio_path) {
                    Ok(video) => {
                        self.audio = Some(video);
                        self.audio_path_loaded = self.audio_path.clone();
                        self.audio_loaded = true;
                    },
                    Err(error) => log::error!("Error: Could not load audio {}: {}", self.audio_path, error),
                }

            }
            Message::EndOfStream => {},
            Message::NewFrame => {},
            Message::NextFile => {},
            Message::PreviousFile => {},
            Message::RedrawRequested(timeinterval) => {},
            Message::Seek(timeinterval) => {},
            Message::SeekRelease => {},
            Message::ToggleLoop => {},
            Message::ToggleMute => {},
            Message::TogglePause => {},
        }
    }
}