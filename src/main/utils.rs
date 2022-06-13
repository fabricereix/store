use std::{fs, io, path::PathBuf};

pub fn dir_size(path: impl Into<PathBuf>) -> io::Result<u64> {
    fn dir_size(mut dir: fs::ReadDir) -> io::Result<u64> {
        dir.try_fold(0, |acc, file| {
            let file = file?;
            let size = match file.metadata()? {
                data if data.is_dir() => dir_size(fs::read_dir(file.path())?)?,
                data => data.len(),
            };
            Ok(acc + size)
        })
    }

    dir_size(fs::read_dir(path.into())?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_folder_size() {
        assert!(fs_extra::dir::get_size("tests/directory_including_broken_links").is_err());
        assert_eq!(
            dir_size("tests/directory_including_broken_links").unwrap(),
            20
        );
    }
}
