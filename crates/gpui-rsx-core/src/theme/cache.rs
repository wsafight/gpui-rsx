//! Loading `rsx.toml` from disk with a bounded cache keyed by path and content.
use super::*;
use std::io::Read;

#[cfg(unix)]
type FileIdentity = (u64, u64);
#[cfg(not(unix))]
type FileIdentity = Option<(u64, std::time::SystemTime)>;

fn file_identity(metadata: &std::fs::Metadata) -> FileIdentity {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        (metadata.dev(), metadata.ino())
    }
    #[cfg(not(unix))]
    {
        metadata
            .modified()
            .ok()
            .map(|mtime| (metadata.len(), mtime))
    }
}

impl Theme {
    /// Loads `rsx.toml` from `dir`, caching by canonical path and content fingerprint.
    ///
    /// Returns `Ok(None)` when the file does not exist.
    pub fn load_from_dir(dir: &Path) -> Result<Option<Arc<Theme>>, String> {
        static CACHE: OnceLock<Mutex<ThemeCache>> = OnceLock::new();

        let requested_path = dir.join(CONFIG_FILE);
        let mut file = match std::fs::File::open(&requested_path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                return Err(format!(
                    "failed to read {}: {error}",
                    requested_path.display()
                ));
            }
        };
        let metadata = file.metadata().map_err(|error| {
            format!(
                "failed to read metadata for {}: {error}",
                requested_path.display()
            )
        })?;
        let identity = file_identity(&metadata);
        let mut source = String::with_capacity(metadata.len() as usize);
        file.read_to_string(&mut source)
            .map_err(|error| format!("failed to read {}: {error}", requested_path.display()))?;
        let fingerprint = content_fingerprint(source.as_bytes());
        let cache = CACHE.get_or_init(Default::default);

        if let Ok(mut guard) = cache.lock()
            && let Some(canonical) = guard.resolve_cached_path(&requested_path, identity)
            && let Some(theme) = guard.get(&canonical, fingerprint)
        {
            return Ok(Some(theme));
        }

        let cache_path =
            std::fs::canonicalize(&requested_path).unwrap_or_else(|_| requested_path.clone());
        if let Ok(mut guard) = cache.lock() {
            guard.remember_path_alias(requested_path, identity, cache_path.clone());
            if let Some(theme) = guard.get(&cache_path, fingerprint) {
                return Ok(Some(theme));
            }
        }

        let mut theme = Theme::parse(&source)?;
        theme.path = Some(cache_path.clone());
        let theme = Arc::new(theme);
        if let Ok(mut guard) = cache.lock() {
            guard.insert(cache_path, fingerprint, Arc::clone(&theme));
        }
        Ok(Some(theme))
    }
}

pub(super) const MAX_THEME_CACHE_ENTRIES: usize = 64;

#[derive(Default)]
pub(super) struct ThemeCache {
    pub(super) entries: HashMap<PathBuf, (u64, Arc<Theme>)>,
    aliases: HashMap<PathBuf, (FileIdentity, PathBuf)>,
    order: VecDeque<PathBuf>,
}

impl ThemeCache {
    fn resolve_cached_path(&self, requested: &Path, identity: FileIdentity) -> Option<PathBuf> {
        let (cached_id, canonical) = self.aliases.get(requested)?;
        (*cached_id == identity).then(|| canonical.clone())
    }

    fn remember_path_alias(
        &mut self,
        requested: PathBuf,
        identity: FileIdentity,
        canonical: PathBuf,
    ) {
        if self.aliases.len() >= MAX_THEME_CACHE_ENTRIES * 2
            && !self.aliases.contains_key(&requested)
        {
            self.aliases.clear();
        }
        self.aliases.insert(requested, (identity, canonical));
    }

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
        if self.order.back().is_some_and(|recent| recent == path) {
            return;
        }
        self.order.retain(|cached| cached != path);
        self.order.push_back(path.to_owned());
    }
}

fn content_fingerprint(source: &[u8]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    source.hash(&mut hasher);
    hasher.finish()
}
