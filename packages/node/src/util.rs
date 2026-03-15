/// A simple macro to obtain a type from the depot. Causing a panic if the type is not present.
#[macro_export]
macro_rules! obtain {
    ($depot:expr, $type:ty) => {
        $depot.obtain::<$type>().expect(concat!(
            concat!("Tried to obtain an instance of ", stringify!($type)),
            " which the depot didn't have"
        ))
    };
}

/// A simple macro to obtain the currently authenticated user from the depot.
#[macro_export]
macro_rules! get_user {
    ($depot:expr) => {
        $depot
            .get::<crate::auth::LuminaryUser>("user")
            .ok()
            .and_then(|v| Some(v))
            .expect("User can not be obtained from a unprotected endpoint.")
    };
}
