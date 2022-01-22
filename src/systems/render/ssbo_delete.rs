use crate::render::ssbo::SSBO_TO_DELETE;

pub fn delete_dropped_ssbos() {
    for binding_point in SSBO_TO_DELETE.lock().unwrap().drain(..) {
        unsafe {
            gl::DeleteBuffers(1, &binding_point);
        }
    }
}
