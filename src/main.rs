#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(dead_code)]
#![allow(unused_imports)]


#![feature(linked_list_cursors)]

use std::{
    mem,
    io::{
        self,
        Write,
    },
    time,
    thread,
    collections::LinkedList,
    collections::linked_list,
    default::Default,
    convert::From,
};

use termion::{
    raw::IntoRawMode,
    cursor::DetectCursorPos,
    input::TermRead,
    event::Key,
};

use unicode_segmentation::{
    UnicodeSegmentation,
    GraphemeCursor,
};

use unicode_width::{
    UnicodeWidthStr,
};


fn main() {
    let mut buf = Buffer::from("hello\nworld");

    let stdout = io::stdout();
    let stdin = io::stdin();

    let mut view0 = View::new(&mut buf, 10, 10);
    let mut r = TerminalRenderer::new(stdout, None, None).unwrap();

    r.set_xy(0, 0).unwrap();

    let mut edit_mode = false;

    for c in stdin.keys() {
        if edit_mode {
            match c.unwrap() {
                Key::Esc => {
                    edit_mode = false;
                },
                Key::Backspace => {
                    view0.backspace();
                },
                Key::Char(c) => {
                    match c {
                        '\n' | '\r' => {
                            view0.insert_line_below();
                            view0.first_col();
                            view0.scroll_rel(1);
                        },
                        '\t' => {
                            view0.tab();
                        }
                        _ => {
                            let mut buf: [u8;8] = [0;8];
                            let cs = c.encode_utf8(&mut buf);
                            view0.insert_str(cs);
                        }
                    }
                }
                _ => {},
            }
        } else {
            match c.unwrap() {
                Key::Char('q') => break,
                // move and insert
                Key::Char('i') => {
                    edit_mode = true;
                    //r.debug("going to insert mode");
                },
                Key::Char('I') => {
                    view0.first_non_space_col();
                    edit_mode = true;
                },
                Key::Char('o') => {
                    view0.insert_line_below();
                    view0.scroll_rel(1);
                    edit_mode = true;
                },
                Key::Char('O') => {
                    view0.insert_line_above();
                    view0.scroll_rel(-1);
                    edit_mode = true;
                }
                Key::Char('a') => {
                    view0.next_col();
                    edit_mode = true;
                },
                Key::Char('A') => {
                    view0.last_col();
                    edit_mode = true;
                },
                // movement
                Key::Char('0') => {
                    view0.first_col();
                },
                Key::Char('w') => {
                    view0.next_word();
                },
                Key::Char('b') => {
                    view0.prev_word();
                },
                Key::Char('h') => {
                    // left
                    view0.prev_col();
                },
                Key::Char('j') => {
                    // down
                    view0.scroll_rel(1);
                },
                Key::Char('k') => {
                    // down
                    view0.scroll_rel(-1);
                },
                Key::Char('l') => {
                    // right
                    view0.next_col();
                }
                _ => {}
            }
        }

        r.clear().unwrap();
        r.set_xy(0, 0).unwrap();
        view0.render(&mut r).unwrap();
        r.flush().unwrap();
    }

    mem::drop(r);

    println!("{}", "buffer: ");
    buf.collate(&mut io::stdout());
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
enum Error {
    InvalidCursorSize(usize),
    InvalidRendererSize(usize),
    Io(io::Error),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

trait Renderer {
    fn height(&self) -> usize;
    fn width(&self) -> usize;

    // write the string
    fn write(&mut self, s: &str) -> Result<()>;

    // move the cursor to the first column
    fn ret(&mut self) -> Result<()>;

    // move left or right n cols, negative is left
    fn move_x(&mut self, n: isize) -> Result<()>;

    // move up or down n rows, negative is up
    fn move_y(&mut self, n: isize) -> Result<()>;
}

struct TerminalRenderer{
    stdout: termion::raw::RawTerminal<io::Stdout>,
    height: u16,
    width: u16,
    cx: u16,
    cy: u16,
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
    fn new(stdout: io::Stdout, h: Option<usize>, w : Option<usize>) ->
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
        };

        ret.goto()?;

        Ok(ret)
    }


    fn debug(&mut self, s: &str) {
        let save = self.save_curs();
        self.load_curs((1, 1));
        self.write(s);
        self.load_curs(save);
    }

    fn flush(&mut self) -> Result<()> {
        self.stdout.flush()?;

        Ok(())
    }

    fn save_curs(&self) -> (u16, u16) {
        (self.cx, self.cy)
    }

    fn load_curs(&mut self, cur: (u16, u16)) {
        let (cx, cy) = cur;
        self.cx = cx;
        self.cy = cy;
        self.goto();
    }

    fn clear(&mut self) -> Result<()> {
        write!(self.stdout, "{}", termion::clear::All)?;
        Ok(())
    }

    fn goto(&mut self) -> Result<()> {
        write!(self.stdout, "{}",
               termion::cursor::Goto(
                   self.cx,
                   self.cy,
                   )
               )?;
        Ok(())
    }

    fn set_xy(&mut self, x: usize, y: usize) -> Result<()> {

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

    fn ret(&mut self) -> Result<()> {
        self.cx = 1;
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

struct View<'buf> {
    cur : linked_list::CursorMut<'buf, Line>,
    cx : usize, // x position in current line
}


impl<'buf> View <'buf> {
    fn new(buf: &'buf mut Buffer, height : u16, width: u16) -> Self {
        if buf.lines.len() == 0 {
            buf.lines.push_back(Line::from(""));
        }

        View{
            cur: buf.lines.cursor_front_mut(),
            cx: 0,
        }
    }

    fn current_mut(&mut self) -> &mut Line {
        self.cur.current().unwrap()
    }

    fn current(&self) -> &Line {
        self.cur.as_cursor().current().unwrap()
    }

    fn render<R : Renderer> (&self, r: &mut R) -> Result<()> {
        // half height
        let hh = r.height() / 2;

        r.move_y(hh as isize)?;

        // go up
        let mut up_lines = 0;
        let mut cur = self.cur.as_cursor();
        for n in 0..hh {
            if let Some(line) = cur.current() {
                r.write(line.s.as_str())?;
                r.ret()?;
                r.move_y(-1)?;
                up_lines += 1;
                cur.move_prev();
            } else {
                break;
            }
        }

        // reset
        r.move_y(up_lines+1)?;

        // go down
        let mut down_lines = 0;
        let mut cur = self.cur.as_cursor();
        cur.move_next();
        down_lines += 1;
        for n in 0..hh {
            if let Some(line) = cur.current() {
                r.write(line.s.as_str())?;
                r.ret()?;
                r.move_y(1)?;
                down_lines += 1;
                cur.move_next();
            } else {
                break;
            }
        }

        // write set cursor
        r.move_y(-down_lines)?;
        r.ret()?;
        r.move_x(self.cx.min(self.current().cols()) as isize)?;
        Ok(())
    }

    fn last_col(&mut self) {
        self.cx = self.current().cols();
    }

    fn first_col(&mut self) {
        self.cx = 0;
    }

    fn first_non_space_col(&mut self) {
        self.cx = self.current().first_non_white_space();
    }

    fn next_col(&mut self) {
        if self.cx < self.cur.current().unwrap().cols() {
            self.cx += 1;
        } else if self.cur.peek_next().is_some() {
            self.cx = 0;
            self.scroll_rel(1);
        }
    }

    fn prev_col(&mut self) {
        if self.cx > 0 {
            self.cx -= 1;
        } else if self.cur.peek_prev().is_some() {
            self.scroll_rel(-1);
            self.cx = self.current().cols();
        }
    }

    fn next_word(&mut self) {
        self.cx = self.current().next_word_col(self.cx);
    }

    fn prev_word(&mut self) {
        self.cx = self.current().prev_word_col(self.cx);
    }

    fn backspace(&mut self) {
        let cs = self.cur.current().unwrap();
        let col = self.cx.min(cs.cols());
        if col > 0 {
            self.cx = col - cs.remove_col(col - 1);
        }
    }

    fn tab(&mut self) {
        let cs = self.cur.current().unwrap();
        let col = cs.insert_str_col(self.cx, "\t");
        self.cx += 8;
    }

    fn insert_str(&mut self, s: &str) {
        let cs = self.cur.current().unwrap();
        let col = cs.insert_str_col(self.cx, s);
        self.cx += col;
    }

    fn insert_line_above(&mut self) {
        self.cur.insert_before(Line::from(""));
    }

    fn insert_line_below(&mut self) {
        self.cur.insert_after(Line::from(""));
    }

    fn cursor(&self) -> (usize, usize) {
        (self.cx, self.cur.index().unwrap())
    }

    fn scroll_abs(&mut self, p: u16) {
        self.scroll_rel(p as i32 - self.cur.index().unwrap() as i32);
    }

    fn scroll_rel(&mut self, d: i32) {
        if d == 0 {
            return
        }

        for _ in 0..(d.abs()) {
            if d < 0 {
                if self.cur.peek_prev().is_none() {
                    break;
                }
                self.cur.move_prev();
            } else {
                if self.cur.peek_next().is_none() {
                    break;
                }
                self.cur.move_next();
            }

        }
    }
}



#[derive(Debug)]
struct Buffer {
    lines: LinkedList<Line>,
}

impl Buffer {
    fn collate<W : io::Write> (&self, w: &mut W) -> Result<()> {
        for line in self.lines.iter() {
            // TODO pick line ending
            write!(w, "{}{}", line.s, "\n")?;
        }

        Ok(())
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Buffer{
            lines: LinkedList::new(),
        }
    }
}

impl From<&str> for Buffer {
    fn from(s: &str) -> Self {
        let mut ret = Buffer::default();
        for line in s.lines() {
            ret.lines.push_back(Line::from(line));
        }

        ret
    }
}

impl From<&str> for Line {
    fn from(s: &str) -> Self {
        Line{
            s: String::from(s),
        }
    }
}

impl From<String> for Line {
    fn from(s: String) -> Self {
        Line{
            s: s,
        }
    }
}

#[derive(Debug)]
struct Line {
    s: String,
}

impl std::fmt::Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.s)
    }
}

