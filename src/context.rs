use std::{cell::RefCell, collections::HashMap, path::PathBuf};

use crate::{ast::{Ast, AstInfo}, directive::DirectiveValue, parser::TokenInfo};

#[derive(Debug)]
pub struct Context {
    pub target: Vec<u8>,
    pub tokens: RefCell<Vec<TokenInfo>>,
    pub asts: RefCell<Vec<AstInfo>>,
    pub references: RefCell<HashMap<String, Vec<DirectiveValue>>>,
    pub files: RefCell<Vec<PathBuf>>,
    pub work_directory: PathBuf
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

    pub fn add_file(&self, base_file_id: usize, file: String) -> PathBuf {
        let mut files = self.files.borrow_mut();
        
        let path = match files.get(base_file_id) {
            Some(path) => path.parent().map(|parent| parent.to_owned()),
            None => None
        };

        let full_file_path = match path {
            Some(path) => path.join(file),
            None => self.work_directory.join(file)
        };

        files.push(full_file_path.clone());
        full_file_path
    }

    pub fn last_file_id(&self) -> usize {
        self.files.borrow().len() - 1
    }

    pub fn last_path(&self) -> Option<String> {
        match self.files.borrow().last() {
            Some(path) => match path.parent() {
                Some(parent) => parent.as_os_str().to_str().map(|path| path.to_string()),
                None => None
            },
            None => self.work_directory.as_os_str().to_str().map(|path| path.to_string())
        }
    }

    pub fn get_path(&self, file_id: usize) -> Option<PathBuf> {
        match self.files.borrow().get(file_id) {
            Some(path) => path.parent().map(|parent| parent.to_owned()),
            None => None
        }
    }
}


impl Default for Context {
    fn default() -> Self {
        let work_directory = match std::env::current_dir() {
            Ok(path) => path,
            Err(error) => panic!("Could not find current directory. ({})", error)
        };

        Self {
            work_directory,
            target: Default::default(),
            tokens: Default::default(),
            asts: Default::default(),
            references: Default::default(),
            files: Default::default()
        }
    }
}
