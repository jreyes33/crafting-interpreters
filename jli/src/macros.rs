#[macro_export]
macro_rules! ast {
    ($trait:ident -> $vr:ty [$($s:ident($($f:ident : $t:ty),*)),+$(,)*]) => {
        use paste::paste;

        pub trait $trait {
            fn accept(&self, visitor: &dyn Visitor<$vr>) -> $vr;
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
                fn accept(&self, visitor: &dyn Visitor<$vr>) -> $vr {
                    paste! {
                        visitor.[<visit_ $s:lower _ $trait:lower>](self)
                    }
                }
            }
        )*
    }
}
