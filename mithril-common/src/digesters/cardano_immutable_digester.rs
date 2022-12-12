use crate::{
    digesters::CardanoImmutableDigesterCacheProvider,
    digesters::{ImmutableDigester, ImmutableDigesterError, ImmutableFile},
    entities::{Beacon, HexEncodedDigest, ImmutableFileName},
};
use async_trait::async_trait;
use sha2::{Digest, Sha256};
use slog::{debug, info, Logger};
use std::{collections::BTreeMap, io, path::PathBuf, sync::Arc};

/// Result of a cache computation, contains the digest and the list of new entries to add
/// to the [CardanoImmutableDigesterCacheProvider].
type CacheComputationResult =
    Result<([u8; 32], Vec<(ImmutableFileName, HexEncodedDigest)>), io::Error>;

/// A digester working directly on a Cardano DB immutables files
pub struct CardanoImmutableDigester {
    /// A cardano node DB directory
    db_directory: PathBuf,

    /// A [CardanoImmutableDigesterCacheProvider] instance
    cache_provider: Arc<dyn CardanoImmutableDigesterCacheProvider>,

    /// The logger where the logs should be written
    logger: Logger,
}

impl CardanoImmutableDigester {
    /// ImmutableDigester factory
    pub fn new(
        db_directory: PathBuf,
        cache_provider: Arc<dyn CardanoImmutableDigesterCacheProvider>,
        logger: Logger,
    ) -> Self {
        Self {
            db_directory,
            cache_provider,
            logger,
        }
    }
}

#[async_trait]
impl ImmutableDigester for CardanoImmutableDigester {
    async fn compute_digest(&self, beacon: &Beacon) -> Result<String, ImmutableDigesterError> {
        let up_to_file_number = beacon.immutable_file_number;
        let immutables = ImmutableFile::list_completed_in_dir(&self.db_directory)?
            .into_iter()
            .filter(|f| f.number <= up_to_file_number)
            .collect::<Vec<_>>();

        match immutables.last() {
            None => Err(ImmutableDigesterError::NotEnoughImmutable {
                expected_number: up_to_file_number,
                found_number: None,
            }),
            Some(last_immutable_file) if last_immutable_file.number < up_to_file_number => {
                Err(ImmutableDigesterError::NotEnoughImmutable {
                    expected_number: up_to_file_number,
                    found_number: Some(last_immutable_file.number),
                })
            }
            Some(_) => {
                info!(self.logger, "#compute_digest"; "beacon" => #?beacon, "nb_of_immutables" => immutables.len());
                let cached_values = self.cache_provider.get(immutables).await;

                // digest is done in a separate thread because it is blocking the whole task
                let logger = self.logger.clone();
                let thread_beacon = beacon.clone();
                let (hash, new_cache_entries) =
                    tokio::task::spawn_blocking(move || -> CacheComputationResult {
                        compute_hash(logger, &thread_beacon, cached_values)
                    })
                    .await
                    .map_err(|e| ImmutableDigesterError::DigestComputationError(e.into()))??;
                let digest = hex::encode(hash);
                self.cache_provider.store(new_cache_entries).await;

                debug!(self.logger, "#computed digest: {:?}", digest);

                Ok(digest)
            }
        }
    }
}

fn compute_hash(
    logger: Logger,
    beacon: &Beacon,
    entries: BTreeMap<ImmutableFile, Option<HexEncodedDigest>>,
) -> CacheComputationResult {
    let mut hasher = Sha256::new();
    let mut new_cached_entries = Vec::new();
    let mut progress = Progress {
        index: 0,
        total: entries.len(),
    };

    hasher.update(beacon.compute_hash().as_bytes());

    for (ix, (entry, cache)) in entries.iter().enumerate() {
        match cache {
            None => {
                let data = hex::encode(entry.compute_raw_hash::<Sha256>()?);
                hasher.update(&data);
                new_cached_entries.push((entry.filename.clone(), data));
            }
            Some(digest) => {
                hasher.update(digest);
            }
        };

        if progress.report(ix) {
            info!(logger, "hashing: {}", &progress);
        }
    }

    Ok((hasher.finalize().into(), new_cached_entries))
}

struct Progress {
    index: usize,
    total: usize,
}

impl Progress {
    fn report(&mut self, ix: usize) -> bool {
        self.index = ix;
        (20 * ix) % self.total == 0
    }

