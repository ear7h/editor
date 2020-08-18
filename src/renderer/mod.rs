use crate::{
    Result,
};

mod terminal_renderer;
pub use terminal_renderer::TerminalRenderer;

pub trait Renderer {
    fn height(&self) -> usize;
    fn width(&self) -> usize;

    // write the string
    fn write(&mut self, s: &str) -> Result<()>;

    // move the cursor to the first column
    fn ret(&mut self) -> Result<()>;

    // move the cursor to the first row
    fn vret(&mut self) -> Result<()>;

    // move left or right n cols, negative is left
    fn move_x(&mut self, n: isize) -> Result<()>;

    // move up or down n rows, negative is up
    fn move_y(&mut self, n: isize) -> Result<()>;
}
