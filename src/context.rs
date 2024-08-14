use std::cell::RefCell;

use crate::{ast::{Ast, AstInfo}, parser::TokenInfo};

#[derive(Debug)]
pub struct Context<'a> {
    pub source: &'a [u8],
    pub target: Vec<u8>,
    pub tokens: RefCell<Vec<TokenInfo<'a>>>,
    pub asts: RefCell<Vec<AstInfo<'a>>>,
}

impl<'a> Context<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            target: Vec::new(),
            asts: Default::default(),
            source: data,
            tokens: Default::default()
        }
    }
    
    pub fn add_ast(&self, token_index: usize, ast: Ast<'a>) {
        let token_info = &self.tokens.borrow()[token_index];

        let info = AstInfo {
            line: token_info.line,
            column: token_info.column,
            end: token_info.end,
            ast
        };

        self.asts.borrow_mut().push(info);
    }
}
