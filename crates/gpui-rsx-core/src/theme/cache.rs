//! Loading `rsx.toml` from disk with a bounded cache keyed by path and content.
use super::*;

impl Theme {
    /// Loads `rsx.toml` from `dir`, caching by canonical path and content fingerprint.
    ///
    /// Returns `Ok(None)` when the file does not exist.
    pub fn load_from_dir(dir: &Path) -> Result<Option<Arc<Theme>>, String> {
        static CACHE: OnceLock<Mutex<ThemeCache>> = OnceLock::new();

        let requested_path = dir.join(CONFIG_FILE);
        // Different relative paths can point at the same workspace config. Canonicalizing
        // avoids retaining duplicate entries and makes the cache bound meaningful.
        let cache_path =
            std::fs::canonicalize(&requested_path).unwrap_or_else(|_| requested_path.clone());
        let source = match std::fs::read_to_string(&cache_path) {
            Ok(source) => source,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                return Err(format!(
                    "failed to read {}: {error}",
                    requested_path.display()
                ));
            }
        };
        let fingerprint = content_fingerprint(source.as_bytes());
        let cache = CACHE.get_or_init(Default::default);
        if let Ok(mut cache) = cache.lock()
            && let Some(theme) = cache.get(&cache_path, fingerprint)
        {
            return Ok(Some(theme));
        }

        let mut theme = Theme::parse(&source)?;
        theme.path = Some(cache_path.clone());
        let theme = Arc::new(theme);
        if let Ok(mut cache) = cache.lock() {
            cache.insert(cache_path, fingerprint, Arc::clone(&theme));
        }
        Ok(Some(theme))
    }
}

pub(super) const MAX_THEME_CACHE_ENTRIES: usize = 64;

#[derive(Default)]
pub(super) struct ThemeCache {
    pub(super) entries: HashMap<PathBuf, (u64, Arc<Theme>)>,
    order: VecDeque<PathBuf>,
}

impl ThemeCache {
    pub(super) fn get(&mut self, path: &Path, fingerprint: u64) -> Option<Arc<Theme>> {
        let (cached_fingerprint, theme) = self.entries.get(path)?;
        if *cached_fingerprint != fingerprint {
            return None;
        }
        let theme = Arc::clone(theme);
        self.touch(path);
        Some(theme)
    }

    pub(super) fn insert(&mut self, path: PathBuf, fingerprint: u64, theme: Arc<Theme>) {
        self.entries.insert(path.clone(), (fingerprint, theme));
        self.touch(&path);
        while self.entries.len() > MAX_THEME_CACHE_ENTRIES {
            let Some(oldest) = self.order.pop_front() else {
                break;
            };
            if oldest != path {
                self.entries.remove(&oldest);
            }
        }
    }

    pub(super) fn touch(&mut self, path: &Path) {
        self.order.retain(|cached| cached != path);
        self.order.push_back(path.to_owned());
    }
}

fn content_fingerprint(source: &[u8]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    source.hash(&mut hasher);
    hasher.finish()
}
