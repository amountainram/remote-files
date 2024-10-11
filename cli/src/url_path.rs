use http::uri::{InvalidUri, PathAndQuery};
use std::str::FromStr;

#[derive(Clone)]
pub struct UrlPath(Vec<String>);

impl UrlPath {
    /// Returns an absolute URL path starting with a `/`.
    /// It represents a remote file since it won't end with another `/`
    ///
    /// ```
    /// let path = "/hello/".parse::<UrlPath>();
    /// assert_eq!(path.to_absolute_path().as_str(), "/hello");
    /// ```
    pub fn to_absolute_path(&self) -> String {
        let buf = String::from("/");
        self.0.iter().fold(buf, |mut buf, next| {
            buf.push_str(next);
            buf
        })
    }

    /// Returns an absolute URL path starting and ending with a `/`.
    ///
    /// ```
    /// let path = "hello".parse::<UrlPath>();
    /// assert_eq!(path.to_absolute_dir_path().as_str(), "/hello/");
    /// ```
    pub fn to_absolute_dir_path(&self) -> String {
        let mut path = self.to_absolute_path();
        path.push('/');
        path
    }
}

impl FromStr for UrlPath {
    type Err = InvalidUri;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let path_and_query = s.parse::<PathAndQuery>()?;
        let path = path_and_query.path().split('?').next();

        Ok(Self(
            path.map(|p| {
                p.split('/')
                    .filter(|&s| s.is_empty())
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
        ))
    }
}
