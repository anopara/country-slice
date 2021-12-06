use hotwatch::Event;
use hotwatch::Hotwatch;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;

use super::shader::ShaderProgram;

pub struct ShaderWatch {
    hotwatch: Hotwatch,
    pub event_shader_changed: Arc<Mutex<HashSet<String>>>,
}

impl ShaderWatch {
    pub fn new() -> Self {
        Self {
            hotwatch: Hotwatch::new_with_custom_delay(std::time::Duration::from_millis(100))
                .expect("hotwatch failed to initialize"),
            event_shader_changed: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn watch(&mut self, shader_program: &ShaderProgram) {
        for path in shader_program.src_paths() {
            self.watch_path(&path);
        }
    }

    fn watch_path(&mut self, path: &String) {
        let realname = self.event_shader_changed.clone();
        let path_string = path.clone();

        self.hotwatch
            .watch(path, move |event| {
                if let Event::Write(_pathbuf) = event {
                    //println!("{:?} has changed", path_string.clone());
                    let mut vec = realname.lock().unwrap();
                    vec.insert(path_string.clone());
                }
            })
            .expect("failed to watch file!");
    }
}
