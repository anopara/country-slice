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

use std::process::Command;

pub fn validate_shaders(folder: &str) {
    let paths = std::fs::read_dir(folder).unwrap();

    for path in paths {
        let p = path.unwrap().path();
        let p = p.to_str().unwrap();

        let output = Command::new("glslangValidator.exe")
            .arg(p)
            .status()
            .expect("failed to execute process");

        if !output.success() {
            log::error!(
                "Shader has failed validation. See the error above. status: {}",
                output
            );
        }
    }
}
