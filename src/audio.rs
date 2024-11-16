use cosmic::iced::{Element, Length};
use cosmic::iced::widget::{Container, column, Space, Button};
use cosmic::widget::{self, segmented_button};

#[derive(Debug, Clone)]
pub enum Message {
    ToBrowser,
    ToImage,
    ToVideo,
    ToAudio,
    Open(String),
}

//single static page, is not supposed to do anything other than display a button that lets you move to the next page
pub struct Audio {
    //pub audio_model: segmented_button::Model<segmented_button::SingleSelect>,
    audio_path: String,
}

impl Audio {
    pub fn new() -> Self {
        Audio {
            //audio_model: segmented_button::ModelBuilder::default().build(),
            audio_path: String::new(),
        }
    }
    //I don't know when this is called..'
    pub fn update(&mut self, message: Message){
        match message {
            Message::ToBrowser => {},
            Message::ToImage => {},
            Message::ToVideo => {},
            Message::ToAudio => {},
            Message::Open(path) => {
                self.audio_path = path.clone();
            }
        }
    }

    pub fn view<'a>(&self) -> Element<'a, Message> {
        Container::new(
            column![
            Space::new(Length::Shrink, Length::Fill),
            Button::new("To Browser").on_press(Message::ToBrowser),
            Button::new("To Image").on_press(Message::ToImage),
            Button::new("To Video").on_press(Message::ToVideo),
            Space::new(Length::Shrink, Length::Fill),
            ]
        )
        .padding(20.)
        .center_x()
        .width(Length::Fill)
        .into()
    }
}