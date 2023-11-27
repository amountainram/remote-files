use rf::configuration::Configuration;
use serde_json::{self, Value};
use std::{env, error::Error, fmt::Display, fs, io::BufReader};

#[derive(Debug)]
struct ExamplesError {
    idx: usize,
    error: serde_json::Error,
}

impl Display for ExamplesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "example {}: {}", self.idx, self.error)
    }
}

impl Error for ExamplesError {}

struct Schema(Value);

impl Schema {
    fn load_schema() -> Self {
        let working_dir = env::current_dir().unwrap();
        let schema_path = working_dir.join("schemas/configuration.schema.json");
        let file = fs::File::open(schema_path).unwrap();
        let buffer = BufReader::new(file);

        let value = serde_json::from_reader(buffer).unwrap();

        Self(value)
    }

    fn load_examples(self) -> Vec<Result<Configuration, serde_json::Error>> {
        self.0
            .as_object()
            .and_then(|obj| obj.get("examples"))
            .and_then(|examples| examples.as_array().cloned())
            .map(|examples| {
                examples
                    .into_iter()
                    .map(|example| serde_json::from_value(example))
                    .collect()
            })
            .expect("no example found")
    }
}

#[test]
fn examples_should_be_validated_by_schema() {
    let examples = Schema::load_schema().load_examples();

    for (idx, example) in examples.into_iter().enumerate() {
        example
            .map_err(|error| ExamplesError { idx, error })
            .unwrap();
    }
}