    fn percent(&self) -> f64 {
        (self.index as f64 * 100.0 / self.total as f64).ceil()
    }
}

impl std::fmt::Display for Progress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}/{} ({}%)", self.index, self.total, self.percent())
    }
}

#[cfg(test)]
mod tests {
    use super::Progress;
    use crate::{
        digesters::{
            CardanoImmutableDigester, CardanoImmutableDigesterCacheProvider, ImmutableDigester,
            ImmutableDigesterError, ImmutableFile, MemoryCardanoImmutableDigesterCacheProvider,
        },
        entities::{Beacon, HexEncodedDigest, ImmutableFileNumber},
    };
    use sha2::{Digest, Sha256};
    use slog::Drain;
    use std::{
        collections::BTreeMap,
        fs,
        fs::File,
        io::prelude::*,
        path::{Path, PathBuf},
        sync::Arc,
    };
    use tokio::time::Instant;

    fn get_test_dir(subdir_name: &str) -> PathBuf {
        let parent_dir = std::env::temp_dir()
            .join("mithril_test")
            .join("cardano_immutable_digester")
            .join(subdir_name);

        if parent_dir.exists() {
            fs::remove_dir_all(&parent_dir)
                .unwrap_or_else(|e| panic!("Could not remove dir {:?}: {}", parent_dir, e));
        }
        fs::create_dir_all(&parent_dir)
            .unwrap_or_else(|e| panic!("Could not create dir {:?}: {}", parent_dir, e));

        parent_dir
    }

    fn write_immutable_trio(
        parent_dir: &Path,
        immutable: ImmutableFileNumber,
    ) -> Vec<ImmutableFile> {
        let mut result = vec![];
        for filename in [
            format!("{}.chunk", immutable),
            format!("{}.primary", immutable),
            format!("{}.secondary", immutable),
        ] {
            let file = write_dummy_file(parent_dir, &filename);
            result.push(ImmutableFile {
                number: immutable.to_owned(),
                path: file,
                filename: filename.to_string(),
            });
        }
        result
    }

    /// Create a chunk/primary/secondary trio for each given [ImmutableFileNumber].
    ///
    /// If `append_uncompleted_trio` is true it will add another trio of file, that won't be included
    /// in the returned vec, to simulate the last 3 'uncompleted / wip' files that can be found in
    /// a cardano immutable db.
    fn create_fake_immutables(
        parent_dir: &Path,
        immutable_numbers: &[ImmutableFileNumber],
        append_uncompleted_trio: bool,
    ) -> Vec<ImmutableFile> {
        if immutable_numbers.is_empty() {
            panic!("At least one immutable numbers must be given");
        }

        let mut immutable_numbers = immutable_numbers.to_vec();
        immutable_numbers.sort();

        if append_uncompleted_trio {
            write_immutable_trio(parent_dir, immutable_numbers.last().unwrap() + 1);
        }

        immutable_numbers
            .into_iter()
            .flat_map(|ifn| write_immutable_trio(parent_dir, ifn))
            .collect::<Vec<_>>()
    }

    /// Create a file with the given name in the given dir, write some text to it, and then
    /// return its path.
    fn write_dummy_file(parent_dir: &Path, filename: &str) -> PathBuf {
        let file = parent_dir.join(Path::new(filename));
        let mut source_file = File::create(&file).unwrap();
        write!(source_file, "This is a test file named '{}'", filename).unwrap();
        file
    }

    fn create_logger() -> slog::Logger {
        let decorator = slog_term::PlainDecorator::new(slog_term::TestStdoutWriter);
        let drain = slog_term::CompactFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();
        slog::Logger::root(Arc::new(drain), slog::o!())
    }

