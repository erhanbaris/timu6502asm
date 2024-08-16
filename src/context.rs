use std::{cell::RefCell, collections::HashMap};

use crate::{ast::{Ast, AstInfo}, directive::DirectiveValue, parser::TokenInfo};

#[derive(Debug, Default)]
pub struct Context {
    pub target: Vec<u8>,
    pub tokens: RefCell<Vec<TokenInfo>>,
    pub asts: RefCell<Vec<AstInfo>>,
    pub references: RefCell<HashMap<String, Vec<DirectiveValue>>>,
    pub files: RefCell<Vec<String>>
}

impl Context {
    pub fn add_ast(&self, token_index: usize, ast: Ast) {
        let token_info = &self.tokens.borrow()[token_index];

        let info = AstInfo {
            line: token_info.line,
            column: token_info.column,
            end: token_info.end,
            ast
        };

        self.asts.borrow_mut().push(info);
    }

    pub fn add_file(&self, file: String) {
        self.files.borrow_mut().push(file);
    }

    pub fn last_file_id(&self) -> usize {
        self.files.borrow().len() - 1
    }
}
