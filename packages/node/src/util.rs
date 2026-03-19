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
            .get::<crate::api::auth::LuminaryUser>("user")
            .ok()
            .and_then(|v| Some(v))
            .expect("User can not be obtained from a unprotected endpoint.")
    };
}

/// Helper macro to create or reference an OpenAPI schema.
#[macro_export]
macro_rules! schema_ref_or {
    ($components:expr, $block:expr) => {{
        let name = salvo::oapi::naming::assign_name::<Self>(Default::default());
        let ref_or = salvo::oapi::RefOr::Ref(salvo::oapi::Ref::new(format!("#/components/schemas/{}", name)));

        if !$components.schemas.contains_key(&name) {
            $components.schemas.insert(name.clone(), ref_or.clone());
            let schema = $block;
            $components.schemas.insert(name, schema)
        }

        ref_or
    }};
}

#[macro_export]
macro_rules! eyre_fmt {
    ($err:expr) => {
        $err.chain().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    };
}
