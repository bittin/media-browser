use cosmic::iced::Length;

#[derive(Debug, Clone)]
pub enum Message {
    ToBrowser,
    ToImage,
    ToVideo,
    ToAudio,
    Open(String),
    NextFile,
    PreviousFile,
    ZoomPlus,
    ZoomMinus,
    ZoomFit,
    Seek,
}

//single static page, is not supposed to do anything other than display a button that lets you move to the next page
pub struct ImageView {
    //pub image_model: segmented_button::Model<segmented_button::SingleSelect>,
    image_viewer: cosmic::iced::widget::image::Viewer<cosmic::widget::image::Handle>,
    pub image_path: String,
    pub image_path_loaded: String,
    width: Length,
    height: Length, 
    min_scale: f32,
    max_scale: f32,
    scale_step: f32,
}

impl ImageView {
    pub fn new() -> Self {
        ImageView {
            //image_model: segmented_button::ModelBuilder::default().build(),
            image_viewer: cosmic::iced::widget::image::Viewer::new(cosmic::widget::image::Handle::from_path("./examples/logo.png")),
            image_path: "./examples/logo.png".to_string(),
            image_path_loaded: String::new(),
            width: Length::Fixed(4096.0),
            height: Length::Fixed(4096.0), 
            min_scale: 0.1,
            max_scale: 10.0,
            scale_step: 0.2,
        }
    }
    //I don't know when this is called..'
    pub fn update(&mut self, message: Message){
        match message {
            Message::ToBrowser => {},
            Message::ToImage => {},
            Message::ToVideo => {},
            Message::ToAudio => {},
            Message::Open(imagepath) => {
                self.image_path = imagepath.clone();
                self.image_viewer =  cosmic::iced::widget::image::Viewer::new(
                    cosmic::widget::image::Handle::from_path(self.image_path.clone()))
                    .width(self.width)
                    .height(self.height)
                    .min_scale(self.min_scale)
                    .max_scale(self.max_scale)
                    .scale_step(self.scale_step)
                    .padding(5.0);
                self.image_path_loaded = self.image_path.clone();
            }
            Message::NextFile => {},
            Message::PreviousFile => {},
            Message::ZoomPlus => {},
            Message::ZoomMinus => {},
            Message::ZoomFit => {},
            Message::Seek => {},
        }
    }

}