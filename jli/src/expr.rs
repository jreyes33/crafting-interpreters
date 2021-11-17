use crate::object::Object;
use crate::token::Token;
use crate::Result;
use std::rc::Rc;

pub type VisitorResult = Result<Rc<dyn Object>>;

pub trait Visitor<O> {
    fn visit_binary_expr(&self, expr: &Binary) -> O;
    fn visit_grouping_expr(&self, expr: &Grouping) -> O;
    fn visit_literal_expr(&self, expr: &Literal) -> O;
    fn visit_unary_expr(&self, expr: &Unary) -> O;
}

macro_rules! expr {
    ($v:vis $s:ident($($f:ident : $t:ty),*)) => {
        $v struct $s {
            $(pub $f: $t,)*
        }

        impl $s {
            pub fn boxed($($f: $t,)*) -> Box<Self> {
                Box::new(Self::new($($f,)*))
            }

            fn new($($f: $t,)*) -> Self {
                Self { $($f,)* }
            }
        }

    }
}

pub trait Expr {
    fn accept(&self, visitor: &dyn Visitor<VisitorResult>) -> VisitorResult;
}

expr!(pub Binary(left: Box<dyn Expr>, operator: Token, right: Box<dyn Expr>));
expr!(pub Grouping(expression: Box<dyn Expr>));
expr!(pub Literal(value: Rc<dyn Object>));
expr!(pub Unary(operator: Token, right: Box<dyn Expr>));

// TODO: is it possible to generate these methods with the macro?
impl Expr for Binary {
    fn accept(&self, visitor: &dyn Visitor<VisitorResult>) -> VisitorResult {
        visitor.visit_binary_expr(self)
    }
}

impl Expr for Grouping {
    fn accept(&self, visitor: &dyn Visitor<VisitorResult>) -> VisitorResult {
        visitor.visit_grouping_expr(self)
    }
}

impl Expr for Literal {
    fn accept(&self, visitor: &dyn Visitor<VisitorResult>) -> VisitorResult {
        visitor.visit_literal_expr(self)
    }
}

impl Expr for Unary {
    fn accept(&self, visitor: &dyn Visitor<VisitorResult>) -> VisitorResult {
        visitor.visit_unary_expr(self)
    }
}
