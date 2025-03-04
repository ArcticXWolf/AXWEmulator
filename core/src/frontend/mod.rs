use std::error::Error;

use error::FrontendError;
use graphics::FrameReceiver;
use input::InputSender;
use text::TextReceiver;

pub mod error;
pub mod graphics;
pub mod input;
pub mod text;

pub trait Frontend {
    type Error: Error;

    fn register_text_reciever(
        &mut self,
        _reciever: TextReceiver,
    ) -> Result<(), FrontendError<Self::Error>> {
        Err(FrontendError::TextNotSupported)
    }

    fn register_graphics_reciever(
        &mut self,
        _reciever: FrameReceiver,
    ) -> Result<(), FrontendError<Self::Error>> {
        Err(FrontendError::GraphicsNotSupported)
    }

    fn register_input_sender(
        &mut self,
        _sender: InputSender,
    ) -> Result<(), FrontendError<Self::Error>> {
        Err(FrontendError::InputNotSupported)
    }
}
