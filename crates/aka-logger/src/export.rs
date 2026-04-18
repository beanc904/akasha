#[cfg(feature = "export")]
use crate::LogStore;

#[cfg(feature = "export")]
impl LogStore {
    pub fn export_buf_to_file<P>(&self, path: P)
    where
        P: AsRef<std::path::Path>,
    {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::options()
            .append(true)
            .create(true)
            .open(path)
            .unwrap();

        for line in self.buf_all() {
            writeln!(file, "{}", line).unwrap();
        }
    }
}
