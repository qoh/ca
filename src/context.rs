use std::collections::{HashMap, HashSet};

use super::parser::Expr;

pub struct Scope {
	vars: HashMap<String, Expr>
}

impl Scope {
	pub fn new() -> Self {
		Scope { vars: HashMap::new() }
	}

}

pub struct Context<'a> {
	scope: &'a mut Scope,
	evaluating: HashSet<String>
}

impl<'a> Context<'a> {
	pub fn new(scope: &'a mut Scope) -> Self {
		Context {
			scope,
			evaluating: HashSet::new()
		}
	}

	pub fn insert(&mut self, name: String, value: Expr) {
		self.scope.vars.insert(name, value);
	}

	pub fn get(&self, name: &String) -> Option<Expr> {
		if self.evaluating.contains(name) {
			None
		} else if let Some(r) = self.scope.vars.get(name) {
			Some((*r).clone())
		} else {
			None
		}
	}

	pub fn evaluate<'b>(&'b mut self, name: String) -> Context<'b> {
		let mut evaluating = self.evaluating.clone();
		evaluating.insert(name);

		Context {
			scope: self.scope,
			evaluating
		}
	}
}