    fn compute_expected_hash(beacon: &Beacon, entries: &[ImmutableFile]) -> HexEncodedDigest {
        let mut hasher = Sha256::new();
        hasher.update(beacon.compute_hash().as_bytes());

        for entry in entries {
            let data = hex::encode(entry.compute_raw_hash::<Sha256>().unwrap());
            hasher.update(&data);
        }

        hex::encode(hasher.finalize())
    }

    #[test]
    fn reports_progress_every_5_percent() {
        let mut progress = Progress {
            index: 0,
            total: 7000,
        };

        assert!(!progress.report(1));
        assert!(!progress.report(4));
        assert!(progress.report(350));
        assert!(!progress.report(351));
    }

    #[test]
    fn reports_progress_when_total_lower_than_20() {
        let mut progress = Progress {
            index: 0,
            total: 16,
        };

        assert!(progress.report(4));
        assert!(progress.report(12));
        assert!(!progress.report(3));
        assert!(!progress.report(15));
    }

    #[tokio::test]
    async fn fail_if_no_file_in_folder() {
        let dir = get_test_dir("fail_if_no_file_in_folder/immutable");
        let cache = MemoryCardanoImmutableDigesterCacheProvider::default();
        let digester = CardanoImmutableDigester::new(dir, Arc::new(cache), create_logger());
        let beacon = Beacon::new("devnet".to_string(), 1, 1);

        let result = digester
            .compute_digest(&beacon)
            .await
            .expect_err("compute_digest should've failed");

        assert_eq!(
            format!(
                "{:?}",
                ImmutableDigesterError::NotEnoughImmutable {
                    expected_number: beacon.immutable_file_number,
                    found_number: None,
                }
            ),
            format!("{:?}", result)
        );
    }

    #[tokio::test]
    async fn fail_if_a_invalid_file_is_in_immutable_folder() {
        let dir = get_test_dir("fail_if_no_immutable_exist/immutable");
        write_dummy_file(&dir, "not_immutable");
        let cache = MemoryCardanoImmutableDigesterCacheProvider::default();
        let digester = CardanoImmutableDigester::new(dir, Arc::new(cache), create_logger());
        let beacon = Beacon::new("devnet".to_string(), 1, 1);

        assert!(digester.compute_digest(&beacon).await.is_err());
    }

    #[tokio::test]
    async fn fail_if_theres_only_the_uncompleted_immutable_trio() {
        let dir = get_test_dir("fail_if_theres_only_the_uncompleted_immutable_trio/immutable");
        write_immutable_trio(&dir, 1);
        let cache = MemoryCardanoImmutableDigesterCacheProvider::default();
        let digester = CardanoImmutableDigester::new(dir, Arc::new(cache), create_logger());
        let beacon = Beacon::new("devnet".to_string(), 1, 1);

        let result = digester
            .compute_digest(&beacon)
            .await
            .expect_err("compute_digest should've failed");

        assert_eq!(
            format!(
                "{:?}",
                ImmutableDigesterError::NotEnoughImmutable {
                    expected_number: beacon.immutable_file_number,
                    found_number: None,
                }
            ),
            format!("{:?}", result)
        );
    }

    #[tokio::test]
    async fn fail_if_less_immutable_than_what_required_in_beacon() {
        let dir = get_test_dir("fail_if_less_immutable_than_what_required_in_beacon/immutable");
        create_fake_immutables(&dir, &[1, 2, 3, 4, 5], true);
        let cache = MemoryCardanoImmutableDigesterCacheProvider::default();
        let digester = CardanoImmutableDigester::new(dir, Arc::new(cache), create_logger());
        let beacon = Beacon::new("devnet".to_string(), 1, 10);

        let result = digester
            .compute_digest(&beacon)
            .await
            .expect_err("compute_digest should've failed");

        assert_eq!(
            format!(
                "{:?}",
                ImmutableDigesterError::NotEnoughImmutable {
                    expected_number: beacon.immutable_file_number,
                    found_number: Some(5),
                }
            ),
            format!("{:?}", result)
        );
    }

