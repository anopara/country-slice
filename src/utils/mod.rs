pub mod load_gltf;
pub mod load_json;

pub mod custom_macro {
    /*
    macro_rules! log_if_error {
        ($result: ident) => {
            match result {
                Ok(()) => (),
                Err(err_msg) => log::error!(err_msg),
            }
        };
    }
    */

    macro_rules! log_if_error {
        ($expression: expr) => {
            $expression.unwrap_or_else(|err| log::error!("{}", err))
        };
    }

    pub(crate) use log_if_error; // <-- the trick
}
