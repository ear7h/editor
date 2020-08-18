#![feature(linked_list_cursors)]

mod error;
pub use error::{
    Error,
    Result,
};

mod renderer;
pub use renderer::{
    Renderer,
    TerminalRenderer,
};

mod buffer;
pub use buffer::{
    View,
    Buffer,
    Line,
    LineMut,
    LineConfig,
};
