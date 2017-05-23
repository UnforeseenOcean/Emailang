use user::UserPath;
use instruction::Instruction;
use interpreter::Interpreter;
use mail::Draft;
use environment::Environment;
use std::str::FromStr;
use regex;

#[derive(Clone, Debug)]
pub enum Type {
	Null,
	Text(String),
	UserPath(UserPath),
	Tuple(Vec<Type>),
	Expression(Box<Instruction>)
}

impl Type {
	pub fn get_num<T>(&self, inter: &mut Interpreter, from: &UserPath,
	                  env: &mut Environment) -> Option<T>
	where T: FromStr {
		match *self {
			Type::Text(ref s) => s.parse::<T>().ok(),
			Type::Expression(_) => self.resolve(inter, from, env).get_num(inter, from, env),
			_ => None
		}
	}

	fn get_modname(&self, inter: &mut Interpreter, from: &UserPath,
	               env: &mut Environment) -> String {
		match *self {
			Type::Text(ref s) => s.clone(),
			Type::Tuple(ref t) => t[0].get_string(inter, from, env).unwrap(),
			Type::Expression(_) => self.resolve(inter, from, env).get_modname(inter, from, env),
			_ => panic!()
		}
	}

	fn get_modargs(&self, inter: &mut Interpreter, from: &UserPath,
	               env: &mut Environment) -> Vec<Type> {
		match *self {
			Type::Text(_) => Vec::new(),
			Type::Tuple(ref t) => t[1..].to_vec(),
			Type::Expression(_) => self.resolve(inter, from, env).get_modargs(inter, from, env),
			_ => panic!()
		}
	}

	pub fn modify(&self, other: &Type, inter: &mut Interpreter, from: &UserPath,
	              env: &mut Environment) -> Option<Type> {
		let mod_name = self.get_modname(inter, from, env);
		let mod_args = self.get_modargs(inter, from, env);
		match mod_name.as_str() {
			"chars" => { // Turn a string into a tuple of characters
				assert!(mod_args.len() == 0);
				Some(Type::Tuple(other.get_string(inter, from, env).unwrap().chars()
					.map(|v|Type::Text(v.to_string())).collect()))
			},
			"merge" => {
				assert!(mod_args.len() == 0);
				Some(Type::Text(
					other.unpack(inter, from, env).iter()
					.map(|v|v.get_string(inter, from, env).unwrap())
					.collect::<String>()
				))
			},
			"filter" => {
				assert!(mod_args.len() == 1);
				let r = regex::Regex::new(&mod_args[0].get_string(inter, from, env).unwrap()).unwrap();
				Some(Type::Tuple(
					other.unpack(inter, from, env).iter()
					.map(|v|v.get_string(inter, from, env).unwrap())
					.filter(|v|r.is_match(&v))
					.map(|v|Type::Text(v))
					.collect()
				))
			},
			_ => None
		}
	}

	pub fn get_bool(&self, inter: &mut Interpreter, from: &UserPath,
	                env: &mut Environment) -> bool {
		match *self {
			Type::Null => false,
			Type::Text(ref s) => {
				!["false", "0", ""].contains(&s.to_lowercase().as_str())
			},
			Type::Tuple(ref t) => t.len() > 0,
			Type::Expression(_) => self.resolve(inter, from, env).get_bool(inter, from, env),
			_ => true
		}
	}

	pub fn resolve(&self, inter: &mut Interpreter, from: &UserPath, env: &mut Environment) -> Type {
		match *self {
			Type::Expression(ref exp) => {
				exp.call(inter, from, env).resolve(inter, from, env)
			},
			Type::Tuple(ref tuple) => {
				Type::Tuple(tuple.iter().map(|v|v.resolve(inter, from, env)).collect())
			},
			ref other => other.clone()
		}
	}

	pub fn len(&self, inter: &mut Interpreter, from: &UserPath,
	           env: &mut Environment) -> Option<usize> {
		match *self {
			Type::Tuple(ref vec) => Some(vec.len()),
			Type::Text(ref text) => Some(text.chars().count()),
			Type::Expression(_) => self.resolve(inter, from, env).len(inter, from, env),
			_ => None
		}
	}

