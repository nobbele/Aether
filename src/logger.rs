use std::{collections::HashMap, fs::File, io::BufWriter, marker::PhantomData};

use crate::{
    builders::LoggerBuilder,
    endpoint::{Endpoint, EndpointHash},
    EndpointSuper, LOGGER,
};

pub struct Logger {
    pub(crate) fmt: Box<dyn std::any::Any + std::marker::Send + std::marker::Sync>,
    pub(crate) endpoints: HashMap<EndpointHash, Endpoint>,
}

impl Logger {
    pub(crate) fn log(&mut self, target: EndpointHash, message: String) {
        let endpoint = self
            .endpoints
            .get_mut(&target)
            .expect("Attempted to use un-initialized endpoint");

        if endpoint.disabled {
            return;
        }

        if !endpoint.silent {
            println!("{}", message);
        }

        if let Some(file) = &mut endpoint.file {
            use std::io::Write;
            writeln!(file, "{}", message).unwrap();
        }
    }
}

/// Dropping this will also drop the current logger.
pub struct KeepAlive(pub(crate) PhantomData<()>);

impl Drop for KeepAlive {
    fn drop(&mut self) {
        let mut guard = LOGGER.lock().unwrap();
        let mut logger = std::mem::take(&mut *guard).unwrap();
        for endpoint in logger.endpoints.values_mut() {
            if let Some(file) = &mut endpoint.file {
                use std::io::Write;
                file.flush().unwrap();
            }
        }
    }
}

pub(crate) fn setup_logger<EP: EndpointSuper>(builder: LoggerBuilder<EP>) {
    if let Some(path) = &builder.base_path {
        std::fs::create_dir_all(path).unwrap();
    }

    let mut existing = Vec::new();

    for ep_builder in builder.endpoints.values() {
        if let Some(path) = &ep_builder.path {
            let path = builder.base_path.as_ref().unwrap().join(path);
            if path.exists() {
                existing.push(path);
            }
        }
    }

    if !existing.is_empty() {
        #[cfg(feature = "archive")]
        let mut archives = HashMap::new();

        for path in existing {
            #[cfg(feature = "archive")]
            {
                use std::io::{BufRead, BufReader};

                let mut file = BufReader::new(File::open(&path).unwrap());
                let mut header = String::new();
                let date = match file.read_line(&mut header) {
                        Ok(_) if header.trim_start().starts_with('%') && header.trim_end().ends_with('%') => {
                            chrono::DateTime::parse_from_rfc2822(header.trim().trim_start_matches('%').trim_end_matches('%').trim()).expect("Unable to parse `time` field in old log header, aborting since we can't safely archive the target file.")
                        },
                        e => panic!("Failed to read log header, aborting since we can't safely archive the target file. {:?}", e)
                    };

                let archive = archives.entry(date).or_insert_with(|| {
                    zip::ZipWriter::new(
                        File::create(
                            builder
                                .base_path
                                .as_ref()
                                .unwrap()
                                .join(format!("{}.zip", date)),
                        )
                        .unwrap(),
                    )
                });

                archive
                    .start_file(
                        path.file_name().unwrap().to_str().unwrap(),
                        zip::write::FileOptions::default(),
                    )
                    .unwrap();
                std::io::copy(&mut file, archive).unwrap();
            }

            #[cfg(not(feature = "archive"))]
            {
                let mut new_path = path.clone();
                while new_path.exists() {
                    new_path.set_extension(
                        new_path
                            .extension()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap()
                            .to_owned()
                            + ".bk",
                    );
                }

                std::fs::rename(path, new_path).unwrap();
            }
        }

        #[cfg(feature = "archive")]
        for (_, mut archive) in archives {
            archive.finish().unwrap();
        }
    }

    let creation_date = chrono::Utc::now().to_rfc2822();
    let mut endpoints = HashMap::new();
    for (hash, ep_builder) in builder.endpoints {
        let file = ep_builder.path.map(|path| {
            use std::io::Write;

            let mut file = BufWriter::new(
                File::create(builder.base_path.as_ref().unwrap().join(path))
                    .expect("Unable to create log output file."),
            );
            writeln!(&mut file, "% {creation_date} %").unwrap();
            file.flush().unwrap();
            file
        });

        endpoints.insert(
            hash,
            Endpoint {
                file,
                silent: ep_builder.silent,
                disabled: ep_builder.disabled,
            },
        );
    }

    let mut logger = LOGGER.lock().unwrap();
    if logger
        .replace(Logger {
            fmt: Box::new(builder.fmt),
            endpoints,
        })
        .is_some()
    {
        panic!("Logger was already initialized!");
    }
}
