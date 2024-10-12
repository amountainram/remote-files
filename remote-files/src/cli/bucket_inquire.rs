use anyhow::{Context as _, Result};
use inquire::{autocompletion::Replacement, Autocomplete, CustomUserError, Password, Select, Text};
use remote_files_configuration::{
    url_path::{UrlDirPath, UrlPath},
    Bucket, BucketVariant, GCSConfig, GcsStorageClass, Secret,
};
use rustyline::completion::{self, Pair};
use std::{path::PathBuf, sync::Arc};

#[derive(Clone)]
struct FilenameCompleter {
    completer: Arc<completion::FilenameCompleter>,
    suggestions: Vec<(String, String)>,
}

impl FilenameCompleter {
    pub fn new() -> Self {
        Self {
            completer: Arc::new(completion::FilenameCompleter::new()),
            suggestions: vec![],
        }
    }
}

impl Autocomplete for FilenameCompleter {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, CustomUserError> {
        let mut entries = vec![];

        if let Ok((_, candidates)) = self.completer.complete_path(input, input.len()) {
            self.suggestions.clear();
            candidates.into_iter().fold(
                (&mut self.suggestions, &mut entries),
                |(acc, entries),
                 Pair {
                     display,
                     replacement,
                 }| {
                    acc.push((display.clone(), replacement));
                    entries.push(display);
                    (acc, entries)
                },
            );
        }

        Ok(entries)
    }

    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<Replacement, CustomUserError> {
        if let Some(next) = highlighted_suggestion {
            if let Some((_, next)) = self.suggestions.iter().find(|(d, _)| d == &next) {
                Ok(Some(next.clone()))
            } else {
                Ok(Some(input.to_string()))
            }
        } else if let Some((_, next)) = self.suggestions.first() {
            Ok(Some(next.clone()))
        } else {
            Ok(None)
        }
    }
}

pub fn reject_empty_string(input: String) -> Option<String> {
    (!input.trim().is_empty()).then_some(input)
}

pub fn reject_empty_url(input: UrlPath) -> Option<UrlPath> {
    (!input.is_empty()).then_some(input)
}

pub fn reject_empty_url_dir(input: UrlDirPath) -> Option<UrlDirPath> {
    (!input.is_empty()).then_some(input)
}

pub fn bucket_inquire() -> Result<Bucket> {
    let variant = Select::new(
        "Which type of bucket do you want to track?",
        BucketVariant::variants(),
    )
    .prompt()?;

    match variant {
        BucketVariant::gcs => {
            let name = Text::new("Insert bucket name:")
                .prompt()
                .with_context(|| "retrieving gcs bucket name")?;
            let credential = Password::new("Insert gcs bucket credential (or skip in case you are planning to use an external 'application_default_credentials.json' file):").without_confirmation().prompt_skippable()
                                        .with_context(|| "retrieving gcs bucket credential")?.and_then(reject_empty_string).map(Secret::from);
            let credential_path = Text::new(
                "Insert gcs bucket credential path like 'application_default_credentials.json':",
            )
            .with_autocomplete(FilenameCompleter::new())
            .prompt_skippable()
            .with_context(|| "retrieving gcs bucket credential path")?
            .and_then(reject_empty_string)
            .map(PathBuf::from);
            let default_storage_class = Select::new(
                "Insert gcs bucket default storage class:",
                GcsStorageClass::variants(),
            )
            .prompt_skippable()
            .with_context(|| "retrieving gcs bucket default storage class")?;
            let endpoint = Text::new("Insert custom gcs entry endpoint:")
                .prompt_skippable()
                .with_context(|| "retrieving gcs bucket endpoint")?
                .map(|e| e.parse::<UrlPath>())
                .transpose()
                .with_context(|| "parsing endpoint as url")?
                .and_then(reject_empty_url);
            let prefix = Text::new("Insert custom gcs prefix as folder root base:")
                .prompt_skippable()
                .with_context(|| "retrieving gcs bucket prefix")?
                .map(|e| e.parse::<UrlDirPath>())
                .transpose()
                .with_context(|| "parsing endpoint as folder url")?
                .and_then(reject_empty_url_dir);
            let predefined_acl = Text::new("Insert custom gcs ACLs:")
                .prompt_skippable()
                .with_context(|| "retrieving gcs bucket ACLs")?
                .and_then(reject_empty_string);

            Ok(Bucket::Gcs(
                GCSConfig::builder()
                    .name(name)
                    .credential(credential)
                    .credential_path(credential_path)
                    .default_storage_class(default_storage_class)
                    .endpoint(endpoint)
                    .prefix(prefix)
                    .predefined_acl(predefined_acl)
                    .build(),
            ))
        }
        BucketVariant::s3 => todo!(),
    }
}