	pub fn index(&self, pos: isize, inter: &mut Interpreter, from: &UserPath,
	             env: &mut Environment) -> Option<Type> {
		let selflen = self.len(inter, from, env).unwrap();
		let pos = if pos < 0 {
			((selflen as isize) + pos) as usize
		} else {
			pos as usize
		};
		match *self {
			Type::Tuple(ref vec) => Some(vec[pos].clone()),
			Type::Text(ref text) => Some(Type::Text(text.chars().nth(pos).unwrap().to_string())),
			Type::Expression(_) => self.resolve(inter, from, env)
			                           .index(pos as isize, inter, from, env),
			_ => None
		}
	}

	pub fn slice(&self, a: isize, b: isize, inter: &mut Interpreter, from: &UserPath,
	             env: &mut Environment) -> Option<Type> {
		let selflen = self.len(inter, from, env).unwrap();
		let a = if a < 0 {
			((selflen as isize) + a) as usize
		} else {
			a as usize
		};
		let b = if b < 0 {
			((selflen as isize) + b) as usize
		} else {
			b as usize
		};
		match *self {
			Type::Tuple(ref vec) => Some(Type::Tuple(vec[a..b].to_vec())),
			Type::Text(ref text) => {
				let chars = text.chars();
				Some(Type::Text(chars.skip(a).take(b-a).collect()))
			},
			Type::Expression(_) => self.resolve(inter, from, env)
				.slice(a as isize, b as isize, inter, from, env),
			_ => None
		}
	}

	pub fn is_null(&self) -> bool {
		if let Type::Null = *self {
			true
		} else {
			false
		}
	}

	pub fn get_typename(&self) -> &'static str {
		match *self {
			Type::Null => "null",
			Type::Text(_) => "text",
			Type::Tuple(_) => "tuple",
			Type::UserPath(_) => "user",
			Type::Expression(_) => "expression"
		}
	}

	pub fn get_string(&self, inter: &mut Interpreter, from: &UserPath,
	                  env: &mut Environment) -> Option<String> {
		match *self {
			Type::Text(ref val) => Some(val.clone()),
			Type::Expression(_) => self.resolve(inter, from, env).get_string(inter, from, env),
			Type::UserPath(ref path) => Some(format!("{}@{}", &path.0, &path.1)),
			_ => None
		}
	}

	pub fn get_tuple(&self, inter: &mut Interpreter, from: &UserPath,
	                 env: &mut Environment) -> Option<Vec<Type>> {
		match *self {
			Type::Tuple(ref v) => Some(v.clone()),
			Type::Expression(_) => self.resolve(inter, from, env).get_tuple(inter, from, env),
			_ => None
		}
	}

	pub fn unpack(&self, inter: &mut Interpreter, from: &UserPath,
	              env: &mut Environment) -> Vec<Type> {
		match self.get_tuple(inter, from, env) {
			Some(v) => v,
			None => vec![self.clone()]
		}
	}

	pub fn get_draft(&self, inter: &mut Interpreter, from: &UserPath,
	                 env: &mut Environment) -> Option<Draft> {
		match *self {
			Type::Tuple(ref t) => {
				Some(Draft {
					subject: t.get(0).map(
						|v|v.get_string(inter, from, env).unwrap_or("".to_string())
					).unwrap_or("".to_string()),
					message: t.get(1).map(
						|v|v.get_string(inter, from, env).unwrap_or("".to_string())
					).unwrap_or("".to_string()),
					attachments: (2..).take_while(|v|*v<t.len()).map(
						|v|t[v].get_string(inter, from, env).unwrap_or("".to_string())
					).collect()
				})
			},
			Type::Text(ref val) => {
				Some(Draft {
					subject: val.to_string(),
					message: "".to_string(),
					attachments: Vec::new()
				})
			},
			Type::Expression(_) => self.resolve(inter, from, env).get_draft(inter, from, env),
			_ => None
		}
	}

	pub fn get_user(&self, inter: &mut Interpreter, from: &UserPath,
	                env: &mut Environment) -> Option<UserPath> {
		match *self {
			Type::UserPath(ref val) => Some(val.clone()),
			Type::Expression(_) => self.resolve(inter, from, env).get_user(inter, from, env),
			_ => None
		}
	}
}
