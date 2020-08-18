use std::{
    io::{
        self,
    },
    collections::LinkedList,
    default::Default,
    convert::From,
};

use crate::{
    Result,
};



#[derive(Debug)]
pub struct Buffer {
    pub(super) lines: LinkedList<String>,
}

impl Buffer {
    pub fn collate<W : io::Write> (&self, w: &mut W) -> Result<()> {
        for line in self.lines.iter() {
            // TODO pick line ending
            write!(w, "{}{}", line, "\n")?;
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
            ret.lines.push_back(line.into());
        }

        ret
    }
}

