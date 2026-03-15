use salvo::Response;
use salvo::http::StatusCode;
use salvo::oapi::schema::{Array, BasicType, Object, OneOf, Schema};
use salvo::oapi::{Components, EndpointOutRegister, Operation, RefOr, ToSchema};
use salvo::writing::Scribe;
use serde::Serialize;

use crate::schema_ref_or;

/// A unified response type for Luminary API endpoints, consisting of [LuminarySuccessResponse] and [LuminaryFailResponse].
pub type LuminaryResponse<T> = Result<LuminarySuccessResponse<T>, LuminaryFailResponse>;

/// Registers the given schema as a 200 response, merging with any existing schema.
fn register_or_merge(operation: &mut Operation, schema: RefOr<Schema>) {
    if let Some(RefOr::Type(response)) = operation.responses.get_mut("200") {
        if let Some(content) = response.contents.get_mut("application/json") {
            let existing = std::mem::take(&mut content.schema);

            content.schema = match existing {
                RefOr::Type(Schema::OneOf(one_of)) => one_of.item(schema).into(),
                other => OneOf::new().item(other).item(schema).into(),
            };
        } else {
            response.contents.insert(
                "application/json".to_owned(),
                salvo::oapi::Content::new(OneOf::new().item(schema)),
            );
        }

        return;
    }

    operation.responses.insert(
        "200",
        salvo::oapi::Response::new("Response").add_content(
            "application/json",
            salvo::oapi::Content::new(OneOf::new().item(schema)),
        ),
    );
}

/// A successful response containing type `T`.
#[derive(Debug)]
pub struct LuminarySuccessResponse<T: Serialize> {
    data: T,
}

impl<T: Serialize + ToSchema + 'static> ToSchema for LuminarySuccessResponse<T> {
    fn to_schema(components: &mut Components) -> RefOr<Schema> {
        return schema_ref_or!(
            components,
            Schema::Object(Box::new(
                Object::new()
                    .property(
                        "success",
                        Object::with_type(BasicType::Boolean).enum_values([true]),
                    )
                    .required("success")
                    .property("data", T::to_schema(components))
                    .required("data"),
            ))
        );
    }
}

impl<T: Serialize> Serialize for LuminarySuccessResponse<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("LuminarySuccessResponse", 2)?;
        state.serialize_field("success", &true)?;
        state.serialize_field("data", &self.data)?;
        state.end()
    }
}

impl<T> From<T> for LuminarySuccessResponse<T>
where
    T: Serialize,
{
    fn from(value: T) -> Self {
        Self { data: value }
    }
}

impl<T: Serialize + Send> Scribe for LuminarySuccessResponse<T> {
    fn render(self, res: &mut Response) {
        res.status_code(StatusCode::OK);
        res.render(salvo::writing::Json(self));
    }
}

impl<T: Serialize + Send + ToSchema + 'static> EndpointOutRegister for LuminarySuccessResponse<T> {
    fn register(components: &mut Components, operation: &mut Operation) {
        let schema = Self::to_schema(components);
        register_or_merge(operation, schema);
    }
}

/// A failed response containing a vec of eyre context layers.
#[derive(Debug)]
pub struct LuminaryFailResponse {
    error: Vec<String>,
}

impl ToSchema for LuminaryFailResponse {
    fn to_schema(components: &mut Components) -> RefOr<Schema> {
        return schema_ref_or!(
            components,
            Schema::Object(Box::new(
                Object::new()
                    .property(
                        "success",
                        Object::with_type(BasicType::Boolean).enum_values([false]),
                    )
                    .required("success")
                    .property("error", Array::new().items(Object::with_type(BasicType::String)))
                    .required("error"),
            ))
        );
    }
}

impl Serialize for LuminaryFailResponse {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("LuminaryFailResponse", 2)?;
        state.serialize_field("success", &false)?;
        state.serialize_field("error", &self.error)?;
        state.end()
    }
}

impl From<eyre::Report> for LuminaryFailResponse {
    fn from(value: eyre::Report) -> Self {
        Self {
            error: value.chain().map(|e| e.to_string()).collect(),
        }
    }
}

impl Scribe for LuminaryFailResponse {
    fn render(self, res: &mut Response) {
        res.status_code(StatusCode::OK);
        res.render(salvo::writing::Json(self));
    }
}

impl EndpointOutRegister for LuminaryFailResponse {
    fn register(components: &mut Components, operation: &mut Operation) {
        let schema = Self::to_schema(components);
        register_or_merge(operation, schema);
    }
}
