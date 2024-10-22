#![allow(unused_doc_comments)]
mod foklang;
use {libc, std::{
  env, fs, io::{self, IsTerminal, Read, Write}, path::Path, sync::Mutex
}};

static termios: Mutex<libc::termios> = Mutex::new(libc::termios { c_iflag: 0, c_oflag: 0, c_cflag: 0, c_lflag: 0, c_line: 1, c_cc: [0 as u8; 32], c_ispeed: 1, c_ospeed: 1 });


fn setup_termios() {
  termios.lock().unwrap().c_cflag &= !libc::CSIZE;
  termios.lock().unwrap().c_cflag |= libc::CS8;
  termios.lock().unwrap().c_cc[libc::VMIN] = 1;
}

extern "C" fn disable_raw_mode() {
  unsafe {
    libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, &(*termios.lock().unwrap()));
  }
}
fn enable_raw_mode() {
  unsafe {
    libc::tcgetattr(libc::STDIN_FILENO, &mut *termios.lock().unwrap());
    libc::atexit(disable_raw_mode);
    let mut raw = *termios.lock().unwrap();
    raw.c_lflag &= !(libc::ECHO | libc::ICANON);
    libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, &raw);
  }
}


#[repr(C)]              /// shout out (github.com) softprops/termsize!!
#[derive(Debug)]
pub struct UnixSize {
    pub rows: libc::c_ushort,
    pub cols: libc::c_ushort,
    x: libc::c_ushort,
    y: libc::c_ushort,
}

pub struct TerminalSize {pub rows: u16, pub cols: u16}

fn get_terminal_size() -> Option<TerminalSize> {
  if !std::io::stdout().is_terminal() {
    return None;
  }
  let mut us = UnixSize { // fuck windows
    rows: 0,
    cols: 0,
    x: 0,
    y: 0,
  };
  let r = unsafe { libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut us) };
  if r == 0 {
    Some(TerminalSize{
      rows: us.rows,
      cols: us.cols,
    })
  } else {
    None
  }
}


#[derive(Debug)]
pub struct KeyEvent {
  pub code: KeyCode,
  pub modifiers: Vec<Modifier>,
}
#[derive(Debug,PartialEq)]
pub enum Modifier {
  Control,
  //why even do it at this point
}
#[derive(Debug,PartialEq)]
pub enum Direction {
  Up,
  Down,
  Right,
  Left
}

#[derive(Debug,PartialEq)]
pub enum KeyCode {
  Escape,
  Colon,
  Enter,
  Backspace,
  Delete,
  Arrow(Direction),
  Char(char),
}

const ESCAPE: char = 27 as char;
const BACKSPACE: char = '\u{7f}';
const TAB: char = '\t';
const ENTER: char = '\n';

fn getch() -> char {
  io::stdin().bytes().next().unwrap().unwrap() as char
}



#[derive(Debug,Clone,PartialEq)]
enum BufferType {
  File,
  Terminal
}
#[derive(Debug,Clone,PartialEq)]
struct EditorBuffer {
  cursor: (u32, u32),
  lines: Vec<String>,
  display_start_line: u32,
  display_offset_collumn: u32,
  buf_type: BufferType,
  buf_name: String,
  save_path: String,
}
#[derive(Debug,Clone,PartialEq)]
enum State {
  Control,
  Command,
  Input,
}

#[derive(Debug,Clone,PartialEq)]
struct Program {
  state: State,                 // Terminal State, described further in State enum 
  buffers: Vec<EditorBuffer>,   // Buffers, windows open - listed in 1st line
  current: usize,               // current Buffer index

