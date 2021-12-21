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
    // check if validator exists
    if !std::path::Path::new("glslangValidator.exe").exists() {
        log::error!("glslangValidator.exe not found");
        return;
    }

    if let Ok(paths) = std::fs::read_dir(folder) {
        for path in paths {
            let p = path.unwrap().path();
            let p = p.to_str().unwrap();

            if let Ok(output) = Command::new("glslangValidator").arg(p).status() {
                if !output.success() {
                    log::error!(
                        "Shader has failed validation. See the error above. status: {}",
                        output
                    );
                }
            } else {
                log::error!("failed to execute process");
                return;
            }
        }
        log::info!("Shader validation complete");
    } else {
        log::error!("The given path doesn't exist {}", folder);
    }
}
