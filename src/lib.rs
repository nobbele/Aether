use builders::LoggerBuilder;
use std::{
    collections::HashMap, fs::File, hash::Hash, io::BufWriter, marker::PhantomData, sync::Mutex,
};

pub mod builders;

lazy_static::lazy_static! {
    static ref ENDPOINT_TYPE: Mutex<Option<std::any::TypeId>> = Mutex::new(None);
    static ref LOGGER: Mutex<Option<Logger>> = Mutex::new(None);
}

pub trait EndpointSuper: std::any::Any + std::fmt::Debug + std::hash::Hash {}
impl<T: std::any::Any + std::fmt::Debug + std::hash::Hash> EndpointSuper for T {}

#[macro_export]
macro_rules! log {
    ($target:expr, $($arg:tt)*) => {
        aether::impl_log($target, format!($($arg)*))
    };
}

#[doc(hidden)]
pub fn impl_log<EP: EndpointSuper + 'static>(endpoint: EP, message: String) {
    let mut guard = LOGGER.lock().unwrap();
    let logger = guard.as_mut().expect("Uninitialized logger. Did you forget to call `aether::init` or did you drop the `KeepAlive` object early?");
    let fmt = logger
        .fmt
        .downcast_ref::<fn(LogEntry<EP>) -> String>()
        .unwrap();
    let output = fmt(LogEntry {
        time: chrono::Utc::now(),
        endpoint: &endpoint,
        text: &message,
    });

    logger.log(hash(&endpoint), output);
}

pub struct LogEntry<'a, EP> {
    pub time: chrono::DateTime<chrono::Utc>,
    pub endpoint: &'a EP,
    pub text: &'a str,
}

#[derive(Hash, PartialEq, Eq)]
struct EndpointHash(u64);

fn hash(obj: &impl EndpointSuper) -> EndpointHash {
    use std::hash::Hasher;

    assert_eq!(ENDPOINT_TYPE.lock().unwrap().unwrap(), obj.type_id());

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    obj.hash(&mut hasher);
    EndpointHash(hasher.finish())
}

#[derive(Default)]
struct Endpoint {
    file: Option<BufWriter<File>>,
    silent: bool,
}

struct Logger {
    fmt: Box<dyn std::any::Any + std::marker::Send + std::marker::Sync>,
    endpoints: HashMap<EndpointHash, Endpoint>,
}

impl Logger {
    fn log(&mut self, target: EndpointHash, message: String) {
        let endpoint = self
            .endpoints
            .get_mut(&target)
            .expect("Attempted to use un-initialized endpoint");

        if !endpoint.silent {
            println!("{}", message);
        }

        if let Some(file) = &mut endpoint.file {
            use std::io::Write;
            writeln!(file, "{}", message).unwrap();
        }
    }
}

pub struct KeepAlive(PhantomData<()>);

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

fn setup_logger<EP: EndpointSuper>(builder: LoggerBuilder<EP>) {
    if let Some(path) = &builder.base_path {
        std::fs::create_dir_all(path).unwrap();
    }

    let mut existing = Vec::new();

    for (_hash, ep_builder) in &builder.endpoints {
        if let Some(path) = &ep_builder.path {
            let path = builder.base_path.as_ref().unwrap().join(path);
            if path.exists() {
                existing.push(path);
            }
        }
    }

    if existing.len() > 0 {
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
            },
        );
    }

    let mut logger = LOGGER.lock().unwrap();
    if let Some(_) = logger.replace(Logger {
        fmt: Box::new(builder.fmt),
        endpoints,
    }) {
        panic!("Logger was already initialized!");
    }
}

pub fn init<EP: EndpointSuper>() -> LoggerBuilder<EP> {
    let mut guard = ENDPOINT_TYPE.lock().unwrap();
    if let Some(_) = guard.replace(std::any::TypeId::of::<EP>()) {
        panic!("Logger has already been (at least partially) initialized!");
    }

    LoggerBuilder {
        base_path: None,
        fmt: |log| {
            format!(
                "{} [{:?}] {}",
                log.time.format("%T.%3f"),
                log.endpoint,
                log.text
            )
        },
        endpoints: HashMap::new(),
    }
}