  foklang: foklang::foklang::Foklang,    // foklang instance
  io: String,                   // lower line command line
  io_cursor: u32,               // location of cursor in IO (x)
  io_history: Vec<String>,      // history of used commands to scroll via arrows
  io_history_index: usize,      // index of history
  exit: bool,                   // whether to exit at the end of loop
}
trait Editor {
  fn evaluate_io(&mut self) -> String;              // evaluate terminal line; use modified foklang for that
  fn display(&mut self);
  fn clear(&mut self);
  fn get_buffer(&mut self) -> &mut EditorBuffer;
  fn move_cursor(&mut self, vector: (i32, i32));
  fn move_io_cursor(&mut self, vector: i32);
  fn write_string(&mut self, string: String);

}
impl Editor for Program {
  fn evaluate_io(&mut self) -> String {
    let mut ch = self.io.chars();
    ch.next();

    let mut foklang = self.foklang.clone();

    let panics = std::panic::catch_unwind(|| {
      let (program,io) = foklang.run(ch.collect::<String>(), self.clone()); // foklang.run returns display of returned value from foklang code
      drop(foklang);
      (program,io)}
    );
    if panics.is_ok() {
      let uw = panics.unwrap();
      *self = uw.0;
      uw.1
    } else {
      String::from("Foklang panicked.")
    }
  }
  fn clear(&mut self) {
    print!("\x1b[?47l\x1b 8");
    let _ = io::stdout().flush();
  }
  fn display(&mut self) {
    let (tery, terx) = (get_terminal_size().unwrap().rows,  get_terminal_size().unwrap().cols);
    let mut result = String::new();
    result += "\x1b[2J\x1b[H";
    let max_buf_display_len = 16;
    for i in 0..self.buffers.len() {
      if i == self.current {
        result+="\x1b[48;2;67;14;119m\x1b[1m";
      } else {
        result+="\x1b[48;2;46;14;79m\x1b[22m";
      }
      
      let mut buf_name = self.buffers[i].buf_name.clone();
      while buf_name.len() > max_buf_display_len {
        let mut buf_ch = buf_name.chars();
        buf_ch.next();
        buf_name = buf_ch.collect::<String>();
      }
      if buf_name.len() == max_buf_display_len {
        let mut buf_ch = buf_name.chars();
        buf_ch.next();
        buf_ch.next();
        buf_name = buf_ch.collect::<String>();
      }

      result += &(vec![" "; ((max_buf_display_len - buf_name.len()) as f32 / 2.0).floor() as usize].into_iter().collect::<String>() + &buf_name + &vec![" "; ((max_buf_display_len - buf_name.len()) as f32 / 2.0).ceil() as usize].into_iter().collect::<String>());
    }
    result += "\x1b[22m\x1b[48;2;65;31;86m";
    result += &(vec![" "; terx as usize - self.buffers.len()*max_buf_display_len]).into_iter().collect::<String>();
    //result +=  "\x1b[0m";
    result += "\x1b[48;2;44;22;58m";
    //let mut display_sl = 0 as u32;
    if self.get_buffer().cursor.1 > (tery-4) as u32 + self.get_buffer().display_start_line {
      self.get_buffer().display_start_line = self.get_buffer().cursor.1 % (tery-4) as u32;
    } else if self.get_buffer().cursor.1 < self.get_buffer().display_start_line {
      self.get_buffer().display_start_line = self.get_buffer().cursor.1;
    }
    //self.get_buffer().display_start_line = display_sl;
    let left = self.get_buffer().display_start_line as usize;

    
    if (self.get_buffer().lines.len() as u16) < ((tery-3) - left as u16) {
      for i in self.get_buffer().lines.clone() {
        result += &(i.to_owned() + &vec![" "; terx as usize - i.len() ].into_iter().collect::<String>() + "\n");
      }
      result += "\x1b[38;2;5;8;191m";
      for i in 0..(((tery-3) - left as u16) - self.get_buffer().lines.len() as u16) {
        result += &("~".to_owned() + &vec![" "; terx as usize - 1].into_iter().collect::<String>() + "\n");
      }
      result += "\x1b[38;2;255;255;255m";
    } 
    else {
      for i in &self.get_buffer().lines[left..left+(tery-3) as usize] {
        result += &(i.to_owned() + &vec![" "; terx as usize - i.len() ].into_iter().collect::<String>() + "\n");
      }
    }


    result += "\x1b[48;2;65;31;86m";
    result += &(vec![" "; terx as usize]).into_iter().collect::<String>();
    result += "\n";


    while self.io.len() > (terx as usize -1) {
      let mut ioc = self.io.chars();
      ioc.next();
      self.io = ioc.collect();
    }
    let c = self.get_buffer().cursor;
    let cursor_string = format!("{}::{}:{};", self.get_buffer().display_start_line, c.0, c.1);
    result += &(self.io.clone() + &(vec![" "; terx as usize - self.io.len() - 1 - cursor_string.len()]).into_iter().collect::<String>());
    result += &cursor_string;
    result += match self.state {
      State::Input => "1",
      State::Control => "0",
      State::Command => "2",
    };
    match self.state {
      State::Command => {
        let column = self.io_cursor+1;
        result += &format!("\x1b[{tery};{column}H");
      },
      _ => {
        let column = self.get_buffer().cursor.0+1;
        result += &format!("\x1b[{line};{column}H", line=self.get_buffer().cursor.1+2 - self.get_buffer().display_start_line);
      },
    };
    result += "\x1b[0m";
    print!("{}", result);
    let _ = io::stdout().flush();
  }


