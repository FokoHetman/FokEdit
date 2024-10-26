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


#[derive(Debug,Clone,PartialEq)]
pub struct KeyEvent {
  pub code: KeyCode,
  pub modifiers: Vec<Modifier>,
}
#[derive(Debug,PartialEq,Clone)]
pub enum Modifier {
  Control,
  Shift,
  //why even do it at this point
}
#[derive(Debug,PartialEq,Clone)]
pub enum Direction {
  Up,
  Down,
  Right,
  Left
}

#[derive(Debug,PartialEq,Clone)]
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
fn get_arrow() -> KeyCode {
  match getch() {'A' => KeyCode::Arrow(Direction::Up), 'B' => KeyCode::Arrow(Direction::Down), 'C' => KeyCode::Arrow(Direction::Right), 'D' => KeyCode::Arrow(Direction::Left),
                                                           _ => KeyCode::Escape }
}

#[derive(Debug,Clone,PartialEq)]
pub struct EmptyLine {
  text: String,
}

#[derive(Debug,Clone,PartialEq)]
pub struct RGB {
  r: u8,
  g: u8,
  b: u8,
}


#[derive(Debug,Clone,PartialEq)]
pub struct Keybinds {
  keybinds: Vec<(KeyEvent,String)>,
}

impl Default for Keybinds {
  fn default() -> Self {
    Self {keybinds: vec![]}
  }
}

#[derive(Debug,Clone,PartialEq)]
pub struct DebugConfig {
  cursor: bool,
}

#[derive(Debug,Clone,PartialEq)]
pub struct ElementsConfig {
  empty_line: EmptyLine,
  debug: DebugConfig,
}

impl Default for ElementsConfig {
  fn default() -> Self {
    Self {
      empty_line: EmptyLine{text: String::from("~")},
      debug: DebugConfig{cursor: true},
    }
  }
}

#[derive(Debug,Clone,PartialEq)]
pub struct ColorConfig {
  background: RGB,
  foreground: RGB,
  border: RGB,

  empty_line_color: RGB,

  active_buffer: RGB,
  inactive_buffer: RGB,

  io_background: RGB,
  io_foreground: RGB,
}
impl Default for ColorConfig {
  fn default() -> Self {
    /*Self {background: RGB{r: 65, g: 31, b: 86}, foreground: RGB{r: 255, g: 255, b: 255}, border: RGB{r: 40, g: 40, b: 40},
          active_buffer: RGB{r: 75, g: 81, b: 66}, inactive_buffer: RGB{r: 46, g: 14, b: 79}, empty_line_color: RGB{r: 0, g: 0, b: 255},
          io_background: RGB{r: 56, g: 68, b: 37}, io_foreground: RGB{r: 255, g: 255, b: 255}*/ 
      Self {background: RGB{r: 255, g: 255, b: 255}, foreground: RGB{r: 255, g: 255, b: 255}, border: RGB{r: 255, g: 255, b: 255},
          active_buffer: RGB{r: 255, g: 255, b: 255}, inactive_buffer: RGB{r: 255, g: 255, b: 255}, empty_line_color: RGB{r: 255, g: 255, b: 255},
          io_background: RGB{r: 255, g: 255, b: 255}, io_foreground: RGB{r: 255, g: 255, b: 255}
    }
  }
}

#[derive(Debug,Clone,PartialEq)]
pub struct FokEditConfig {
  colors: ColorConfig,
  elements: ElementsConfig,
  keybinds: Keybinds,
  /*
  highlighting: HighlightingConfig,
  options: FokEditOpts,
  */
}
impl Default for FokEditConfig {
  fn default() -> Self {
    Self {colors: ColorConfig{..Default::default()}, elements: ElementsConfig{..Default::default()}, keybinds: Keybinds{..Default::default()}}
  }
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
pub struct Program {
  state: State,                 // Terminal State, described further in State enum 
  buffers: Vec<EditorBuffer>,   // Buffers, windows open - listed in 1st line
  current: usize,               // current Buffer index

  foklang: foklang::foklang::Foklang,    // foklang instance
  io: String,                   // lower line command line
  io_cursor: u32,               // location of cursor in IO (x)
  io_history: Vec<String>,      // history of used commands to scroll via arrows
  io_history_index: usize,      // index of history
  exit: bool,                   // whether to exit at the end of loop