impl Line {
    fn col2idx(&self, col: usize) -> usize {
        let mut width = 0;
        let mut idx = 0;
        for (gidx, g)  in self.s.grapheme_indices(true) {
            width += UnicodeWidthStr::width(g);
            idx = gidx;

            if width > col {
                return idx
            }
        }

        // we ran out of characters
        self.s.len()
    }

    fn as_str(&self) -> &str {
        self.s.as_str()
    }

    fn cols(&self) -> usize {
        let mut ret = 0;
        for g in UnicodeSegmentation::graphemes(self.s.as_str(), true) {
            ret += match g {
                "\t" => 8,
                _ => UnicodeWidthStr::width(g),
            };
        }

        ret
    }

    fn first_non_white_space(&self) -> usize {
        let mut ret = 0;
        for g in UnicodeSegmentation::graphemes(self.s.as_str(), true) {
            if CharClass::from_str(g) == CharClass::WhiteSpace {
                return ret
            }
            ret += UnicodeWidthStr::width(g);
        }
        ret
    }

    // returns the column of the next word
    // it will get stck in the column after the last
    // grapheme
    fn prev_word_col(&self, col: usize) -> usize {

        // algo:
        // move to previous graph
        // if prev is space
        //      move back to first non-space
        // move back to first of this type
        let idx = self.col2idx(col);
        let s = self.s.as_str();

        let mut gc = GraphemeCursor::new(idx, s.len(), true);

        let prev_idx = if let Some(i) = gc.prev_boundary(s, 0).unwrap() {
            i
        } else {
            return 0;
        };
        let mut ret = col-1;

        let mut cl = CharClass::from_char(
            s[prev_idx..].chars().next().unwrap(),
        );

        let mut last_cl = cl;

        if cl == CharClass::WhiteSpace {
            while let Some(bound) = gc.prev_boundary(s, 0).unwrap()  {
                let c = s[bound..].chars().next().unwrap();
                ret -= 1;
                last_cl = CharClass::from_char(c);
                if cl != last_cl {
                    break;
                }
            }
            cl = last_cl
        }


        while let Some(bound) = gc.prev_boundary(s, 0).unwrap()  {
            let c = s[bound..].chars().next().unwrap();
            last_cl = CharClass::from_char(c);

            if cl != last_cl {
                break;
            }
            ret -= 1;
        }

        ret
    }

