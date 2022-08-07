use crate::define_spec;
use uuid::Uuid;

define_spec! {
    LoginSuccessSpec {
        uuid: Uuid,
        username: String,
    }, Debug
}
