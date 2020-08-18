
use std::{
    collections::linked_list::CursorMut,
};

use crate::{
    Line,
    LineMut,
    LineConfig,
    Buffer,
    Renderer,
    Result,
};

pub struct View<'buf> {
    cur : CursorMut<'buf, String>,
    lcfg : &'buf LineConfig,
    cx : usize, // x position in current line
}

impl<'buf> View <'buf> {
    pub fn new(buf: &'buf mut Buffer) -> Self {
        if buf.lines.len() == 0 {
            buf.lines.push_back("".into());
        }

        View{
            cur: buf.lines.cursor_front_mut(),
            cx: 0,
            lcfg: &LineConfig{
                tab_width: 8,
            },
        }
    }


    fn current(&self) -> Line {
        Line{
            s: self.cur.as_cursor().current().unwrap().as_str(),
            cfg: self.lcfg,
        }
    }

    fn current_mut(&mut self) -> LineMut {
        LineMut{
            s: self.cur.current().unwrap(),
            cfg: self.lcfg,
        }
    }

    pub fn render<R : Renderer> (&self, r: &mut R) -> Result<()> {
        // half height
        let hh = r.height() / 2;

        r.move_y(hh as isize)?;

        // go up
        let mut up_lines = 0;
        let mut cur = self.cur.as_cursor();
        for _n in 0..hh {
            if let Some(line) = cur.current() {
                r.write(line.as_str())?;
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
        for _n in 0..hh {
            if let Some(line) = cur.current() {
                r.write(line.as_str())?;
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

    pub fn last_col(&mut self) {
        self.cx = self.current().cols();
    }

    pub fn first_col(&mut self) {
        self.cx = 0;
    }

    pub fn first_non_space_col(&mut self) {
        self.cx = self.current().first_non_white_space();
    }

    pub fn next_col(&mut self) {
        if self.cx < self.current().cols() {
            self.cx += 1;
        } else if self.cur.peek_next().is_some() {
            self.cx = 0;
            self.scroll_rel(1);
        }
    }

    pub fn prev_col(&mut self) {
        if self.cx > 0 {
            self.cx -= 1;
        } else if self.cur.peek_prev().is_some() {
            self.scroll_rel(-1);
            self.cx = self.current().cols();
        }
    }

    pub fn next_word(&mut self) {
        self.cx = self.current().next_word_col(self.cx);
    }

    pub fn prev_word(&mut self) {
        self.cx = self.current().prev_word_col(self.cx);
    }

    pub fn backspace(&mut self) {
        let cx = self.cx;
        let mut cs = self.current_mut();
        let col = cx.min(cs.as_line().cols());

        if col > 0 {
            self.cx = col - cs.remove_col(col - 1);
        }
    }

    pub fn tab(&mut self) {
        let cx = self.cx;
        let mut cs = self.current_mut();
        self.cx = cs.insert_str_col(cx, "\t");
    }

    pub fn insert_str(&mut self, s: &str) {
        let cx = self.cx;
        let mut cs = self.current_mut();
        let col = cs.insert_str_col(cx, s);
        self.cx += col;
    }

    pub fn insert_line_above(&mut self) {
        self.cur.insert_before("".into());
    }

    pub fn insert_line_below(&mut self) {
        self.cur.insert_after("".into());
    }

    pub fn cursor(&self) -> (usize, usize) {
        (self.cx, self.cur.index().unwrap())
    }

    pub fn scroll_abs(&mut self, p: u16) {
        self.scroll_rel(p as i32 - self.cur.index().unwrap() as i32);
    }

    pub fn scroll_rel(&mut self, d: i32) {
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
