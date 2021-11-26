use crate::expr::{self, Assign, Binary, Call, Expr, Grouping, Literal, Logical, Unary, Variable};
use crate::interpreter::Interpreter;
use crate::object::Nil;
use crate::stmt::{self, Block, Expression, Function, If, Print, Return, Stmt, Var, While};
use crate::token::Token;
use crate::Result;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

pub struct Resolver {
    interpreter: Rc<RefCell<Interpreter>>,
    scopes: Vec<HashMap<String, bool>>,
    current_function: FunctionType,
    nil: Rc<Nil>,
}

#[derive(Clone)]
enum FunctionType {
    None,
    Function,
}

impl Resolver {
    pub fn new(interpreter: Rc<RefCell<Interpreter>>) -> Self {
        Self {
            interpreter,
            scopes: vec![],
            current_function: FunctionType::None,
            nil: Rc::new(Nil),
        }
    }

    pub fn resolve<S>(&mut self, statements: &[S]) -> Result<()>
    where
        S: Deref<Target = dyn Stmt>,
    {
        for statement in statements {
            self.resolve_stmt(&**statement)?;
        }
        Ok(())
    }

    fn resolve_expr(&mut self, expr: &dyn Expr) -> Result<()> {
        expr.accept(self).map(|_| ())
    }

    fn resolve_stmt(&mut self, stmt: &dyn Stmt) -> Result<()> {
        stmt.accept(self)
    }

    fn resolve_function(&mut self, function: &Function, function_type: FunctionType) -> Result<()> {
        let enclosing_function = self.current_function.clone();
        self.current_function = function_type;
        self.begin_scope();
        for param in &function.params {
            self.declare(param)?;
            self.define(param);
        }
        self.resolve(function.body.as_slice())?;
        self.end_scope();
        self.current_function = enclosing_function;
        Ok(())
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) -> Result<()> {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(&name.lexeme) {
                return Err("Already a variable with this name in this scope.".into());
            }
            scope.insert(name.lexeme.clone(), false);
        }
        Ok(())
    }

    fn define(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexeme.clone(), true);
        }
    }

    fn resolve_local(&self, expr: &dyn Expr, name: &Token) {
        for (i, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter
                    .borrow_mut()
                    .resolve(expr, self.scopes.len() - 1 - i);
                return;
            }
        }
    }
}

impl expr::Visitor<expr::VisitorResult> for Resolver {
    fn visit_assign_expr(&mut self, expr: &Assign) -> expr::VisitorResult {
        self.resolve_expr(&*expr.value)?;
        self.resolve_local(expr, &expr.name);
        Ok(self.nil.clone())
    }

    fn visit_binary_expr(&mut self, expr: &Binary) -> expr::VisitorResult {
        self.resolve_expr(&*expr.left)?;
        self.resolve_expr(&*expr.right)?;
        Ok(self.nil.clone())
    }

    fn visit_call_expr(&mut self, expr: &Call) -> expr::VisitorResult {
        self.resolve_expr(&*expr.callee)?;
        for argument in &expr.arguments {
            self.resolve_expr(&**argument)?;
        }
        Ok(self.nil.clone())
    }

    fn visit_grouping_expr(&mut self, expr: &Grouping) -> expr::VisitorResult {
        self.resolve_expr(&*expr.expression)?;
        Ok(self.nil.clone())
    }

    fn visit_literal_expr(&mut self, _expr: &Literal) -> expr::VisitorResult {
        Ok(self.nil.clone())
    }

    fn visit_logical_expr(&mut self, expr: &Logical) -> expr::VisitorResult {
        self.resolve_expr(&*expr.left)?;
        self.resolve_expr(&*expr.right)?;
        Ok(self.nil.clone())
    }

    fn visit_unary_expr(&mut self, expr: &Unary) -> expr::VisitorResult {
        self.resolve_expr(&*expr.right)?;
        Ok(self.nil.clone())
    }

    fn visit_variable_expr(&mut self, expr: &Variable) -> expr::VisitorResult {
        if let Some(scope) = self.scopes.last() {
            if scope.get(&expr.name.lexeme) == Some(&false) {
                return Err("Can't read local variable in its own initializer.".into());
            }
        }
        self.resolve_local(expr, &expr.name);
        Ok(self.nil.clone())
    }
}

impl stmt::Visitor<stmt::VisitorResult> for Resolver {
    fn visit_block_stmt(&mut self, stmt: &Block) -> stmt::VisitorResult {
        self.begin_scope();
        self.resolve(&stmt.statements)?;
        self.end_scope();
        Ok(())
    }

    fn visit_expression_stmt(&mut self, stmt: &Expression) -> stmt::VisitorResult {
        self.resolve_expr(&*stmt.expression)
    }

    fn visit_function_stmt(&mut self, stmt: &Function) -> stmt::VisitorResult {
        self.declare(&stmt.name)?;
        self.define(&stmt.name);
        self.resolve_function(stmt, FunctionType::Function)
    }

    fn visit_if_stmt(&mut self, stmt: &If) -> stmt::VisitorResult {
        self.resolve_expr(&*stmt.condition)?;
        self.resolve_stmt(&*stmt.then_branch)?;
        if let Some(else_branch) = &stmt.else_branch {
            self.resolve_stmt(&**else_branch)?;
        }
        Ok(())
    }

    fn visit_print_stmt(&mut self, stmt: &Print) -> stmt::VisitorResult {
        self.resolve_expr(&*stmt.expression)
    }

    fn visit_return_stmt(&mut self, stmt: &Return) -> stmt::VisitorResult {
        if let FunctionType::None = self.current_function {
            return Err("Can't return from top-level code.".into());
        }
        if let Some(v) = &stmt.value {
            self.resolve_expr(&**v)?;
        }
        Ok(())
    }

    fn visit_var_stmt(&mut self, stmt: &Var) -> stmt::VisitorResult {
        self.declare(&stmt.name)?;
        if let Some(init) = &stmt.initializer {
            self.resolve_expr(&**init)?;
        }
        self.define(&stmt.name);
        Ok(())
    }

    fn visit_while_stmt(&mut self, stmt: &While) -> stmt::VisitorResult {
        self.resolve_expr(&*stmt.condition)?;
        self.resolve_stmt(&*stmt.body)
    }
}
