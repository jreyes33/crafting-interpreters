use crate::object::Object;
use crate::token::Token;
use crate::Result;
use paste::paste;
use std::rc::Rc;

pub type VisitorResult = Result<Rc<dyn Object>>;

macro_rules! ast {
    ($trait:ident, $($s:ident($($f:ident : $t:ty),*)),*$(,)*) => {
        pub trait $trait {
            fn accept(&self, visitor: &dyn Visitor<VisitorResult>) -> VisitorResult;
        }

        pub trait Visitor<O> {
            paste! {
                $(fn [<visit_ $s:lower _ $trait:lower>](&self, expr: &$s) -> O;)*
            }
        }

        $(
            pub struct $s {
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

            impl $trait for $s {
                fn accept(&self, visitor: &dyn Visitor<VisitorResult>) -> VisitorResult {
                    paste! {
                        visitor.[<visit_ $s:lower _ $trait:lower>](self)
                    }
                }
            }
        )*
    }
}

ast!(
    Expr,
    Binary(left: Box<dyn Expr>, operator: Token, right: Box<dyn Expr>),
    Grouping(expression: Box<dyn Expr>),
    Literal(value: Rc<dyn Object>),
    Unary(operator: Token, right: Box<dyn Expr>),
);