  fn get_buffer(&mut self) -> &mut EditorBuffer {
    return &mut self.buffers[self.current]
  }
  fn move_cursor(&mut self, vector: (i32, i32)) {
    let mut n0 = self.get_buffer().cursor.0 as i32 + vector.0;
    let mut n1 = self.get_buffer().cursor.1 as i32 + vector.1;

    if n1<0 {
      n1 = 0;
    } else if n1 >= self.get_buffer().lines.len() as i32 {
      n1 = self.get_buffer().lines.len() as i32 -1;
    }
    if vector.1 != 0 {
      if n0 > self.get_buffer().lines[n1 as usize].len() as i32  {
        n0 = self.get_buffer().lines[n1 as usize].len() as i32;
      }
    }

    let y = n1 as usize;
    let line_len = self.get_buffer().lines[y].len() as i32;
    if n0 < 0 {
      n0 = 0;
    } else if n0 > line_len {
      n0 = line_len;
    }

    self.get_buffer().cursor.0 = n0 as u32;
     self.get_buffer().cursor.1 = n1 as u32;
   
  }
  fn move_io_cursor(&mut self, vector: i32) {
    let mut n0 = self.io_cursor as i32 + vector;
    if n0 < 0 {
      n0 = 0;
    } else if n0 > self.io.len() as i32 {
      n0 = self.io.len() as i32;
    }
    self.io_cursor = n0 as u32;
  }
  fn write_string(&mut self, string: String) {
    let index = (self.get_buffer().cursor.1) as usize;
    let x = self.get_buffer().cursor.0 as usize;
    let str1 = self.get_buffer().lines[index][..x].to_owned() + &string;
    self.get_buffer().lines[index] = str1 + &self.get_buffer().lines[index][x..];
  }
}
impl EditorBuffer {
  fn compile_text(&mut self) -> String {
    let mut result = String::new();
    for i in self.lines.clone() {
      result += &(i + "\n");
    }
    result.remove(result.len()-1);
    result
  }
}
fn handle_key_event(program: &mut Program, event: KeyEvent) {
  let (tery, terx) = (get_terminal_size().unwrap().rows,  get_terminal_size().unwrap().cols);
  match event.code {
    KeyCode::Enter => {
      match program.state {
        State::Command => {
          program.io = program.evaluate_io();
          program.state = State::Control;
        },
        State::Input => {
          let index = program.get_buffer().cursor.1;
          let index2 = program.get_buffer().cursor.0;
          let mut leftlist = program.get_buffer().lines[..index as usize].into_iter().map(|x| x.to_string()).collect::<Vec<String>>();
          leftlist.push(program.get_buffer().lines[index as usize][..index2 as usize].to_string());
          leftlist.push(program.get_buffer().lines[index as usize][index2 as usize..].to_string());
          leftlist.append(&mut program.get_buffer().lines[index as usize+1..].into_iter().map(|x| x.to_string()).collect::<Vec<String>>());
          program.get_buffer().lines = leftlist;
          program.move_cursor((-i16::MAX as i32, 1));
        },
        State::Control => {
          program.move_cursor((-i16::MAX as i32, 1));
        },
      }
    },
    KeyCode::Escape => {
      match program.state {
        State::Command => {
          program.state = State::Control;
        },
        State::Input => {
          program.state = State::Control;
        },
        State::Control => {},
      }
    },
    KeyCode::Delete => {
      match program.state {
        State::Command => {
          let mut ioc = program.io.chars().collect::<Vec<char>>();
          if ioc.len() > program.io_cursor as usize {
            ioc.remove(program.io_cursor as usize);
            program.io = ioc.into_iter().collect::<String>();
          }
        },
        State::Control => {
          let index = (program.get_buffer().display_start_line + program.get_buffer().cursor.1) as usize;
          let x = program.get_buffer().cursor.0 as usize;
          let ic = program.get_buffer().lines[index].clone();
          let strc = ic[..x].chars();
          let mut right = ic[x..].chars();
          right.next();
          program.get_buffer().lines[index] = strc.collect::<String>() + &right.collect::<String>();
        },
        State::Input => {

        },
      }
    },
    KeyCode::Backspace => {
      match program.state {
        State::Command => {
          let mut ioc = program.io.chars().collect::<Vec<char>>();
          ioc.remove(program.io_cursor as usize -1);
          program.io = ioc.into_iter().collect::<String>();
          program.move_io_cursor(-1);
          if program.io.len()==0 {
            program.state = State::Control;
          }
        },
        State::Input => {
          if program.get_buffer().cursor.0>0 {
            let index = (program.get_buffer().display_start_line + program.get_buffer().cursor.1) as usize;
            let x = program.get_buffer().cursor.0 as usize;
            let mut strc = program.get_buffer().lines[index][..x].chars();
            strc.next_back();
            program.get_buffer().lines[index] = strc.collect::<String>() + &program.get_buffer().lines[index][x..];
            
            program.move_cursor((-1,0));
          } else if program.get_buffer().cursor.0 == 0 && program.get_buffer().cursor.1 > 0 {
            let cursor = program.get_buffer().cursor.1;
            let cline = program.get_buffer().lines[cursor as usize].clone();
            program.get_buffer().lines[cursor as usize -1] += &cline;
            program.get_buffer().lines.remove(cursor as usize);
            let x = (program.get_buffer().lines[cursor as usize -1].len() - cline.len()) as i32;
            program.move_cursor((x, -1));
          }
        },
        State::Control => {
          if program.get_buffer().cursor.0 == 0 && program.get_buffer().cursor.1 > 0 {
            program.move_cursor((i16::MAX as i32, -1));
          } else {
            program.move_cursor((-1,0));
          }
        },
      }
    },
    KeyCode::Colon => {
      match program.state {
        State::Command => {
          program.io += ":";
          program.move_io_cursor(1);
          
        },
        State::Input => {
          program.write_string(String::from(":"));
        },
        State::Control => {
          program.state = State::Command;
          program.io = String::from(":");
          program.io_cursor = 1;
        },
      }

    },
    KeyCode::Arrow(d) => {
      match d {
        Direction::Up => {
          match program.state {
            State::Command => {
              if program.io_history_index+1 < program.io_history.len() {
                program.io_history_index += 1;
                program.io = program.io_history[program.io_history_index].clone();
              } 
            }
            _ => {
              program.move_cursor((0, -1));
            }
          }
        },
        Direction::Down => {
          match program.state {
            State::Command => {
              if program.io_history_index-1 >= 0 {
                program.io_history_index -= 1;
                program.io = program.io_history[program.io_history_index].clone();
              }
            }
            _ => {
              program.move_cursor((0, 1));
            }
          }
        },
        Direction::Left => {
          match program.state {
            State::Command => {
              program.move_io_cursor(-1);
            },
            _ => {
              program.move_cursor((-1, 0));
            }
          }
        },
        Direction::Right => {
          match program.state {
            State::Command => {
              program.move_io_cursor(1);
            },
            _ => {
              program.move_cursor((1, 0));
            }
          }
        },
      }
    },
    KeyCode::Char(c) => {
      match program.state { 
        State::Command => {
          let left = (program.io[0..program.io_cursor as usize]).to_owned() + &c.to_string();
          program.io = left + &program.io[program.io_cursor as usize..];
          program.move_io_cursor(1);
        },
        State::Control => {
          match c {
            'i' => {program.state = State::Input;},
            'a' => {program.state = State::Input;},
            _ => {
              //program.io = String::from("You're in Control Mode!");
            },
          }
        },
        State::Input => {
          program.write_string(c.to_string());
          program.move_cursor((1,0));
        },
      }
    },
  }
}


