use super::{Command, ExtractCommand, Package};
use std::env;
use std::fs;
use std::fs::{DirEntry, File};
use std::io;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::process::Stdio;

#[derive(Clone, Debug, PartialEq)]
pub struct Installer {
    pub package_id: String,
    pub package_dir: PathBuf,
    pub installer_dir: PathBuf,
    pub download_dir: PathBuf,
    pub extract_dir: PathBuf,
    pub state: InstallerState,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InstallerState {
    pub download_file: Option<PathBuf>,
    pub current_dir: PathBuf,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InstallerError {
    pub package_id: String,
    pub message: String,
}

impl Installer {
    // create folder
    pub fn init(
        store_packages_dir: &Path,
        installer_dir: &Path,
        package: &Package,
    ) -> Result<Installer, String> {
        let package_id = package.id.clone();

        if !store_packages_dir.exists() && fs::create_dir_all(store_packages_dir).is_err() {
            return Err(format!(
                "Directory {} can not be created",
                store_packages_dir.display()
            ));
        }

        let package_dir = store_packages_dir
            .canonicalize()
            .unwrap()
            .join(package.id.clone());

        // create working directory "silently" if needed
        let download_dir = installer_dir.join(package.id.clone()).join("download");
        if !download_dir.exists() && fs::create_dir_all(download_dir.clone()).is_err() {
            return Err(format!(
                "Directory {} can not be created",
                download_dir.display()
            ));
        }
        let download_dir = download_dir.canonicalize().unwrap();

        let extract_dir = installer_dir.join(package.id.clone()).join("extract");
        if !extract_dir.exists() && fs::create_dir_all(&extract_dir).is_err() {
            return Err(format!(
                "Directory {} can not be created",
                extract_dir.display()
            ));
        }
        let extract_dir = extract_dir.canonicalize().unwrap();
        let installer_dir = installer_dir
            .canonicalize()
            .unwrap()
            .join(package_id.clone());

        let state = InstallerState {
            download_file: None,
            current_dir: extract_dir.clone(),
        };
        Ok(Installer {
            download_dir,
            extract_dir,
            installer_dir,
            package_dir,
            package_id,
            state,
        })
    }

    pub fn is_installed(&self) -> bool {
        self.package_dir.exists()
    }

    pub fn create_directory(&self) -> Result<String, String> {
        if fs::create_dir_all(self.package_dir.clone()).is_err() {
            Err(format!(
                "Directory {} can not be created",
                self.package_dir.display()
            ))
        } else {
            Ok(format!(
                "Directory {} has been created",
                self.package_dir.display()
            ))
        }
    }

    pub fn delete_directory(&self) -> String {
        //eprintln!(">>>delete directory {}", self.package_dir.display());
        if self.package_dir.exists() {
            fs::remove_dir_all(self.package_dir.display().to_string()).expect("directory deleted");
            format!("Directory {} has been deleted", self.package_dir.display())
        } else {
            // should not have been called
            format!("Directory {} does not exist", self.package_dir.display())
        }
    }

    pub fn exec_command(&mut self, command: &Command, verbose: bool) -> Result<String, String> {
        match command {
            Command::Download(url) => {
                let basename = Path::new(&url).file_name().unwrap().to_str().unwrap();
                let download_file = self.download_dir.join(basename);

                //eprintln!(">> Downloading {} to {}", url, download_file.display());
                if download_file.exists() {
                    self.state.download_file = Some(download_file.clone());
                    return Ok(format!(
                        "File {} already downloaded",
                        download_file.display()
                    ));
                }

                match reqwest::blocking::get(url) {
                    Ok(mut response) => {
                        if response.status() == 200 {
                            let mut dest = File::create(download_file.clone()).expect("");
                            // let bytes = &mut response.bytes().unwrap();
                            // dest.write(bytes).unwrap();

                            // let mut stream = response.bytes_stream();
                            // while let Some(item) = stream.next() {
                            //     let chunk = match item {
                            //         Ok(item) => item,
                            //         Err(e) => return Err(format!("Error while downloading to file - {}", e.to_string()))
                            //     };
                            //     if let Err(e) = dest.write_all(&chunk) {
                            //         return Err(format!("Error while writing to file - {}", e.to_string()))
                            //     }
                            // }
                            //let mut resp = reqwest::blocking::get("http://httpbin.org/range/5")?;
                            //let mut buf: Vec<u8> = vec![];
                            if let Err(e) = response.copy_to(&mut dest) {
                                return Err(format!("Error downloading - {}", e));
                            }

                            let message =
                                format!("File {} has been written", download_file.display());
                            self.state.download_file = Some(download_file);
                            Ok(message)
                        } else {
                            Err(format!(
                                "Url <{}> can not be downloaded: status {}",
                                url,
                                response.status()
                            ))
                        }
                    }
                    Err(e) => Err(e.to_string()),
                }
            }
            Command::Extract(extract_command) => {
                let download_file = if let Some(f) = self.state.download_file.clone() {
                    f
                } else {
                    return Err("Download file has not been set".to_string());
                };
                match extract_command {
                    ExtractCommand::TarGz => {
                        //eprintln!(">>> extracting {}", download_file.display());
                        let tar_gz = File::open(download_file.clone()).unwrap();
                        let tar = flate2::read::GzDecoder::new(tar_gz);
                        if verbose {
                            eprintln!("{} has been uncompressed", download_file.display());
                        }
                        let mut archive = tar::Archive::new(tar);
                        if let Err(e) = archive.unpack(self.extract_dir.clone()) {
                            return Err(format!(
                                "can not extract {} - {}",
                                download_file.display(),
                                e
                            ));
                        };
                    }
                    ExtractCommand::TarXz => {
                        let mut tar_file = download_file.clone();
                        tar_file.set_extension("");
                        uncompress_xz(&download_file, &tar_file)?;

                        let tar_file = File::open(tar_file).unwrap();
                        let mut archive = tar::Archive::new(tar_file);
                        archive.unpack(self.extract_dir.clone()).unwrap();
                    }
                    ExtractCommand::TarBz2 => {
                        let mut tar_file = download_file.clone();
                        tar_file.set_extension("");
                        uncompress_bz2(&download_file, &tar_file)?;
                        let tar_file = File::open(tar_file).unwrap();
                        let mut archive = tar::Archive::new(tar_file);
                        archive.unpack(self.extract_dir.clone()).unwrap();
                    }
                    ExtractCommand::Zip => {
                        let archive_file = File::open(download_file).unwrap();
                        let mut archive = zip::ZipArchive::new(archive_file).unwrap();
                        for i in 0..archive.len() {
                            let mut file = archive.by_index(i).unwrap();
                            // eprintln!("=> {:?}", file.name());
                            let outpath = match file.enclosed_name() {
                                Some(path) => path.to_owned(),
                                None => continue,
                            };
                            let outpath = self.extract_dir.join(outpath);
                            // eprintln!("=> {:?}", outpath);

                            if (*file.name()).ends_with('/') {
                                // println!("File {} extracted to \"{}\"", i, outpath.display());
                                fs::create_dir_all(&outpath).unwrap();
                            } else {
                                if verbose {
                                    println!(
                                        "File {} extracted to \"{}\" ({} bytes)",
                                        i,
                                        outpath.display(),
                                        file.size()
                                    );
                                }
                                if let Some(p) = outpath.parent() {
                                    if !p.exists() {
                                        fs::create_dir_all(&p).unwrap();
                                    }
                                }
                                let mut outfile = fs::File::create(&outpath).unwrap();
                                io::copy(&mut file, &mut outfile).unwrap();
                            }

                            // Get and Set permissions
                            #[cfg(unix)]
                            {
                                use std::os::unix::fs::PermissionsExt;

                                if let Some(mode) = file.unix_mode() {
                                    fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))
                                        .unwrap();
                                }
                            };
                        }
                    }
                }

                // set current directory
                // inside the extracted directory
                // if it is not a tarbomb
                let children = fs::read_dir(self.extract_dir.clone())
                    .unwrap()
                    .map(|r| r.unwrap())
                    .collect::<Vec<DirEntry>>();

                if children.len() == 1 {
                    let child = children.get(0).unwrap().file_name();
                    let child = child.to_str().unwrap();

                    let file = self.extract_dir.clone().join(child);
                    if file.is_dir() {
                        self.state.current_dir = self.extract_dir.clone().join(child);
                    }
                }
                Ok(format!(
                    "Extracted file in {}",
                    self.state.current_dir.display()
                ))
            }
            Command::Copy() => {
                for path in
                    fs::read_dir(self.state.current_dir.clone()).map_err(|e| e.to_string())?
                {
                    let filename = path.unwrap().file_name();
                    let filename = filename.to_str().unwrap();
                    let source = self.state.current_dir.join(filename);
                    let dst = self.package_dir.join(filename);
                    if verbose {
                        eprintln!("Copying {} to {}", source.display(), dst.display());
                    }
                    if source.is_dir() {
                        copy_dir_all(&source, &dst).map_err(|e| {
                            format!("Copying directory {}: {}", source.display(), e)
                        })?;
                    } else {
                        fs::copy(&source, &dst)
                            .map_err(|e| format!("Copying file {}: {}", source.display(), e))?;
                    }
                }
                Ok(format!(
                    "Copying files from {} to {}",
                    self.state.current_dir.display(),
                    self.package_dir.display()
                ))
            }
            Command::Shell(s) => {
                // extract shell script
                let script_file = self.installer_dir.join("build.sh");
                let mut file = if let Ok(f) = File::create(script_file.clone()) {
                    f
                } else {
                    return Err(format!(
                        "Can not create build script {}",
                        script_file.display()
                    ));
                };

                if let Err(e) = file.write_all(s.as_bytes()) {
                    return Err(e.to_string());
                }

                if let Err(e) = env::set_current_dir(self.state.current_dir.clone()) {
                    return Err(e.to_string());
                }

                // set environment variable
                env::set_var("PACKAGE_DIR", self.package_dir.clone());
                env::set_var("PACKAGES_DIR", self.package_dir.parent().unwrap());
                if let Some(download_file) = self.state.download_file.clone() {
                    env::set_var("DOWNLOAD_FILE", download_file);
                }

                match exec_script(&script_file, verbose) {
                    Ok(_) => Ok(format!(
                        "Script {} executed with success",
                        script_file.display()
                    )),
                    Err(e) => Err(e),
                }
            }
        }
    }
}

