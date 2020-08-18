use std::{
    io::{
        self,
        Write,
    },
};

use termion::{
    raw::IntoRawMode,
};


use crate::{
    Error,
    Renderer,
    Result,
};



pub struct TerminalRenderer{
    stdout: termion::raw::RawTerminal<io::Stdout>,
    pub height: u16,
    pub width: u16,

    // cursor
    cx: u16,
    cy: u16,

    // origin offset
    ox: u16,
    oy: u16,
}


impl std::fmt::Debug for TerminalRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TerminalRenderer")
            .field("height", &self.height)
            .field("width", &self.width)
            .field("cx", &self.cx)
            .field("cy", &self.cy)
            .finish()
    }
}

impl TerminalRenderer {
    pub fn new(stdout: io::Stdout, h: Option<usize>, w : Option<usize>) ->
        Result<TerminalRenderer> {


        let (tw, th) = termion::terminal_size()
            .map(|(x, y)| (x as usize, y as usize))?;

        let height = h.unwrap_or(th);

        if height > th {
            return Err(Error::InvalidRendererSize(height))
        }

        let width = w.unwrap_or(tw);

        if width > tw {
            return Err(Error::InvalidRendererSize(width))
        }

        let mut stdout = stdout.into_raw_mode()?;
        write!(stdout, "{}", termion::clear::All).unwrap();

        let mut ret = TerminalRenderer{
            stdout: stdout,
            height: height as u16,
            width: width as u16,
            cx: 1,
            cy: 1,
            ox: 1,
            oy: 1,
        };

        ret.goto()?;

        Ok(ret)
    }


    pub fn debug(&mut self, s: &str) {
        let save = self.save_curs();
        self.load_curs((1, 1));
        self.write(s).unwrap();
        self.load_curs(save);
    }

    pub fn flush(&mut self) -> Result<()> {
        self.stdout.flush()?;

        Ok(())
    }

    pub fn save_curs(&self) -> (u16, u16) {
        (self.cx, self.cy)
    }

    pub fn load_curs(&mut self, cur: (u16, u16)) {
        let (cx, cy) = cur;
        self.cx = cx;
        self.cy = cy;
        self.goto().unwrap();
    }

    pub fn clear(&mut self) -> Result<()> {
        write!(self.stdout, "{}", termion::clear::All)?;
        Ok(())
    }

    pub fn goto(&mut self) -> Result<()> {
        write!(self.stdout, "{}",
               termion::cursor::Goto(
                   self.cx,
                   self.cy,
                   )
               )?;
        Ok(())
    }

    pub fn set_xy(&mut self, x: usize, y: usize) -> Result<()> {

        self.cx = x.checked_add(1).and_then(|xx| {
                if xx < u16::MAX.into() {
                    Some(xx as u16)
                } else {
                    None
                }
            })
            .ok_or(Error::InvalidCursorSize(x))?;

        self.cy = y.checked_add(1).and_then(|yy| {
                if yy < u16::MAX.into() {
                    Some(yy as u16)
                } else {
                    None
                }
            })
            .ok_or(Error::InvalidCursorSize(y))?;


        Ok(())
    }
}

impl Renderer for TerminalRenderer {
    fn height(&self) -> usize {
        self.height as usize
    }

    fn width(&self) -> usize {
        self.width as usize
    }

    fn write(&mut self, s: &str) -> Result<()> {
        assert!(s.len() < u16::MAX as usize);

        write!(self.stdout, "{}", termion::clear::CurrentLine)?;

        // TODO(ear7h): trim the string if too long
        write!(self.stdout, "{}", s)?;
        // TODO(ear7h): use unicode length
        self.cx += s.len() as u16;

        Ok(())
    }

    // move to leftmost column
    fn ret(&mut self) -> Result<()> {
        self.cx = self.ox;
        self.goto()
    }

    // move to upermost row
    fn vret(&mut self) -> Result<()> {
        self.cy = self.oy;
        self.goto()
    }

    fn move_x(&mut self, n: isize) -> Result<()> {
        let tmp = self.cx as isize + n;
        assert!(tmp > 0 && tmp < u16::MAX as isize);
        self.cx = tmp as u16;
        self.goto()
    }

    fn move_y(&mut self, n: isize) -> Result<()> {
        let tmp = self.cy as isize + n;
        assert!(tmp > 0 && tmp < u16::MAX as isize);

        self.cy = tmp as u16;
        self.goto()
    }

}

