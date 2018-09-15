macro_rules! from_error {
    ($type:ty, $target:ident, $targetvar:expr) => {
        impl From<$type> for $target {
            fn from(s: $type) -> Self {
                $targetvar(s.into())
            }
        }
    };
}

#[derive(Clone, Debug, Fail)]
pub enum NatsError {
    #[fail(display = "{}", _0)]
    GenericError(String),
}
