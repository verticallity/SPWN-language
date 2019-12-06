mod ast;
mod compiler;
mod native;
mod levelstring;

use std::{env, fs};
use pest::{Parser, iterators::Pair};
use pest_derive::Parser;

//use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "spwn.pest"]
pub struct SPWNParser;

fn main() {
    let args: Vec<String> = env::args().collect();
    let unparsed = fs::read_to_string(&args[1]).expect("Something went wrong reading the file");

    let parse_tree = SPWNParser::parse(Rule::spwn, &unparsed)
        .expect("unsuccessful parse").next().unwrap(); // get and unwrap the `spwn` rule; never fails

    //println!("{:?}\n\n", parse_tree.clone().into_inner());

    let statements = parse_statements(&mut parse_tree.into_inner());

    for statement in statements.iter() {
        println!("{:?}\n\n", statement);
    }

    let compiled = compiler::Compile(statements);

    let mut level_string = String::new();

    for trigger in compiled {
        level_string += &levelstring::serialize_trigger(trigger);
    }

    println!("{:?}", level_string);

}

fn parse_statements(statements: &mut pest::iterators::Pairs<Rule>) -> Vec<ast::Statement>{
    let mut stmts: Vec<ast::Statement> = vec![];

    for statement in statements {
        
        stmts.push(match statement.as_rule() {
            Rule::def => {
                let mut inner = statement.into_inner();
                ast::Statement::Definition(ast::Definition {
                    symbol: inner.next().unwrap().as_span().as_str().to_string(),
                    value:  parse_variable(inner.next().unwrap())
                })
            },
            Rule::event => {
                let mut info = statement.into_inner();
                ast::Statement::Event(ast::Event {
                    symbol:   info.next().unwrap().as_span().as_str().to_string(),
                    args:   info.next().unwrap().into_inner().map(|arg| parse_variable(arg)).collect(),
                    cmp_stmt: ast::CompoundStatement {
                        statements: parse_statements(&mut info.next().unwrap().into_inner())
                    }
                })
            },
            Rule::call => ast::Statement::Call(ast::Call {function: parse_variable(statement.into_inner().next().unwrap())}),
            Rule::native => {
                let mut info = statement.into_inner();
                ast::Statement::Native(ast::Native {
                    function:   parse_variable(info.next().unwrap()),
                    args:   info.next().unwrap().into_inner().map(|arg| parse_variable(arg)).collect()
                })
            },
            Rule::EOI => ast::Statement::EOI,
            _ => {
                println!("{:?} is not added to parse_statements yet", statement.as_rule());
                ast::Statement::EOI
            }
        })
    }
    stmts
}

fn parse_variable(pair: Pair<Rule>) -> ast::Variable {
    
    let mut call_list = pair.into_inner();
    let value = parse_value(call_list.next().unwrap().into_inner().next().unwrap());
    let symbols: Vec<String> = call_list.map(|x| x.as_span().as_str().to_string()).collect();
    
    fn parse_value(pair: Pair<Rule>) -> ast::ValueLiteral {
        match pair.as_rule() {
            Rule::id => {
                let number: u16;
                let mut scope = pair.into_inner();
                let mut unspecified = false;
                let first_value = scope.next().unwrap();
                let class_name: String;

                if first_value.as_rule() == Rule::number {
                    number = first_value.as_span().as_str().parse().unwrap();
                    class_name = scope.next().unwrap().as_span().as_str().to_string();
                } else {
                    unspecified = true;
                    number = 0;
                    class_name = first_value.as_span().as_str().to_string();
                }

                ast::ValueLiteral::ID(ast::ID {number, unspecified, class_name })
            },
            Rule::number => ast::ValueLiteral::Number(pair.as_span().as_str().parse().expect("invalid number")),

            Rule::bool => ast::ValueLiteral::Bool(pair.as_span().as_str() == "true"),

            Rule::cmp_stmt => ast::ValueLiteral::CmpStmt(ast::CompoundStatement {
                statements: parse_statements(&mut pair.into_inner())
            }),
            Rule::value_literal => parse_value(pair.into_inner().next().unwrap()),
            Rule::symbol => ast::ValueLiteral::Symbol(pair.as_span().as_str().to_string()),
            _ => {
                println!("{:?} is not added to parse_values yet", pair.as_rule());
                ast::ValueLiteral::Number(0.0)
            }
        }
    }

    ast::Variable{ value, symbols }
}



