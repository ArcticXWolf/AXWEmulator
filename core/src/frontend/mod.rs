use std::error::Error;

use audio::AudioReceiver;
use error::FrontendError;
use graphics::FrameReceiver;
use input::InputSender;
use text::TextReceiver;

pub mod audio;
pub mod error;
pub mod graphics;
pub mod input;
pub mod text;

pub trait Frontend {
    type Error: Error;

    fn register_text_receiver(
        &mut self,
        _receiver: TextReceiver,
    ) -> Result<(), FrontendError<Self::Error>> {
        Err(FrontendError::TextNotSupported)
    }

    fn register_graphics_receiver(
        &mut self,
        _receiver: FrameReceiver,
    ) -> Result<(), FrontendError<Self::Error>> {
        Err(FrontendError::GraphicsNotSupported)
    }

    fn register_audio_receiver(
        &mut self,
        _receiver: AudioReceiver,
    ) -> Result<(), FrontendError<Self::Error>> {
        Err(FrontendError::AudioNotSupported)
    }

    fn register_input_sender(
        &mut self,
        _sender: InputSender,
    ) -> Result<(), FrontendError<Self::Error>> {
        Err(FrontendError::InputNotSupported)
    }
}
