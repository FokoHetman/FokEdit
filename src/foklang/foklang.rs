use crate::{foklang::core, Editor};
use std::{
  cmp, env, fs, io::{self, Read, Write}
};
use crate::Program;

#[derive(Clone,Debug,PartialEq)]
pub struct Foklang {
  tokenizer: core::tokenizer::Tokenizer,
  parser: core::parser::Parser,
  interpreter: core::interpreter::Interpreter,
  pub env: core::env::Environment,
}
impl Foklang {
  pub fn new() -> Self {
    let tokenizer = core::tokenizer::Tokenizer {};
    let mut parser = core::parser::Parser {};
    let error_handler = core::error_handler::ErrorHandler {};
    let mut env = core::env::Environment{ error_handler, ..Default::default() };
    core::builtins::declare_builtins(&mut env);
    let mut interpreter = core::interpreter::Interpreter {error_handler, tokenizer, parser};

    return Foklang{tokenizer, parser, interpreter, env}
  }
  pub fn run(&mut self, input: String, program: Program) -> (Program,String) {
    let mut tokenized_input = self.tokenizer.tokenize(input);
    let mut parsed_input = self.parser.parse(tokenized_input);
    let mut interpreted_input = self.interpreter.evaluate(parsed_input, &mut self.env, program.clone());

    let value = interpreted_input.value;
    match value {
      core::AST::Fructa::ProgramModifier(nprogram) => {
        (nprogram.clone(), nprogram.io)
      }
      core::AST::Fructa::Numerum(i) => {
        let mut i = i;
        let mut nprogram = program.clone();
        if i<0 { i = 0 };
        nprogram.get_buffer().cursor.1 = cmp::min((nprogram.get_buffer().lines.len() as i32-1).abs() as u32, i as u32);
        (nprogram, value.display())
      }
      _ => {
        (program,value.display())
      }
    }
  }
  pub fn raw_run(&mut self, input: String, program: Program) -> core::AST::Proventus {
    self.interpreter.evaluate(self.parser.parse(self.tokenizer.tokenize(input)), &mut self.env, program.clone())
  }
}

pub fn run(input: String, program: crate::Program) -> String {
  let tokenizer = core::tokenizer::Tokenizer {};
  let mut parser = core::parser::Parser {};
  let error_handler = core::error_handler::ErrorHandler {};
  let mut env = core::env::Environment{ error_handler, ..Default::default() };
  core::builtins::declare_builtins(&mut env);
  let mut interpreter = core::interpreter::Interpreter {error_handler, tokenizer, parser};


  let mut tokenized_input = tokenizer.tokenize(input);
  let mut parsed_input = parser.parse(tokenized_input);
  let mut interpreted_input = interpreter.evaluate(parsed_input, &mut env, program);

  interpreted_input.value.display()
}
