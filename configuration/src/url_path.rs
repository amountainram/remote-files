use http::uri::{InvalidUri, PathAndQuery};
use paste::paste;
use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{de::Visitor, Deserialize, Serialize};
use std::{
    borrow::Cow,
    fmt::{self, Display},
    str::FromStr,
};

macro_rules! impl_serde_json_schema_as_string {
    ($ty:ty) => {
        impl Serialize for $ty {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(&self.to_string())
            }
        }

        impl<'de> Deserialize<'de> for $ty {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                paste! {
                    struct [<$ty Visitor>];

                    impl Visitor<'_> for [<$ty Visitor>] {
                        type Value = $ty;

                        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                            write!(formatter, stringify!($ty))
                        }

                        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                        where
                            E: serde::de::Error,
                        {
                            v.parse::<$ty>()
                                .map_err(|err| <E as serde::de::Error>::custom(err.to_string()))
                        }
                    }

                    deserializer.deserialize_str([<$ty Visitor>])
                }
            }
        }

        impl JsonSchema for $ty {
            fn schema_name() -> Cow<'static, str> {
                paste! {
                    stringify!([<$ty Visitor>]).into()
                }
            }

            fn json_schema(_: &mut SchemaGenerator) -> Schema {
                json_schema!({
                    "type": "string"
                })
            }
        }
    };
}

#[derive(Debug, Clone)]
pub struct UrlPath(Vec<String>);

impl_serde_json_schema_as_string!(UrlPath);

impl UrlPath {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns an absolute URL path starting with a `/`.
    /// It represents a remote file since it won't end with another `/`
    ///
    /// ```
    /// use remote_files::url_path::UrlPath;
    ///
    /// let path = "/hello/".parse::<UrlPath>().unwrap();
    /// assert_eq!(path.to_absolute_path().as_deref(), Some("/hello"));
    ///
    /// let path = "/".parse::<UrlPath>().unwrap();
    /// assert_eq!(path.to_absolute_path(), None);
    /// ```
    pub fn to_absolute_path(&self) -> Option<String> {
        let mut buf = self.to_absolute_dir_path();
        match buf.as_str() {
            "/" => None,
            _ => {
                buf.pop();
                Some(buf)
            }
        }
    }

    /// Returns an absolute URL path starting and ending with a `/`.
    ///
    /// ```
    /// use remote_files::url_path::UrlPath;
    ///
    /// let path = "hello".parse::<UrlPath>().unwrap();
    /// assert_eq!(path.to_absolute_dir_path().as_str(), "/hello/");
    /// ```
    pub fn to_absolute_dir_path(&self) -> String {
        let buf = String::from("/");
        self.0.iter().fold(buf, |mut buf, next| {
            buf.push_str(next);
            buf.push('/');
            buf
        })
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
                    .filter(|&s| !s.is_empty())
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
        ))
    }
}

impl Display for UrlPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            write!(f, "")
        } else {
            let abs_dir_path = self.to_absolute_dir_path();
            write!(f, "{}", &abs_dir_path[1..(abs_dir_path.len() - 1)])
        }
    }
}

#[derive(Debug, Clone)]
pub struct UrlDirPath(Vec<String>);

impl From<UrlPath> for UrlDirPath {
    fn from(UrlPath(segments): UrlPath) -> Self {
        UrlDirPath(segments)
    }
}

impl FromStr for UrlDirPath {
    type Err = <UrlPath as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url_path = s.parse::<UrlPath>()?;
        Ok(url_path.into())
    }
}

impl_serde_json_schema_as_string!(UrlDirPath);

impl Display for UrlDirPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let buf = self.0.iter().fold(String::from("/"), |mut buf, next| {
            buf.push_str(next);
            buf.push('/');
            buf
        });

        write!(f, "{buf}")
    }
}

#[cfg(test)]
mod tests {
    use super::{UrlDirPath, UrlPath};
    use rstest::rstest;

    #[rstest]
    #[case("", "")]
    #[case("hello", "hello")]
    #[case("hello/there", "hello/there")]
    #[case("/hello/there", "hello/there")]
    #[case("hello/there/", "hello/there")]
    #[case("hello/there?query", "hello/there")]
    #[case("hello/there/?query", "hello/there")]
    fn parse_str_to_url_path(#[case] input: &str, #[case] expect: &str) {
        assert_eq!(&input.parse::<UrlPath>().unwrap().to_string(), expect)
    }

    #[rstest]
    #[case("", "/")]
    #[case("hello", "/hello/")]
    #[case("hello/there", "/hello/there/")]
    #[case("/hello/there", "/hello/there/")]
    #[case("hello/there/", "/hello/there/")]
    #[case("hello/there?query", "/hello/there/")]
    #[case("hello/there/?query", "/hello/there/")]
    fn parse_str_to_url_dir_path(#[case] input: &str, #[case] expect: &str) {
        assert_eq!(&input.parse::<UrlDirPath>().unwrap().to_string(), expect)
    }
}