  config: FokEditConfig,
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
        let color_pack = self.config.colors.active_buffer.clone();
        result+=&format!("\x1b[48;2;{r};{g};{b}m\x1b[1m", r=color_pack.r, g=color_pack.g, b=color_pack.b);
      } else {
        let color_pack = self.config.colors.inactive_buffer.clone();
        result+=&format!("\x1b[48;2;{r};{g};{b}m\x1b[22m", r=color_pack.r, g=color_pack.g, b=color_pack.b);
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
    
    let border_color = self.config.colors.border.clone();
    let background_color = self.config.colors.background.clone();

    let foreground_color = self.config.colors.foreground.clone();

    result += &format!("\x1b[22m\x1b[48;2;{r};{g};{b}m", r=border_color.r, g=border_color.g, b=border_color.b);
    result += &(vec![" "; terx as usize - self.buffers.len()*max_buf_display_len]).into_iter().collect::<String>();
    //result +=  "\x1b[0m";
    result += &format!("\x1b[38;2;{r2};{g2};{b2}m\x1b[48;2;{r};{g};{b}m", r=background_color.r, g=background_color.g, b=background_color.b, r2=foreground_color.r, g2=foreground_color.g, b2=foreground_color.b);
    //let mut display_sl = 0 as u32;
    let free_y = tery-3;
    let free_x = terx;
    if self.get_buffer().cursor.1 > (free_y-1) as u32 + self.get_buffer().display_start_line {

      if ((free_y-1) as u32) < self.get_buffer().cursor.1 {
        self.get_buffer().display_start_line = self.get_buffer().cursor.1 - (free_y-1) as u32;
      } else {
        self.get_buffer().display_start_line = (free_y-1) as u32 - self.get_buffer().cursor.1;
      }

    } else if self.get_buffer().cursor.1 < self.get_buffer().display_start_line {
      self.get_buffer().display_start_line = self.get_buffer().cursor.1;
    }

    if self.get_buffer().cursor.0 > (free_x-1) as u32 + self.get_buffer().display_offset_collumn {

      if ((free_x-1) as u32) < self.get_buffer().cursor.0 {
        self.get_buffer().display_offset_collumn = self.get_buffer().cursor.0 - (free_x-1) as u32;
      } else {
        self.get_buffer().display_offset_collumn = (free_x-1) as u32 - self.get_buffer().cursor.0;
      }
    } else if self.get_buffer().cursor.0 < self.get_buffer().display_offset_collumn {
      self.get_buffer().display_offset_collumn = self.get_buffer().cursor.0;
    }
    

    let left = self.get_buffer().display_start_line as usize;

    let offset = self.get_buffer().display_offset_collumn as usize;
    if (self.get_buffer().lines.len() as u16) < (free_y + left as u16) {
      let rlen = self.get_buffer().lines.len();
      
      for i in &self.get_buffer().lines[left..rlen] {
        if offset > 0 {
          if i.len()  > offset {
            let max = std::cmp::min(offset+free_x as usize, i.len());
            result += &(i.to_owned()[offset..max].to_owned() + &vec![" "; free_x as usize - (max-offset)].into_iter().collect::<String>() + "\n");
          } else {
            result += &(vec![" "; free_x as usize ].into_iter().collect::<String>() + "\n");
          }
        } else {
          if i.len() <= free_x as usize {
            result += &(i.to_owned() + &vec![" "; free_x as usize - i.len() ].into_iter().collect::<String>() + "\n");
          } else {
            result += &(i.to_owned()[..free_x as usize].to_owned() + "\n");
          }
        }
      }

      let empty_line_color = self.config.colors.empty_line_color.clone();
      let empty_line_text = self.config.elements.empty_line.text.clone();

      result += &format!("\x1b[38;2;{r};{g};{b}m", r=empty_line_color.r, g=empty_line_color.b, b=empty_line_color.b);
      for i in 0..(((free_y) as u16) - (rlen - left) as u16) {
        result += &(empty_line_text.to_owned() + &vec![" "; free_x as usize - 1].into_iter().collect::<String>() + "\n");
      }
      result += "\x1b[38;2;255;255;255m";
    } 
    else {

      //let reallen = self.get_buffer().lines.len();
      for i in &self.get_buffer().lines[left..left+(free_y) as usize] {
        if offset > 0 {
          if i.len()  > offset {
            let max = std::cmp::min(offset+free_x as usize, i.len());
            result += &(i.to_owned()[offset..max].to_owned() + &vec![" "; free_x as usize - (max-offset)].into_iter().collect::<String>() + "\n");
          } else {
            result += &(vec![" "; free_x as usize ].into_iter().collect::<String>() + "\n");
          }
        } else {
          if i.len() <= free_x as usize {
            result += &(i.to_owned() + &vec![" "; free_x as usize - i.len() ].into_iter().collect::<String>() + "\n");
          } else {
            result += &(i.to_owned()[..free_x as usize].to_owned() + "\n");
          }
        }
      }
    }
      //if i.len()<free_x {
      //    result += &(i.to_owned() + &vec![" "; free_x as usize - i.len() ].into_iter().collect::<String>() + "\n");
      //  }

    let io_background = self.config.colors.io_background.clone();
    let io_foreground = self.config.colors.io_foreground.clone();

    result += &format!("\x1b[38;2;{r2};{g2};{b2}m\x1b[48;2;{r};{g};{b}m", r=io_background.r, g=io_background.g, b=io_background.b, r2=io_foreground.r, g2=io_foreground.g, b2=io_foreground.b);
    result += &(vec![" "; terx as usize]).into_iter().collect::<String>();
    result += "\n";


    while self.io.len() > (terx as usize -1) {
      let mut ioc = self.io.chars();
      ioc.next();
      self.io = ioc.collect();
    }
    let c = self.get_buffer().cursor;
    let col = self.get_buffer().display_offset_collumn;
    let cursor_string = format!("{}:{}:;:{}:{};", col, self.get_buffer().display_start_line, c.0, c.1);
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
        let column = self.get_buffer().cursor.0+1 - self.get_buffer().display_offset_collumn;
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
  let mut is_keybind = false;
  for i in program.config.keybinds.keybinds.clone() {
    if i.0 == event {
      is_keybind = true;
      let mut foklang = program.foklang.clone();

      let panics = std::panic::catch_unwind(|| {
        let (program,io) = foklang.run(i.1.clone(), program.clone()); // foklang.run returns display of returned value from foklang code
        drop(foklang);
        (program,io)
      });
      if panics.is_ok() {
        let uw = panics.unwrap();
        *program = uw.0;
        program.io = uw.1
      } else {
        program.io = format!("Foklang panicked on keybind evaluation: {}.", i.1)
      }
    }
  }
  if !is_keybind {
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
              let index = (program.get_buffer().cursor.1) as usize;
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
              if program.get_buffer().cursor.1 == program.get_buffer().lines.len() as u32 {
                program.get_buffer().display_start_line -= 1;
              }
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
                if program.io_history_index-1 >= 0 { // what
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
    config: FokEditConfig{..Default::default()},
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
          lines: vec![String::new()],
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
        lines: vec![String::new()],
        buf_type: BufferType::File,
        display_start_line: 0,
        display_offset_collumn: 0,
        buf_name: String::from("unnamed"),
        save_path: String::from(""),
      }
    );
  }



