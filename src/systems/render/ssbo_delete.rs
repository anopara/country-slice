use crate::render::ssbo::SSBO_TO_DELETE;

pub fn delete_dropped_ssbos() {
    let bla = std::mem::replace(&mut *SSBO_TO_DELETE.lock().unwrap(), Vec::new());
    for binding_point in bla {
        unsafe {
            gl::DeleteBuffers(1, &binding_point);
        }
    }
}
