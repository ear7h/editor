use unicode_segmentation::{
    UnicodeSegmentation,
    GraphemeCursor,
};

use unicode_width::{
    UnicodeWidthStr,
};


#[derive(Debug)]
pub struct LineConfig {
    pub tab_width: u8,
}

#[derive(Debug)]
pub struct Line<'a, 'b> {
    pub(super) s: &'a str,
    pub(super) cfg: &'b LineConfig,
}

impl<'a,'b> Line <'a, 'b> {
   fn col2idx(&self, col: usize) -> usize {
        let mut width = 0;
        let mut idx;
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

    pub fn cols(&self) -> usize {
        let mut ret = 0;
        for g in UnicodeSegmentation::graphemes(self.s, true) {
            ret += match g {
                "\t" => self.cfg.tab_width as usize,
                _ => UnicodeWidthStr::width(g),
            };
        }

        ret
    }

    pub fn first_non_white_space(&self) -> usize {
        let mut ret = 0;
        for g in UnicodeSegmentation::graphemes(self.s, true) {
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
    pub fn prev_word_col(&self, col: usize) -> usize {

        // algo:
        // move to previous graph
        // if prev is space
        //      move back to first non-space
        // move back to first of this type
        let idx = self.col2idx(col);
        let s = self.s;

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
    pub fn next_word_col(&self, col: usize) -> usize {
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

        for (_, g) in &mut gs {
            ret += 1;
            let c = g.chars().next().unwrap();
            last_cl = CharClass::from_char(c);
            if cl !=  last_cl {
                break;
            }
        }

        if last_cl == CharClass::WhiteSpace {
            let cl = last_cl;
            for (_, g) in gs {
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
}

#[derive(Debug)]
pub struct LineMut<'a, 'b> {
    pub(super) s: &'a mut String,
    pub(super) cfg: &'b LineConfig,
}

impl<'a,'b> LineMut<'a, 'b> {

    pub fn as_line(&self) -> Line<'_, '_> {
        Line{
            s: self.s.as_str(),
            cfg: self.cfg,
        }
    }

    // removes the grapheme at the column and returns
    // its width
    pub fn remove_col(&mut self, col: usize) -> usize {

        let idx = self.as_line().col2idx(col);
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

    pub fn insert_str_col(&mut self, col: usize, s : &str) -> usize {
        let idx = self.as_line().col2idx(col);
        self.s.insert_str(idx, s);
        let ntabs = s.chars()
                .filter(|c| *c == '\t')
                .count();

        UnicodeWidthStr::width(s) + (self.cfg.tab_width as usize * ntabs)
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