    // returns the column of the next word
    // it will get stck in the column after the last
    // grapheme
    fn next_word_col(&self, col: usize) -> usize {
        let idx = self.col2idx(col);
        if idx >= self.s.len() {
            return col
        }

        // start reading grahemes at the current
        // column
        let mut gs = UnicodeSegmentation::grapheme_indices(
            &self.s[idx..],
            true);

        // there is least the last grapheme to read, so its
        // safe to unwrap
        let cl = CharClass::from_char(
            gs.next().unwrap().1.chars().next().unwrap()
        );
        let mut ret = col;

        let mut last_cl = cl;

        for (idx, g) in &mut gs {
            ret += 1;
            let c = g.chars().next().unwrap();
            last_cl = CharClass::from_char(c);
            if cl !=  last_cl {
                break;
            }
        }

        if last_cl == CharClass::WhiteSpace {
            let cl = last_cl;
            for (idx, g) in gs {
                ret += 1;
                let c = g.chars().next().unwrap();
                last_cl = CharClass::from_char(c);
                if cl !=  last_cl {
                    break;
                }
            }
        }

        ret
    }

    // removes the grapheme at the column and returns
    // its width
    fn remove_col(&mut self, col: usize) -> usize {

        let idx = self.col2idx(col);
        //unsafe {self.s.as_bytes_mut()[idx] = '#' as u8;}
        //return 0;

        let s = self.s.as_mut_str();

        let mut gc = GraphemeCursor::new(idx, s.len(), true);
        match gc.next_boundary(s, 0).unwrap() {
            Some(idx_end) => {
                // trim from the middle
                let w = UnicodeWidthStr::width(&s[idx..idx_end]);
                unsafe {
                    s.as_bytes_mut().copy_within(idx_end.., idx);
                };
                self.s.truncate(self.s.len() - (idx_end -idx));

                w
            },
            None => {
                // trim from the end
                let w = UnicodeWidthStr::width(&s[idx..]);
                self.s.truncate(idx);

                w
            },
        }
    }

    fn insert_str_col(&mut self, col: usize, s : &str) -> usize {
        let idx = self.col2idx(col);
        self.s.insert_str(idx, s);
        UnicodeWidthStr::width(s)
    }
}

#[derive(Debug,PartialEq,Eq,Clone,Copy)]
enum CharClass {
    Punctuation,
    WhiteSpace,
    LBracket,
    RBracket,
    Empty,
    Other, // hopefully, probably a letter
}

impl CharClass {
    fn from_str(s: &str) -> CharClass {
        s.chars().next().map_or(Self::Empty, |c| Self::from_char(c))
    }

    fn from_char(c: char) -> CharClass {

        if "({[<\"".find(c).is_some() {
            return CharClass::LBracket;
        }
        if ")}]>\"".find(c).is_some() {
            return CharClass::RBracket;
        }

        if char::is_ascii_punctuation(&c) {
            return CharClass::Punctuation;
        }

        if char::is_ascii_whitespace(&c) {
            return CharClass::WhiteSpace;
        }


        CharClass::Other
    }
}