// in verbose, output the stdout/stderr of the script execution
fn exec_script(script_file: &Path, verbose: bool) -> Result<(), String> {
    let mut bash_command = process::Command::new("bash");
    let command = bash_command.arg("-eu").arg(script_file);

    let command = if verbose {
        command.stdout(Stdio::inherit()).stderr(Stdio::inherit())
    } else {
        command
    };
    match command.output() {
        Ok(output) => {
            if output.status.success() {
                Ok(())
            } else {
                match String::from_utf8(output.stderr) {
                    Ok(s) => Err(s.trim().to_string()),
                    Err(e) => Err(e.to_string().trim().to_string()),
                }
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else if ty.is_symlink() {
            // recreate symlink
            let original = fs::read_link(entry.path()).unwrap();
            let link = entry.file_name();
            let link = link.to_str().unwrap();
            let link = format!("{}/{}", dst.display(), link);
            std::os::unix::fs::symlink(original, link).unwrap();
        } else {
            //eprintln!(">> copying file {}", entry.path().display());
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

// uncompress in place
// remove file extension .xz by the client
fn uncompress_xz(input_file: &Path, output_file: &Path) -> Result<(), String> {
    let mut contents = Vec::new();
    if let Err(e) = File::open(input_file).unwrap().read_to_end(&mut contents) {
        return Err(format!("Can not open {}: {}", input_file.display(), e));
    };
    let mut ret = Vec::new();
    match xz2::read::XzDecoder::new_multi_decoder(contents.as_slice()).read_to_end(&mut ret) {
        Ok(_) => {
            let mut file = match File::create(output_file) {
                Ok(f) => f,
                Err(e) => {
                    return Err(format!(
                        "Can not create file {}: {}",
                        output_file.display(),
                        e
                    ));
                }
            };
            if let Err(e) = file.write_all(&ret) {
                return Err(e.to_string());
            }
            Ok(())
        }
        Err(e) => Err(e.to_string()),
    }
}

// uncompress in place
// remove file extension .xz by the client
fn uncompress_bz2(input_file: &Path, output_file: &Path) -> Result<(), String> {
    let mut contents = Vec::new();
    if let Err(e) = File::open(input_file).unwrap().read_to_end(&mut contents) {
        return Err(format!("Can not open {}: {}", input_file.display(), e));
    };
    let mut ret = vec![];
    let mut decompressor = bzip2::read::BzDecoder::new(contents.as_slice());
    match decompressor.read_to_end(&mut ret) {
        Ok(_) => {
            let mut file = match File::create(output_file) {
                Ok(f) => f,
                Err(e) => {
                    return Err(format!(
                        "Can not create file {}: {}",
                        output_file.display(),
                        e
                    ));
                }
            };
            if let Err(e) = file.write_all(&ret) {
                Err(e.to_string())
            } else {
                Ok(())
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_uncompress_targz() {
        let gz_file = Path::new("tests/resources/mypackage-0.1.0-x86_64-linux.tar.gz");
        let gz_file = File::open(gz_file.clone()).unwrap();
        let tar = flate2::read::GzDecoder::new(gz_file);
        dbg!(tar.into_inner());
    }

    #[test]
    pub fn test_script() {
        let script_file = Path::new("tests/resources/build_ok1.sh");
        assert!(exec_script(script_file, true).is_ok());

        // in verbose mode, error message is already output to stderr
        // you can't then have it anymore
        let script_file = Path::new("tests/resources/build_nok1.sh");
        assert_eq!(
            exec_script(script_file, true).err().unwrap(),
            "".to_string()
        );
        assert_eq!(
            exec_script(script_file, false).err().unwrap(),
            "tests/resources/build_nok1.sh: line 4: /xxx: No such file or directory".to_string()
        );
    }

    //#[test]
    pub fn test_script2() {
        let save_current_directory = std::env::current_dir().unwrap();
        let current_directory = Path::new("target/current_dir");
        if current_directory.exists() {
            fs::remove_dir_all(current_directory.display().to_string()).expect("directory deleted");
        }
        fs::create_dir(current_directory).expect("directory created");
        std::env::set_current_dir(&current_directory).expect("set current directory");

        //let script_file = Path::new("../../tests/resources/build_ok1.sh");
        let script_file = Path::new("/tmp/store/openjdk:11.0.2/build.sh");
        assert!(exec_script(script_file, true).is_ok());

        std::env::set_current_dir(save_current_directory).unwrap();
    }

    #[test]
    pub fn test_copy() {
        let source = Path::new("tests/linked_directory");
        let target = Path::new("target/copy");

        if target.exists() {
            fs::remove_dir_all(target.display().to_string()).expect("directory deleted");
        }
        fs::create_dir(target).expect("directory created");
        copy_dir_all(source, target).unwrap();
    }
}
