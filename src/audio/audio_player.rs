use crate::video::pipeline::VideoPrimitive;

use gstreamer as gst;
use cosmic::iced::{
    advanced::{self, graphics::core::event::Status, layout, widget, Widget},
    Element, Event, event
};
use cosmic::iced_wgpu::primitive::pipeline::Renderer as PrimitiveRenderer;
use log::error;
use std::{marker::PhantomData, sync::atomic::Ordering, time::Duration};
use std::{sync::Arc, time::Instant};

/// Video player widget which displays the current frame of a [`Video`](crate::Video).
pub struct AudioPlayer<'a, Message, Theme = cosmic::iced::Theme, Renderer = cosmic::iced::Renderer>
where
    Renderer: PrimitiveRenderer,
{
    audio: &'a super::audio::Audio,
    content_fit: cosmic::iced::ContentFit,
    width: cosmic::iced::Length,
    height: cosmic::iced::Length,
    on_end_of_stream: Option<Message>,
    on_new_frame: Option<Message>,
    on_subtitle_text: Option<Box<dyn Fn(Option<String>) -> Message + 'a>>,
    on_error: Option<Box<dyn Fn(&glib::Error) -> Message + 'a>>,
    _phantom: PhantomData<(Theme, Renderer)>,
}

impl<'a, Message, Theme, Renderer> AudioPlayer<'a, Message, Theme, Renderer>
where
    Renderer: PrimitiveRenderer,
{
    /// Creates a new video player widget for a given video.
    pub fn new(_audio: &'a super::audio::Audio) -> Self {
        AudioPlayer {
            audio: _audio,
            content_fit: cosmic::iced::ContentFit::Contain,
            width: cosmic::iced::Length::Shrink,
            height: cosmic::iced::Length::Shrink,
            on_end_of_stream: None,
            on_new_frame: None,
            on_subtitle_text: None,
            on_error: None,
            _phantom: Default::default(),
        }
    }

    /// Sets the width of the `AudioPlayer` boundaries.
    pub fn width(self, width: impl Into<cosmic::iced::Length>) -> Self {
        AudioPlayer {
            width: width.into(),
            ..self
        }
    }

    /// Sets the height of the `AudioPlayer` boundaries.
    pub fn height(self, height: impl Into<cosmic::iced::Length>) -> Self {
        AudioPlayer {
            height: height.into(),
            ..self
        }
    }

    /// Sets the `ContentFit` of the `AudioPlayer`.
    pub fn content_fit(self, content_fit: cosmic::iced::ContentFit) -> Self {
        AudioPlayer {
            content_fit,
            ..self
        }
    }

    /// Message to send when the video reaches the end of stream (i.e., the video ends).
    pub fn on_end_of_stream(self, on_end_of_stream: Message) -> Self {
        AudioPlayer {
            on_end_of_stream: Some(on_end_of_stream),
            ..self
        }
    }

    /// Message to send when the video receives a new frame.
    pub fn on_new_frame(self, on_new_frame: Message) -> Self {
        AudioPlayer {
            on_new_frame: Some(on_new_frame),
            ..self
        }
    }

    /// Message to send when the video receives a new frame.
    pub fn on_subtitle_text<F>(self, on_subtitle_text: F) -> Self
    where
        F: 'a + Fn(Option<String>) -> Message,
    {
        AudioPlayer {
            on_subtitle_text: Some(Box::new(on_subtitle_text)),
            ..self
        }
    }

    /// Message to send when the video playback encounters an error.
    pub fn on_error<F>(self, on_error: F) -> Self
    where
        F: 'a + Fn(&glib::Error) -> Message,
    {
        AudioPlayer {
            on_error: Some(Box::new(on_error)),
            ..self
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for AudioPlayer<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: PrimitiveRenderer,
{
    fn size(&self) -> cosmic::iced::Size<cosmic::iced::Length> {
        cosmic::iced::Size {
            width: cosmic::iced::Length::Shrink,
            height: cosmic::iced::Length::Shrink,
        }
    }

    fn layout(
        &self,
        _tree: &mut widget::Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let (video_width, video_height) = self.audio.size();

        // based on `Image::layout`
        let image_size = cosmic::iced::Size::new(video_width as f32, video_height as f32);
        let raw_size = limits.resolve(self.width, self.height, image_size);
        let full_size = self.content_fit.fit(image_size, raw_size);
        let final_size = cosmic::iced::Size {
            width: match self.width {
                cosmic::iced::Length::Shrink => f32::min(raw_size.width, full_size.width),
                _ => raw_size.width,
            },
            height: match self.height {
                cosmic::iced::Length::Shrink => f32::min(raw_size.height, full_size.height),
                _ => raw_size.height,
            },
        };

        layout::Node::new(final_size)
    }

    fn draw(
        &self,
        _tree: &widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &advanced::renderer::Style,
        layout: advanced::Layout<'_>,
        _cursor: advanced::mouse::Cursor,
        _viewport: &cosmic::iced::Rectangle,
    ) {
        let mut inner = self.audio.write();

        // bounds based on `Image::draw`
        let image_size = cosmic::iced::Size::new(inner.width as f32, inner.height as f32);
        let bounds = layout.bounds();
        let adjusted_fit = self.content_fit.fit(image_size, bounds.size());
        let scale = cosmic::iced::Vector::new(
            adjusted_fit.width / image_size.width,
            adjusted_fit.height / image_size.height,
        );
        let final_size = cosmic::iced::Size::new(
            image_size.width as f32 * scale.x, 
            image_size.height as f32 * scale.y);

        let position = match self.content_fit {
            cosmic::iced::ContentFit::None => cosmic::iced::Point::new(
                bounds.x + (image_size.width - adjusted_fit.width) / 2.0,
                bounds.y + (image_size.height - adjusted_fit.height) / 2.0,
            ),
            _ => cosmic::iced::Point::new(
                bounds.center_x() - final_size.width / 2.0,
                bounds.center_y() - final_size.height / 2.0,
            ),
        };

        let drawing_bounds = cosmic::iced::Rectangle::new(position, final_size);

        let upload_frame = inner.upload_frame.swap(false, Ordering::SeqCst);

        if upload_frame {
            let last_frame_time = inner
                .last_frame_time
                .lock()
                .map(|time| *time)
                .unwrap_or_else(|_| Instant::now());
            inner.set_av_offset(Instant::now() - last_frame_time);
        }

        renderer.draw_pipeline_primitive(
            drawing_bounds,
            VideoPrimitive::new(
                inner.id,
                Arc::clone(&inner.alive),
                Arc::clone(&inner.frame),
                (inner.width as _, inner.height as _),
                upload_frame,
            ),
        );
    }

}

impl<'a, Message, Theme, Renderer> From<AudioPlayer<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Theme: 'a,
    Renderer: 'a + PrimitiveRenderer,
{
    fn from(audio_player: AudioPlayer<'a, Message, Theme, Renderer>) -> Self {
        Self::new(audio_player)
    }
}
