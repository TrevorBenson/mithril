use human_bytes::human_bytes;
use std::{
    fs::{create_dir_all, remove_dir},
    path::{Path, PathBuf},
};
use thiserror::Error;

use mithril_client::{common::CompressionAlgorithm, MithrilError, MithrilResult};

/// Check and unpack a downloaded archive in a given directory.
#[derive(Default)]
pub struct CardanoDbUnpacker;

/// Comment to test noop change against ci
/// Errors tied with the CardanoDbUnpacker.
#[derive(Debug, Error)]
pub enum CardanoDbUnpackerError {
    /// Not enough space on the disk. There should be at least the ratio given for the
    /// used algorithm (see [CompressionAlgorithm::free_space_snapshot_ratio]) times
    /// the size of the archive to download to ensure it could be unpacked safely.
    #[error("There is only {} remaining in directory '{}' to store and unpack a {} large archive.", human_bytes(*left_space), pathdir.display(), human_bytes(*archive_size))]
    NotEnoughSpace {
        /// Left space on device
        left_space: f64,

        /// Specified location
        pathdir: PathBuf,

        /// Packed cardano db size
        archive_size: f64,
    },

    /// The directory where the files from cardano db are expanded already exists.
    /// An error is raised because it lets the user a chance to preserve a
    /// previous work.
    #[error("Unpack directory '{0}' already exists, please move or delete it.")]
    UnpackDirectoryAlreadyExists(PathBuf),

    /// Cannot write in the given directory.
    #[error("Unpack directory '{0}' is not writable, please check own or parents' permissions and ownership.")]
    UnpackDirectoryIsNotWritable(PathBuf, #[source] MithrilError),
}

impl CardanoDbUnpacker {
    /// Check all prerequisites are met before starting to download and unpack
    /// big cardano db archive.
    pub fn check_prerequisites(
        pathdir: &Path,
        size: u64,
        compression_algorithm: CompressionAlgorithm,
    ) -> MithrilResult<()> {
        if pathdir.exists() {
            return Err(
                CardanoDbUnpackerError::UnpackDirectoryAlreadyExists(pathdir.to_owned()).into(),
            );
        }
        create_dir_all(pathdir).map_err(|e| {
            CardanoDbUnpackerError::UnpackDirectoryIsNotWritable(pathdir.to_owned(), e.into())
        })?;
        let free_space = fs2::available_space(pathdir)? as f64;
        // `remove_dir` doesn't remove intermediate directories that could have been created by `create_dir_all`
        remove_dir(pathdir)?;

        if free_space < compression_algorithm.free_space_snapshot_ratio() * size as f64 {
            return Err(CardanoDbUnpackerError::NotEnoughSpace {
                left_space: free_space,
                pathdir: pathdir.to_owned(),
                archive_size: size as f64,
            }
            .into());
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use mithril_common::test_utils::TempDir;

    fn create_temporary_empty_directory(name: &str) -> PathBuf {
        TempDir::create("client-cli", name)
    }

    #[test]
    fn should_return_ok() {
        let pathdir = create_temporary_empty_directory("return_ok").join("target_directory");

        CardanoDbUnpacker::check_prerequisites(&pathdir, 12, CompressionAlgorithm::default())
            .expect("check_prerequisites should not fail");
    }

    #[test]
    fn should_return_error_if_unpack_directory_already_exists() {
        let pathdir = create_temporary_empty_directory("existing_directory");

        let error =
            CardanoDbUnpacker::check_prerequisites(&pathdir, 12, CompressionAlgorithm::default())
                .expect_err("check_prerequisites should fail");

        assert!(
            matches!(
                error.downcast_ref::<CardanoDbUnpackerError>(),
                Some(CardanoDbUnpackerError::UnpackDirectoryAlreadyExists(_))
            ),
            "Unexpected error: {:?}",
            error
        );
    }

    // This test is not run on Windows because `set_readonly` is not working on Windows 7+
    // https://doc.rust-lang.org/std/fs/struct.Permissions.html#method.set_readonly
    #[cfg(not(target_os = "windows"))]
    #[test]
    fn should_return_error_if_directory_could_not_be_created() {
        let pathdir = create_temporary_empty_directory("read_only_directory");

        let mut perms = std::fs::metadata(&pathdir).unwrap().permissions();
        perms.set_readonly(true);
        std::fs::set_permissions(&pathdir, perms).unwrap();

        let targetdir = pathdir.join("target_directory");

        let error =
            CardanoDbUnpacker::check_prerequisites(&targetdir, 12, CompressionAlgorithm::default())
                .expect_err("check_prerequisites should fail");

        assert!(
            matches!(
                error.downcast_ref::<CardanoDbUnpackerError>(),
                Some(CardanoDbUnpackerError::UnpackDirectoryIsNotWritable(_, _))
            ),
            "Unexpected error: {:?}",
            error
        );
    }

    #[test]
    fn should_return_error_if_not_enough_available_space() {
        let pathdir =
            create_temporary_empty_directory("enough_available_space").join("target_directory");
        let archive_size = u64::MAX;

        let error = CardanoDbUnpacker::check_prerequisites(
            &pathdir,
            archive_size,
            CompressionAlgorithm::default(),
        )
        .expect_err("check_prerequisites should fail");

        assert!(
            matches!(
                error.downcast_ref::<CardanoDbUnpackerError>(),
                Some(CardanoDbUnpackerError::NotEnoughSpace {
                    left_space: _,
                    pathdir: _,
                    archive_size: _
                })
            ),
            "Unexpected error: {:?}",
            error
        );
    }
}
