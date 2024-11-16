use cosmic::iced::{Element, Length};
use cosmic::iced::widget::{Container, column, Space, Button};
use cosmic::widget::{self, segmented_button, };


#[derive(Debug, Clone)]
pub enum Message {
    ToBrowser,
    ToImage,
    ToVideo,
    ToAudio,
    Open(String),
}

//single static page, is not supposed to do anything other than display a button that lets you move to the next page
pub struct Image {
    //pub image_model: segmented_button::Model<segmented_button::SingleSelect>,
    image_viewer: cosmic::iced::widget::image::Viewer<cosmic::widget::image::Handle>,
    image_path: String,
}

impl Image {
    pub fn new() -> Self {
        Image {
            //image_model: segmented_button::ModelBuilder::default().build(),
            image_viewer: cosmic::iced::widget::image::Viewer::new(cosmic::widget::image::Handle::from_path("./examples/logo.png")),
            image_path: "./examples/logo.png".to_string(),
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
            }
        }
    }

    pub fn view<'a>(&self) -> Element<'a, Message> {
        cosmic::iced::widget::image::Viewer::new(cosmic::widget::image::Handle::from_path(self.image_path.clone())).into()
    }
}