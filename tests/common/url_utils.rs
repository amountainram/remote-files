use urlencoding::encode;

pub struct Url {
    inner: url::Url,
}

impl Url {
    pub fn create_dir_url(segments: &[&str]) -> Self {
        // SAFETY: infallible
        let path = url::Url::try_from("file:///").unwrap();

        Self {
            inner: segments
                .iter()
                .map(|&segment| format!("{}/", encode(segment)))
                .fold(path, |acc, segment| acc.join(segment.as_str()).unwrap()),
        }
    }

    pub fn path(&self) -> &str {
        self.inner.path()
    }

    pub fn end_with_file(&self, filename: &str) -> Self {
        Self {
            inner: self.inner.join(filename).unwrap(),
        }
    }
}
