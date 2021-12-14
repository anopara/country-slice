pub mod load_gltf;
pub mod load_json;

pub mod custom_macro {

    macro_rules! log_if_error {
        ($expression: expr) => {
            $expression.unwrap_or_else(|err| log::error!("{}", err))
        };
    }

    pub(crate) use log_if_error;
}
