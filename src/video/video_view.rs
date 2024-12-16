//use cosmic::iced_wgpu::Renderer;

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
    ToggleSelectAudioStream,
    ToggleSelectSubtitleStream,
}

#[derive(Debug, Clone)]
pub enum Command {
    ModelUpdate(),
    ModelClear(),
    Open(String),
}

//single static page, is not supposed to do anything other than display a button that lets you move to the next page
#[derive(Debug, Default)]
pub struct VideoView {
    //pub video_model: segmented_button::Model<segmented_button::SingleSelect>,
    pub video_path: String,
    file_list: Vec<String>,
    file_id: usize,
    sort_order: String,
    pub video: Option<super::video::Video>,
    pub video_loaded: bool,
    pub video_path_loaded: String,
    pub position: f64,
    pub duration: f64,
    dragging: bool,
    audio_stream: u32,
    subtitle_stream: u32,
    subtitle_active: bool,
}

impl VideoView {
    pub fn new() -> Self {
        VideoView {
            ..Default::default()
        }
    }
    //I don't know when this is called..'
    pub fn update(&mut self, message: Message) {
        match message {
            Message::ToBrowser => {}
            Message::ToImage => {}
            Message::ToVideo => {}
            Message::ToAudio => {}
            Message::ModelUpdate => {}
            Message::ModelClear => {}
            Message::Open(path) => {
                self.video_path = path.clone();
                match super::video::Video::new(&self.video_path) {
                    Ok(video) => {
                        self.video = Some(video);
                        self.video_path_loaded = self.video_path.clone();
                        self.video_loaded = true;
                    },
                    Err(error) => log::error!("Error: Could not load video {}: {}", self.video_path, error),
                }
            }
            Message::EndOfStream => {}
            Message::NewFrame => {}
            Message::NextFile => {}
            Message::PreviousFile => {}
            Message::RedrawRequested(instant) => {}
            Message::Seek(timeinterval) => {}
            Message::SeekRelease => {}
            Message::ToggleLoop => {}
            Message::ToggleMute => {}
            Message::TogglePause => {}
            Message::ToggleSelectAudioStream => {}
            Message::ToggleSelectSubtitleStream => {}
        }
    }
}
