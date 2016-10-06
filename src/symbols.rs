use user::UserPath;
use types::Type;
use error;
use parser;
use parser::symbolparser;
use error::{SyntaxError, SyntaxErrorType};

#[derive(Clone, Debug)]
pub struct Block(pub Vec<SymbolDef>);

impl Block {
	pub fn is_comma_delimited(&self) -> bool {
		for val in &self.0 {
			match val.symbol {
				Symbol::Comma => return true,
				_ => {}
			}
		}
		return false;
	}

	pub fn split_commas(&self) -> Vec<Vec<SymbolDef>> {
		let mut ret = vec![];
		let mut vec = vec![];
		for val in &self.0 {
			match val.symbol {
				Symbol::Comma => {
					ret.push(vec);
					vec = vec![];
				},
				_ => {
					vec.push(val.clone())
				}
			}
		}
		ret.push(vec);
		ret
	}
}

/* Symbols:
 * !        - define
 * <a@b>    - user
 * "a"      - text
 * a        - identifier
 * {a;b;c;} - curlybraced
 * (a,b,c)  - parenthesis
 * >        - arrow
 * ,        - comma
 * ;        - semicolon
**/

#[derive(Clone, Debug)]
pub enum Symbol {
	// Structures
	CurlyBraced(Block),
	Parenthesis(Block),
	// Types
	UserPath(UserPath),
	Identifier(String),
	Text(String),
	// Syntax
	Comma,
	Semicolon,
	Index(Block),
	Slice(Option<Block>, Option<Block>),
	If,
	Else,
	ElseIf,
	// Operators
	Define,
	Arrow,
	Addition,
	Receive,
	Assign,
	Modifier,
}

#[derive(Clone, Debug)]
pub struct SymbolDef {
	pub errfactory: error::SyntaxErrorFactory,
	pub symbol: Symbol
}

#[derive(Clone, Copy)]
pub enum OperatorType {
	LeftToRight(usize),
	RightToLeft(usize),
	Neither
}

impl OperatorType {
	pub fn compare(&self, other:usize) -> bool {
		match *self {
			OperatorType::LeftToRight(val) => val > other,
			OperatorType::RightToLeft(val) => val >= other,
			OperatorType::Neither => false
		}
	}
	pub fn get(&self) -> usize {
		match *self {
			OperatorType::LeftToRight(val) => val,
			OperatorType::RightToLeft(val) => val,
			OperatorType::Neither => 0
		}
	}
	pub fn is_op(&self) -> bool {
		if let OperatorType::Neither = *self {
			false
		} else {
			true
		}
	}
}

impl SymbolDef {
	pub fn get_type(&self) -> Result<Type, SyntaxError> {
		match self.symbol {
			Symbol::Text(ref val) => Ok(Type::Text(val.clone())),
			Symbol::Identifier(ref val) => Ok(Type::Text(val.clone())),
			Symbol::UserPath(ref val) => Ok(Type::UserPath(val.clone())),
			Symbol::Parenthesis(ref val) => {
				if val.is_comma_delimited() {
					let mut tuple = Vec::new();
					for v in val.split_commas() {
						if parser::symbolparser::is_expression(&v) {
							tuple.push(Type::Expression(Box::new(try!(symbolparser::parse_expression(&v)))));
						} else {
							for symdef in v {
								tuple.push(try!(symdef.get_type()));
							}
						}
					}
					Ok(Type::Tuple(tuple))
				} else {
					if symbolparser::is_expression(&val.0) {
						Ok(Type::Expression(Box::new(try!(symbolparser::parse_expression(&val.0)))))
					} else {
						assert!(val.0.len() == 1);
						Ok(try!(val.0[0].get_type()))
					}
				}
			},
			_ => Err(self.errfactory.gen_error(SyntaxErrorType::NotAType))
		}
	}

	pub fn get_operator(&self) -> OperatorType {
		match self.symbol {
			// Separators
			Symbol::Comma      => OperatorType::LeftToRight(2000),
			// Operators
			Symbol::Assign     => OperatorType::RightToLeft(1002),
			Symbol::Arrow      => OperatorType::LeftToRight(1001),
			Symbol::Addition   => OperatorType::LeftToRight(1000),
			// Modifier operators
			Symbol::Modifier   => OperatorType::RightToLeft(3),
			Symbol::Slice(_,_) => OperatorType::RightToLeft(2),
			Symbol::Index(_)   => OperatorType::RightToLeft(2),
			Symbol::Receive    => OperatorType::LeftToRight(1),
			_ => OperatorType::Neither
		}
	}
}