    #[tokio::test]
    async fn can_compute_hash_of_a_hundred_immutable_file_trio() {
        let dir = get_test_dir("can_compute_hash_of_a_hundred_immutable_file_trio/immutable");
        let immutables =
            create_fake_immutables(&dir, &(1..=100).collect::<Vec<ImmutableFileNumber>>(), true);
        let cache = MemoryCardanoImmutableDigesterCacheProvider::default();
        let logger = create_logger();
        let digester = CardanoImmutableDigester::new(dir, Arc::new(cache), logger.clone());
        let beacon = Beacon::new("devnet".to_string(), 1, 100);

        let expected = compute_expected_hash(&beacon, &immutables);
        let result = digester
            .compute_digest(&beacon)
            .await
            .expect("compute_digest must not fail");

        assert_eq!(expected, result)
    }

    #[tokio::test]
    async fn digests_are_stored_into_cache_provider() {
        let dir = get_test_dir("digests_are_stored_into_cache_provider/immutable");
        let immutables = create_fake_immutables(&dir, &[1, 2], true);
        let cache = Arc::new(MemoryCardanoImmutableDigesterCacheProvider::default());
        let logger = create_logger();
        let digester = CardanoImmutableDigester::new(dir, cache.clone(), logger.clone());
        let beacon = Beacon::new("devnet".to_string(), 1, 2);

        digester
            .compute_digest(&beacon)
            .await
            .expect("compute_digest must not fail");

        let cached_entries = cache.get(immutables.clone()).await;
        let expected: BTreeMap<_, _> = immutables
            .into_iter()
            .map(|i| {
                let digest = hex::encode(i.compute_raw_hash::<Sha256>().unwrap());
                (i, Some(digest))
            })
            .collect();

        assert_eq!(expected, cached_entries);
    }

    #[tokio::test]
    async fn computed_digest_with_or_without_cache_are_equals() {
        let dir = get_test_dir("computed_digest_with_or_without_cache_are_equals/immutable");
        create_fake_immutables(&dir, &[1, 2, 3], true);
        let cache = MemoryCardanoImmutableDigesterCacheProvider::default();
        let logger = create_logger();
        let digester = CardanoImmutableDigester::new(dir, Arc::new(cache), logger.clone());
        let beacon = Beacon::new("devnet".to_string(), 1, 3);

        let without_cache_digest = digester
            .compute_digest(&beacon)
            .await
            .expect("compute_digest must not fail");

        let with_cache_digest = digester
            .compute_digest(&beacon)
            .await
            .expect("compute_digest must not fail");

        assert_eq!(
            without_cache_digest, with_cache_digest,
            "Digests with or without cache should be the same"
        );
    }

    #[tokio::test]
    async fn hash_computation_is_quicker_with_a_full_cache() {
        let dir = get_test_dir("hash_computation_is_quicker_with_a_full_cache/immutable");
        create_fake_immutables(&dir, &(1..=300).collect::<Vec<ImmutableFileNumber>>(), true);
        let cache = MemoryCardanoImmutableDigesterCacheProvider::default();
        let logger = create_logger();
        let digester = CardanoImmutableDigester::new(dir, Arc::new(cache), logger.clone());
        let beacon = Beacon::new("devnet".to_string(), 1, 300);

        let now = Instant::now();
        digester
            .compute_digest(&beacon)
            .await
            .expect("compute_digest must not fail");
        let elapsed_without_cache = now.elapsed();

        let now = Instant::now();
        digester
            .compute_digest(&beacon)
            .await
            .expect("compute_digest must not fail");
        let elapsed_with_cache = now.elapsed();

        assert!(
            elapsed_with_cache < (elapsed_without_cache * 2 / 3),
            "digest computation with full cache should be at least 33% faster than without cache,\
            time elapsed: with cache {:?}, without cache {:?}",
            elapsed_with_cache,
            elapsed_without_cache
        );
    }
}
