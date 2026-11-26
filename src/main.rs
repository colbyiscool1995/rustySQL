use std::{io::{self, Write}, process};

mod tokenize;
mod parser;

use crate::{MetaCommandResult::{MetaCommandSuccess, MetaCommandUnrecognizedCommand}, PrepareResult::{PrepareSuccess, PrepareSyntaxError, PrepareUnrecognizedState}};



enum MetaCommandResult  {
    MetaCommandSuccess,
    MetaCommandUnrecognizedCommand,
}

enum PrepareResult {
    PrepareSuccess,
    PrepareUnrecognizedState,
    PrepareSyntaxError,
    
}

enum StatementType {
    StatementInsert,
    StatementSelect,
    None,
}



struct Statement {
    statement_t: StatementType,
}
struct InputBuffer {
    buffer: String,
}



impl Statement {
    fn prepare_statement(&mut self,input_buffer: &InputBuffer) -> PrepareResult{
        
        //tokenize Statement
        let mut lexer = tokenize::Lexer::new(&input_buffer.buffer);
        let tokens = lexer.tokenize().expect("Lexer failed");

        //check if syntax is correct and can be parsed
        let mut parser = parser::Parser::new(tokens);
        match parser.parse_statements() {
            Ok(ast) => {
                println!("{:#?}", ast);
                return PrepareSuccess
            },
            Err(err) => { 
                eprintln!("Syntax Error: {}", err);
                return PrepareSyntaxError
            },
            _ => return PrepareUnrecognizedState
            
        }

    }

    fn execute_statement(&mut self){
        match self.statement_t {
            StatementType::StatementInsert => {
                println!("This is where we would do an insert.")
            }
            StatementType::StatementSelect => {
                println!("This is where we would do an select")
            }
            StatementType::None => { eprintln!("No Statement found") }
        }
    }

    
}

impl InputBuffer {

    fn new() -> Self{
       Self {
        buffer: String::new()
       }
    }

    fn read_input(&mut self) {

        self.buffer.clear();

        let bytes_read = io::stdin()
        .read_line(&mut self.buffer)
        .unwrap_or_else(|err| {
            println!("Error reading input: {}", err);
            process::exit(1);
        });

        if bytes_read == 0 {
            eprintln!("Error reading input");
            process::exit(1);
        }

        let trimmed_len = self.buffer.trim_end_matches(&['\r', '\n'][..]).len();
        self.buffer.truncate(trimmed_len);
     }

    fn close_input_buffer(&mut self){
        self.buffer.clear();
     }

    fn do_meta_command(&mut self) -> MetaCommandResult{
        if self.buffer == ".exit" {
            self.close_input_buffer();
            process::exit(1);
        }
        else if self.buffer == ".test" {
            println!("This is a test.");
            return MetaCommandSuccess;
        }
        else{
            return MetaCommandUnrecognizedCommand
        }
    }
    
}


fn print_prompt(){ 
    print!("db > ");
    io::stdout().flush();
}

fn main() {
    let mut input_buffer = InputBuffer::new();
    loop{
        print_prompt();
        input_buffer.read_input();

        let buffer_cp = input_buffer.buffer.clone();
        let first_char = buffer_cp.chars().next().unwrap();

        if first_char == '.' {
            match input_buffer.do_meta_command() {
                MetaCommandSuccess => continue,
                MetaCommandUnrecognizedCommand => {
                    println!("Unrecognized Command {}", input_buffer.buffer);
                    continue;
                },
            }
        }

        let mut statement = Statement {
            statement_t : StatementType::None,
        };

        match statement.prepare_statement(&input_buffer) {
            PrepareResult::PrepareSuccess => {
                
                
            },
            PrepareResult::PrepareUnrecognizedState => {
                eprintln!("Unrecognized keyword at start of {}", input_buffer.buffer);
                continue;
            },
            PrepareResult::PrepareSyntaxError => {

            }
            
        }

        statement.execute_statement();
        println!("Executed.")

        
    }
}