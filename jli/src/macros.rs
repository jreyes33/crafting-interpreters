#[macro_export]
macro_rules! ast {
    ($trait:ident -> $vr:ty [$($s:ident($($f:ident : $t:ty),*)),+$(,)*]) => {
        use paste::paste;

        pub trait $trait: std::fmt::Debug + crate::object::AsAny {
            fn accept(&self, visitor: &mut dyn Visitor<$vr>) -> $vr;
        }

        pub trait Visitor<O> {
            paste! {
                $(fn [<visit_ $s:lower _ $trait:lower>](&mut self, expr: &$s) -> O;)*
            }
        }

        $(
            #[derive(Debug)]
            pub struct $s {
                $(pub $f: $t,)*
            }

            impl $s {
                pub fn boxed($($f: $t,)*) -> Box<Self> {
                    Box::new(Self::new($($f,)*))
                }

                pub fn new($($f: $t,)*) -> Self {
                    Self { $($f,)* }
                }
            }

            impl $trait for $s {
                fn accept(&self, visitor: &mut dyn Visitor<$vr>) -> $vr {
                    paste! {
                        visitor.[<visit_ $s:lower _ $trait:lower>](self)
                    }
                }
            }
        )*
    }
}
