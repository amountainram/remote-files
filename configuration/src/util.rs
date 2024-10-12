#[macro_export]
macro_rules! opendal_builder {
    ($builder:expr, $( $opt:expr => $method:ident ),* ) => {{
        let builder = $builder;
        $(
            let builder = if let Some(value) = $opt {
                builder.$method(value)
            } else {
                builder
            };
        )*
        builder
    }};
}

#[macro_export]
macro_rules! impl_serde_as_string {
    ($ty:ty) => {
        impl ::serde::Serialize for $ty {
            fn serialize<S>(&self, serializer: S) -> ::core::prelude::v1::Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                serializer.serialize_str(&self.to_string())
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $ty {
            fn deserialize<D>(deserializer: D) -> ::core::prelude::v1::Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                ::paste::paste! {
                    struct [<$ty Visitor>];

                    impl ::serde::de::Visitor<'_> for [<$ty Visitor>] {
                        type Value = $ty;

                        fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                            write!(formatter, stringify!($ty))
                        }

                        fn visit_str<E>(self, v: &str) -> ::core::prelude::v1::Result<Self::Value, E>
                        where
                            E: ::serde::de::Error,
                        {
                            v.parse::<$ty>()
                                .map_err(|err| <E as serde::de::Error>::custom(err.to_string()))
                        }
                    }

                    deserializer.deserialize_str([<$ty Visitor>])
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_json_schema_as_string {
    ($ty:ident) => {
        impl ::schemars::JsonSchema for $ty {
            fn schema_name() -> ::std::borrow::Cow<'static, str> {
                stringify!($ty).into()
            }

            fn json_schema(_: &mut ::schemars::SchemaGenerator) -> ::schemars::Schema {
                ::schemars::json_schema!({
                    "type": "string"
                })
            }
        }
    };
}

#[macro_export]
macro_rules! make_enum_with_variants {
    ($name:ident, $( $variant:ident ), +) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum $name {
            $(
                #[allow(non_camel_case_types)]
                $variant,
            )*
        }

        impl $name {
            pub fn variants() -> Vec<Self> {
                vec![$(Self::$variant),+]
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(
                    f,
                    "{}",
                    match self {
                        $(
                            Self::$variant => stringify!($variant),
                        )*
                    }
                )
            }
        }

        impl ::core::str::FromStr for $name {
            type Err = String;

            fn from_str(s: &str) -> ::core::prelude::v1::Result<Self, Self::Err> {
                match s {
                    $(
                        stringify!($variant) => Ok(Self::$variant),
                    )*
                    unknown => Err(format!("unknown variant {}", unknown))
                }
            }
        }

        impl_serde_as_string!($name);

        impl ::schemars::JsonSchema for $name {
            fn schema_name() -> ::std::borrow::Cow<'static, str> {
                stringify!($name).into()
            }

            fn always_inline_schema() -> bool {
                true
            }

            fn json_schema(_: &mut ::schemars::SchemaGenerator) -> ::schemars::Schema {
                ::schemars::json_schema!({
                    "type": "string",
                    "enum": vec![$(stringify!($variant)),+]
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use schemars::schema_for;

    #[test]
    fn test_macro_impl() {
        make_enum_with_variants!(MyEnum, V1, V2, V3);

        assert_eq!(MyEnum::variants(), vec![MyEnum::V1, MyEnum::V2, MyEnum::V3]);
        assert_eq!(
            MyEnum::variants()
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>(),
            vec!["V1".to_string(), "V2".to_string(), "V3".to_string()]
        );

        assert_eq!(
            serde_json::from_str::<MyEnum>(r#""V1""#).unwrap(),
            MyEnum::V1
        );

        assert_eq!(
            serde_json::to_string_pretty(&schema_for!(MyEnum)).unwrap(),
            indoc! {r#"
                {
                  "$schema": "https://json-schema.org/draft/2020-12/schema",
                  "title": "MyEnum",
                  "type": "string",
                  "enum": [
                    "V1",
                    "V2",
                    "V3"
                  ]
                }"#}
        );
    }
}
