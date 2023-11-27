use std::{
    env,
    error::Error,
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use uuid::Uuid;

pub struct TmpDir(PathBuf);

impl TmpDir {
    pub fn create_tmp_dir() -> Self {
        let id = Uuid::new_v4().to_string();
        let tmp_dir = env::temp_dir().join(id.as_str());

        fs::create_dir(&tmp_dir).unwrap();

        TmpDir(tmp_dir)
    }

    pub fn add_file<P>(&self, path: P, content: &str) -> Result<PathBuf, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let dst = self.0.join(path);
        let mut fd = fs::OpenOptions::new().create(true).write(true).open(&dst)?;

        let bytes = content.as_bytes();

        fd.set_len(bytes.len() as u64)?;
        fd.write_all(content.as_bytes())?;
        fd.flush()?;

        Ok(dst)
    }
}

impl Drop for TmpDir {
    fn drop(&mut self) {
        fs::remove_dir_all(self.0.to_owned())
            .expect(format!("cannot cleanup temp dir '{}'", self.0.display()).as_str());
    }
}
