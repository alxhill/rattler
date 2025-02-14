//! Unlinking packages from an environment.

use std::{
    collections::HashSet,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use indexmap::IndexSet;
use itertools::Itertools;
use rattler_conda_types::PrefixRecord;

/// Error that can occur while unlinking a package.
#[derive(Debug, thiserror::Error)]
pub enum UnlinkError {
    /// Failed to delete a directory.
    #[error("failed to delete empty directory: {0}")]
    FailedToDeleteDirectory(String, std::io::Error),

    /// Failed to delete a file.
    #[error("failed to delete file: {0}")]
    FailedToDeleteFile(String, std::io::Error),

    /// Failed to read a directory.
    #[error("failed to read directory: {0}")]
    FailedToReadDirectory(String, std::io::Error),
}

fn recursively_remove_empty_directories(
    directory_path: &Path,
    target_prefix: &Path,
    is_python_noarch: bool,
) -> Result<PathBuf, UnlinkError> {
    // Never delete the target prefix
    if directory_path == target_prefix || !directory_path.exists() {
        return Ok(directory_path.to_path_buf());
    }

    // Should we make this stronger to protect the user?
    assert!(directory_path.starts_with(target_prefix));

    let mut read_dir = directory_path.read_dir().map_err(|e| {
        UnlinkError::FailedToReadDirectory(directory_path.to_string_lossy().to_string(), e)
    })?;

    match read_dir.next().transpose() {
        Ok(None) => {
            // The directory is empty, delete it
            std::fs::remove_dir(directory_path).map_err(|e| {
                UnlinkError::FailedToDeleteDirectory(
                    directory_path.to_string_lossy().to_string(),
                    e,
                )
            })?;

            // Recursively remove the parent directory
            if let Some(parent) = directory_path.parent() {
                recursively_remove_empty_directories(parent, target_prefix, is_python_noarch)
            } else {
                Ok(directory_path.into())
            }
        }

        // Check if the only entry is a `__pycache__` directory
        Ok(Some(entry))
            if is_python_noarch
                && entry.file_name() == "__pycache__"
                && read_dir.next().is_none() =>
        {
            // The directory is empty, delete it
            std::fs::remove_dir_all(directory_path).map_err(|e| {
                UnlinkError::FailedToDeleteDirectory(
                    directory_path.to_string_lossy().to_string(),
                    e,
                )
            })?;

            // Recursively remove the parent directory
            if let Some(parent) = directory_path.parent() {
                recursively_remove_empty_directories(parent, target_prefix, is_python_noarch)
            } else {
                Ok(directory_path.into())
            }
        }
        _ => Ok(directory_path.into()),
    }
}

/// Completely remove the specified package from the environment.
pub async fn unlink_package(
    target_prefix: &Path,
    prefix_record: &PrefixRecord,
) -> Result<(), UnlinkError> {
    // Check if package is python noarch
    let is_python_noarch = prefix_record
        .repodata_record
        .package_record
        .noarch
        .is_python();

    let mut directories = HashSet::new();

    // Remove all entries
    for paths in prefix_record.paths_data.paths.iter() {
        match tokio::fs::remove_file(target_prefix.join(&paths.relative_path)).await {
            Ok(_) => {}
            Err(e) if e.kind() == ErrorKind::NotFound => {
                // Simply ignore if the file is already gone.
            }
            Err(e) => {
                return Err(UnlinkError::FailedToDeleteFile(
                    paths.relative_path.to_string_lossy().to_string(),
                    e,
                ))
            }
        }

        if let Some(parent) = paths.relative_path.parent() {
            directories.insert(parent.to_path_buf());
        }
    }

    // Sort the directories by length, so that we delete the deepest directories first.
    let mut directories: IndexSet<_> = directories.into_iter().sorted().collect();
    while let Some(directory) = directories.pop() {
        let directory_path = target_prefix.join(&directory);
        let removed_until =
            recursively_remove_empty_directories(&directory_path, target_prefix, is_python_noarch)?;

        // The directory is not empty which means our parent directory is also not empty,
        // recursively remove the parent directory from the set as well.
        while let Some(parent) = removed_until.parent() {
            if !directories.shift_remove(parent) {
                break;
            }
        }
    }

    // Remove the conda-meta file
    let conda_meta_path = target_prefix
        .join("conda-meta")
        .join(prefix_record.file_name());

    tokio::fs::remove_file(&conda_meta_path)
        .await
        .map_err(|e| {
            UnlinkError::FailedToDeleteFile(conda_meta_path.to_string_lossy().to_string(), e)
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, File},
        io::Write,
        path::Path,
        str::FromStr,
    };

    use rattler_conda_types::{Platform, PrefixRecord, Version};

    use crate::{
        get_repodata_record, get_test_data_dir,
        install::{link_package, unlink_package, InstallDriver, InstallOptions, PythonInfo},
    };

    async fn link_ruff(target_prefix: &Path, package: &str) -> PrefixRecord {
        let package_dir = tempfile::TempDir::new().unwrap();

        // Create package cache
        rattler_package_streaming::fs::extract(
            &get_test_data_dir().join(package),
            package_dir.path(),
        )
        .unwrap();

        let py_info =
            PythonInfo::from_version(&Version::from_str("3.10").unwrap(), Platform::Linux64)
                .unwrap();
        let install_options = InstallOptions {
            python_info: Some(py_info),
            ..InstallOptions::default()
        };

        let install_driver = InstallDriver::default();
        // Link the package
        let paths = link_package(
            package_dir.path(),
            target_prefix,
            &install_driver,
            install_options,
        )
        .await
        .unwrap();

        let repodata_record = get_repodata_record(package);
        // Construct a PrefixRecord for the package
        let prefix_record =
            PrefixRecord::from_repodata_record(repodata_record, None, None, paths, None, None);

        install_driver
            .post_process(&vec![prefix_record.clone()], target_prefix)
            .unwrap();

        return prefix_record;
    }

    #[tokio::test]
    async fn test_unlink_package() {
        let environment_dir = tempfile::TempDir::new().unwrap();
        let prefix_record =
            link_ruff(environment_dir.path(), "ruff-0.0.171-py310h298983d_0.conda").await;
        let conda_meta_path = environment_dir.path().join("conda-meta");
        std::fs::create_dir_all(&conda_meta_path).unwrap();

        // Write the conda-meta information
        let pkg_meta_path = conda_meta_path.join(prefix_record.file_name());
        prefix_record.write_to_path(&pkg_meta_path, true).unwrap();

        // Unlink the package
        unlink_package(environment_dir.path(), &prefix_record)
            .await
            .unwrap();

        // Check if the conda-meta file is gone
        assert!(!pkg_meta_path.exists());

        // check that the environment is completely empty except for the conda-meta folder
        let entries = std::fs::read_dir(environment_dir.path())
            .unwrap()
            .collect::<Vec<_>>();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].as_ref().unwrap().file_name(), "conda-meta");
    }

    #[tokio::test]
    async fn test_unlink_package_python_noarch() {
        let target_prefix = tempfile::TempDir::new().unwrap();
        let prefix_record = link_ruff(
            target_prefix.path(),
            "pytweening-1.0.4-pyhd8ed1ab_0.tar.bz2",
        )
        .await;

        let conda_meta_path = target_prefix.path().join("conda-meta");
        std::fs::create_dir_all(&conda_meta_path).unwrap();

        // Write the conda-meta information
        let pkg_meta_path = conda_meta_path.join(prefix_record.file_name());
        prefix_record.write_to_path(&pkg_meta_path, true).unwrap();

        fs::create_dir(
            target_prefix
                .path()
                .join("lib/python3.10/site-packages/pytweening/__pycache__"),
        )
        .unwrap();
        let mut file =
            File::create(target_prefix.path().join(
                "lib/python3.10/site-packages/pytweening/__pycache__/__init__.cpython-310.pyc",
            ))
            .unwrap();
        file.write_all("some funny bytes".as_bytes()).unwrap();
        file.sync_all().unwrap();

        // Unlink the package
        unlink_package(target_prefix.path(), &prefix_record)
            .await
            .unwrap();

        // Check if the conda-meta file is gone
        assert!(!pkg_meta_path.exists());

        // check that the environment is completely empty except for the conda-meta folder
        let entries = std::fs::read_dir(target_prefix.path())
            .unwrap()
            .collect::<Vec<_>>();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].as_ref().unwrap().file_name(), "conda-meta");
    }
}
