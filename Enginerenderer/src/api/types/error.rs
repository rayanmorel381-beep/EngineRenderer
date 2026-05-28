use std::fmt;
use std::io;

#[derive(Debug)]
pub enum RenderError {
    InvalidDimensions { width: usize, height: usize },
    InvalidSampleCount(usize),
    OutputPathInvalid(std::path::PathBuf),
    Io(io::Error),
    SceneEmpty,
    CameraDegenerate,
}

pub type ApiResult<T> = Result<T, RenderError>;

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimensions { width, height } => {
                write!(f, "invalid render dimensions: {width}x{height}")
            }
            Self::InvalidSampleCount(n) => write!(f, "invalid sample count: {n}"),
            Self::OutputPathInvalid(path) => {
                write!(f, "output path is invalid: {}", path.display())
            }
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::SceneEmpty => write!(f, "scene contains no renderable objects"),
            Self::CameraDegenerate => {
                write!(
                    f,
                    "camera eye and target are coincident or produce a degenerate view"
                )
            }
        }
    }
}

impl std::error::Error for RenderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for RenderError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}
