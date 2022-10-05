use uuid::Uuid;

use crate::define_spec;

define_spec! {
    LoginSuccessSpec {
        uuid: Uuid,
        username: String,
    }, Debug
}