fn main() {
  /// USAGE: `executable [files]`
  setup_termios();
  enable_raw_mode();
  let mut program = Program {
    io: String::new(),          // default no command
    state: State::Control,      // default to Control State
    buffers: vec![],            // no windows opened; parsing args `command [args]` will append here
    current: 0,
    io_cursor: 0,
    io_history: vec![],
    io_history_index: 0,
    exit: false,

    foklang: foklang::foklang::Foklang::new(),
  };
  let mut args = env::args();
  args.next();
  for i in args {
    if Path::new(&i).exists() {
      program.buffers.push(
        EditorBuffer {
          cursor: (0, 0),
          lines: fs::read_to_string(i.clone()).unwrap().split("\n").collect::<Vec<&str>>().into_iter().map(|x| String::from(x)).collect::<Vec<String>>(),
          buf_type: BufferType::File,
          display_start_line: 0,
          display_offset_collumn: 0,
          buf_name: i.clone(),
          save_path: i,
        }
      );
    } else {
      program.buffers.push(
        EditorBuffer {
          cursor: (0, 0),
          lines: vec![],
          buf_type: BufferType::File,
          display_start_line: 0,
          display_offset_collumn: 0,
          buf_name: i.clone(),
          save_path: i,
        }
      );
    }
  }
  if program.buffers.len() == 0 {
    program.buffers.push(
      EditorBuffer {
        cursor:  (0, 0),
        lines: vec![],
        buf_type: BufferType::File,
        display_start_line: 0,
        display_offset_collumn: 0,
        buf_name: String::from("unnamed"),
        save_path: String::from(""),
      }
    );
  }



  print!("\x1b 7\x1b[?47h");
  /// MAIN_LOOP 

  program.display();
  for b in io::stdin().bytes() {
    
    //println!("{:#?}", (*program.lock().unwrap()).state);
    
    let c = b.unwrap() as char;
    //println!("{}", c);
    let mut modifiers: Vec<Modifier> = vec![];
    if c.is_control() && ![ENTER, TAB, ESCAPE, BACKSPACE].contains(&c) {
      modifiers.push(Modifier::Control);
    }
    
    let event = KeyEvent{
      code: match c { BACKSPACE => KeyCode::Backspace, ':' => KeyCode::Colon, '\n' => KeyCode::Enter,
          '\u{1b}' => {
              match getch() { 
                    '[' => match getch() {'A' => KeyCode::Arrow(Direction::Up), 'B' => KeyCode::Arrow(Direction::Down), 'C' => KeyCode::Arrow(Direction::Right), 'D' => KeyCode::Arrow(Direction::Left),
                            _ => KeyCode::Escape }, 
                    _ => KeyCode::Escape}},
          _ => KeyCode::Char(c)},
      modifiers,
    };
    //program.io =  format!("{:#?}", event);
    handle_key_event(&mut program, event);
    if program.exit {
      break;
    }
    program.display();
  }
  program.clear();

}



