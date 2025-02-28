use std::fmt;

#[derive(Clone, Debug, thiserror::Error)]
pub enum FrontendError<E> {
    TextNotSupported,
    GraphicsNotSupported,
    #[from(E)]
    Specific(E),
}

impl<E> fmt::Display for FrontendError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FrontendError::TextNotSupported => write!(f, "This frontend doesn't support text"),
            FrontendError::GraphicsNotSupported => {
                write!(f, "This frontend doesn't support graphics")
            }
            FrontendError::Specific(err) => write!(f, "{}", err),
        }
    }
}
