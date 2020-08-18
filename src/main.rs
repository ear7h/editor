#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(dead_code)]

use std::{
    mem,
    io,
    convert::From,
};

use termion::{
    input::TermRead,
    event::Key,
};


use editor::{
    Result,
    Renderer,
    TerminalRenderer,
    Buffer,
    View,
};


fn main() {
    let mut buf = Buffer::from("hello\nworld");

    let stdout = io::stdout();
    let stdin = io::stdin();

    let mut view0 = View::new(&mut buf);
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
        r.goto().unwrap();
        view0.render(&mut r).unwrap();

        let cur = r.save_curs();
        r.set_xy(0, (r.height-1) as usize).unwrap();
        r.goto().unwrap();
        status_line(&mut r, edit_mode).unwrap();
        r.load_curs(cur);

        r.flush().unwrap();
    }

    mem::drop(r);

    println!("{}", "buffer: ");
    buf.collate(&mut io::stdout()).unwrap();
}

fn status_line<R: Renderer>(r: &mut R, edit_mode : bool) -> Result<()> {
    if edit_mode {
        r.write("edit")?;
    } else {
        r.write("insert")?;
    }

    Ok(())
}


