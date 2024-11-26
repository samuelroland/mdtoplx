#[derive(Default, Debug)]
pub struct CompilationStats {
    pub success: u16,
    pub failed: u16,
}

impl CompilationStats {
    pub fn print(&self) {
        println!(
            "Statistics: success compilation: {}, failed compilation: {}",
            self.success, self.failed
        )
    }
}
