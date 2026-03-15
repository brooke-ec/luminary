use salvo::Response;
use salvo::http::StatusCode;
use salvo::oapi::Ref;
use salvo::oapi::naming::assign_name;
use salvo::oapi::schema::{Array, BasicType, Object, OneOf, Schema};
use salvo::oapi::{Components, EndpointOutRegister, Operation, RefOr, ToSchema};
use salvo::writing::Scribe;
use serde::Serialize;

pub type LuminaryResponse<T> = Result<LuminarySuccessResponse<T>, LuminaryFailResponse>;

#[derive(Debug)]
pub struct LuminarySuccessResponse<T: Serialize> {
    data: T,
}

impl<T: Serialize + ToSchema + 'static> ToSchema for LuminarySuccessResponse<T> {
    fn to_schema(components: &mut Components) -> RefOr<Schema> {
        let name = assign_name::<Self>(Default::default());
        let ref_or = RefOr::Ref(Ref::new(format!("#/components/schemas/{}", name)));

        if !components.schemas.contains_key(&name) {
            components.schemas.insert(name.clone(), ref_or.clone());

            let schema = Schema::Object(Box::new(
                Object::new()
                    .property(
                        "success",
                        Object::with_type(BasicType::Boolean).enum_values([true]),
                    )
                    .required("success")
                    .property("data", T::to_schema(components))
                    .required("data"),
            ));
            components.schemas.insert(name, schema);
        }

        ref_or
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
        operation.responses.insert(
            "200",
            salvo::oapi::Response::new("Success response")
                .add_content("application/json", salvo::oapi::Content::new(schema)),
        );
    }
}

#[derive(Debug)]
pub struct LuminaryFailResponse {
    error: Vec<String>,
}

impl ToSchema for LuminaryFailResponse {
    fn to_schema(components: &mut Components) -> RefOr<Schema> {
        let name = assign_name::<Self>(Default::default());
        let ref_or = RefOr::Ref(Ref::new(format!("#/components/schemas/{}", name)));

        if !components.schemas.contains_key(&name) {
            components.schemas.insert(name.clone(), ref_or.clone());

            let schema = Schema::Object(Box::new(
                Object::new()
                    .property(
                        "success",
                        Object::with_type(BasicType::Boolean).enum_values([false]),
                    )
                    .required("success")
                    .property("error", Array::new().items(Object::with_type(BasicType::String)))
                    .required("error"),
            ));
            components.schemas.insert(name, schema);
        }

        ref_or
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
        let fail_schema = Self::to_schema(components);

        if let Some(RefOr::Type(existing)) = operation.responses.get_mut("200") {
            if let Some(content) = existing.contents.get_mut("application/json") {
                let success_schema = std::mem::replace(&mut content.schema, RefOr::Type(Schema::default()));
                content.schema = OneOf::new().item(success_schema).item(fail_schema).into();
            }
        } else {
            operation.responses.insert(
                "200",
                salvo::oapi::Response::new("Response")
                    .add_content("application/json", salvo::oapi::Content::new(fail_schema)),
            );
        }
    }
}