  if Path::new(Path::new(&(env::var("HOME").unwrap() + "/.config/FokEdit/configuration.fok"))).exists() {
    let raw = program.foklang.raw_run(String::from("rgb x y z = x:(y:[z]);") 
        + &fs::read_to_string(&(env::var("HOME").unwrap() + "/.config/FokEdit/configuration.fok")).unwrap(), program.clone());
    
    let ran = foklang::core::builtins::load_fokedit_config(foklang::core::builtins::Arguments { function: foklang::core::builtins::FunctionArgs::singleProgram(raw, program.clone()) });

    match ran.value {
      foklang::core::AST::Fructa::ProgramModifier(nprog) => {
        program = nprog;
      }
      _ => {}
    }
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
                    '[' => match getch() {
                        'A' => KeyCode::Arrow(Direction::Up), 'B' => KeyCode::Arrow(Direction::Down), 'C' => KeyCode::Arrow(Direction::Right), 'D' => KeyCode::Arrow(Direction::Left),
                        '1' => match getch() {
                                ';' => match getch() 
                                { '5' => {modifiers.push(Modifier::Control); get_arrow()}, '2' => {modifiers.push(Modifier::Shift); get_arrow()}, _ => KeyCode::Escape}, _ => KeyCode::Escape
                            },
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



