use super::model::{
    flutter_sdk::FlutterSdk,
    remote_flutter_sdk::{GitRefsKind, RemoteFlutterSdk},
};
use crate::{
    context::{Architecture, FenvContext, OperatingSystem},
    external::git_command::GitCommand,
    util::path_like::PathLike,
};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, info};
use std::collections::HashSet;
use std::io::Write;
use xz2::read::XzDecoder;

pub struct RemoteSdkRepository;

pub const REMOTE_SDK_REPOSITORY: RemoteSdkRepository = RemoteSdkRepository;

impl RemoteSdkRepository {
    pub fn fetch_available_sdk_list(
        &self,
        git_command: &impl GitCommand,
    ) -> anyhow::Result<Vec<RemoteFlutterSdk>> {
        let mut sdks = list_remote_sdks_by_tags(git_command)?;
        sdks.extend(list_remote_sdks_by_branches(git_command)?);
        Ok(sdks)
    }

    pub fn install_sdk(
        &self,
        context: &impl FenvContext,
        git_command: &impl GitCommand,
        sdk: &RemoteFlutterSdk,
    ) -> anyhow::Result<PathLike> {
        match &sdk.kind {
            GitRefsKind::Tag(_) => install_sdk_by_tag(context, git_command, sdk),
            GitRefsKind::Head(channel) => {
                let destination = context.fenv_sdk_root(channel);
                git_command.clone_flutter_sdk_by_channel(channel, &destination.to_string())?;
                anyhow::Ok(destination)
            }
        }
    }
}

fn install_sdk_by_tag(
    context: &impl FenvContext,
    git_command: &impl GitCommand,
    sdk: &RemoteFlutterSdk,
) -> Result<PathLike, anyhow::Error> {
    let destination = context.fenv_sdk_root(&sdk.display_name());
    match generate_download_url(context.os(), context.arch(), &sdk.display_name()) {
        Some(url) => match tokio::runtime::Runtime::new()?.block_on(
            download_flutter_sdk_by_version(&url, &destination.to_string()),
        ) {
            Ok(_) => anyhow::Ok(destination),
            Err(e) => {
                info!(
                    "Failed to download SDK from URL: {}. Falling back to git clone. Error: {}",
                    url, e
                );
                git_command
                    .clone_flutter_sdk_by_version(&sdk.display_name(), &destination.to_string())?;
                anyhow::Ok(destination)
            }
        },
        None => {
            git_command
                .clone_flutter_sdk_by_version(&sdk.display_name(), &destination.to_string())?;
            anyhow::Ok(destination)
        }
    }
}

async fn download_flutter_sdk_by_version(url: &str, destination: &str) -> anyhow::Result<()> {
    debug!("Downloading Flutter SDK from: {}", url);

    let temp_dir = tempfile::Builder::new().prefix("fenv_download").tempdir()?;
    let temp_file = temp_dir.path().join("flutter_sdk_archive");

    download_file(url, &temp_file).await?;

    let extract_temp_dir = tempfile::Builder::new().prefix("fenv_extract").tempdir()?;
    let extract_path = extract_temp_dir.path();
    let destination_path = std::path::Path::new(destination);

    if url.ends_with(".zip") {
        unzip_archive(&temp_file, extract_path)?;
    } else if url.ends_with(".tar.xz") {
        untar_xz_archive(&temp_file, extract_path)?;
    } else {
        return Err(anyhow::anyhow!("Unsupported archive format"));
    }

    move_extracted_contents(extract_path, destination_path)?;

    debug!(
        "Successfully downloaded and extracted Flutter SDK to: {}",
        destination
    );
    Ok(())
}

async fn download_file(url: &str, temp_file: &std::path::Path) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Failed to download SDK: HTTP {}",
            response.status()
        ));
    }

    debug!("Downloaded SDK: {}", response.status());

    let total_size = response.content_length().unwrap_or(0);
    let mut file = std::fs::File::create(temp_file)?;

    println!("Now downloading: {}", url);
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} [{bytes_per_sec}] (Remaining: {eta})")
        .unwrap()
        .progress_chars("#>-"));

    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk)?;
        downloaded += chunk.len() as u64;
        pb.set_position(downloaded);
    }

    pb.finish_with_message("Download completed");
    Ok(())
}

fn unzip_archive(
    archive_path: &std::path::Path,
    extract_path: &std::path::Path,
) -> anyhow::Result<()> {
    let archive = std::fs::File::open(archive_path)?;
    let mut archive = zip::ZipArchive::new(archive)?;
    let total_files = archive.len();
    let mut extracted_files = 0;

    let pb = ProgressBar::new(total_files as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} files")
            .unwrap()
            .progress_chars("#>-"),
    );

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name();
        let outpath = extract_path.join(name);
        if name.ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
        extracted_files += 1;
        pb.set_position(extracted_files as u64);
    }
    pb.finish_with_message("Extraction completed");
    Ok(())
}

fn untar_xz_archive(
    archive_path: &std::path::Path,
    extract_path: &std::path::Path,
) -> anyhow::Result<()> {
    let mut xz_file = std::fs::File::open(archive_path)?;
    let mut xz_reader = XzDecoder::new(&mut xz_file);

    let temp_dir = tempfile::Builder::new().prefix("fenv_tar").tempdir()?;
    let tar_temp = temp_dir.path().join("temp.tar");
    let mut tar_file = std::fs::File::create(&tar_temp)?;
    std::io::copy(&mut xz_reader, &mut tar_file)?;
    drop(tar_file);

    let tar_file = std::fs::File::open(&tar_temp)?;
    let mut archive = tar::Archive::new(tar_file);
    let entries = archive.entries()?;
    let total_files = entries.count();
    let mut extracted_files = 0;

    let pb = ProgressBar::new(total_files as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} files")
            .unwrap()
            .progress_chars("#>-"),
    );

    let tar_file = std::fs::File::open(&tar_temp)?;
    let mut archive = tar::Archive::new(tar_file);
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        let name = path.to_str().unwrap();
        let outpath = extract_path.join(name);
        if name.ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut entry, &mut outfile)?;
        }
        extracted_files += 1;
        pb.set_position(extracted_files as u64);
    }
    pb.finish_with_message("Extraction completed");
    Ok(())
}

// Helper function to move extracted contents to destination
fn move_extracted_contents(
    extract_path: &std::path::Path,
    destination_path: &std::path::Path,
) -> anyhow::Result<()> {
    let flutter_dir = extract_path.join("flutter");
    if flutter_dir.exists() {
        for entry in std::fs::read_dir(&flutter_dir)? {
            let entry = entry?;
            let target = destination_path.join(entry.file_name());
            if entry.file_type()?.is_dir() {
                std::fs::create_dir_all(&target)?;
                copy_dir_all(entry.path(), &target)?;
            } else {
                std::fs::copy(entry.path(), &target)?;
            }
        }
    } else {
        // If no flutter directory, move everything to destination
        for entry in std::fs::read_dir(extract_path)? {
            let entry = entry?;
            let target = destination_path.join(entry.file_name());
            if entry.file_type()?.is_dir() {
                std::fs::create_dir_all(&target)?;
                copy_dir_all(entry.path(), &target)?;
            } else {
                std::fs::copy(entry.path(), &target)?;
            }
        }
    }
    Ok(())
}

// Helper function to copy directory recursively
fn copy_dir_all(
    src: impl AsRef<std::path::Path>,
    dst: impl AsRef<std::path::Path>,
) -> std::io::Result<()> {
    std::fs::create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn list_remote_sdks_by_tags(
    git_command: &impl GitCommand,
) -> anyhow::Result<Vec<RemoteFlutterSdk>> {
    let git_output = git_command.list_remote_sdks_by_tags()?;
    debug!("list_remote_sdks_by_tags(): stdout:\n{git_output}");

    let mut lines = git_output.split("\n");
    // Holds kind keys for eliminating duplications
    let mut registered_kind_keys: HashSet<String> = HashSet::new();
    let mut git_refs = lines
        .by_ref()
        .map(|line| RemoteFlutterSdk::parse(line))
        .flatten()
        // Remove duplications
        .filter(|sdk| {
            let key = sdk.kind.key();
            if registered_kind_keys.contains(&key) {
                false
            } else {
                registered_kind_keys.insert(key);
                true
            }
        })
        .collect::<Vec<RemoteFlutterSdk>>();
    git_refs.sort_by(|a, b| a.kind.cmp(&b.kind));
    Ok(git_refs)
}

fn list_remote_sdks_by_branches(
    git_command: &impl GitCommand,
) -> anyhow::Result<Vec<RemoteFlutterSdk>> {
    let git_output = git_command.list_remote_sdks_by_branches()?;
    debug!("list_remote_sdks_by_branches(): stdout:\n{git_output}");

    let mut lines = git_output.split("\n");
    let git_refs = lines
        .by_ref()
        .map(|line| RemoteFlutterSdk::parse(line))
        .flatten()
        .collect::<Vec<RemoteFlutterSdk>>();
    Ok(git_refs)
}

fn generate_download_url(
    os: OperatingSystem,
    arch: Architecture,
    sdk_version: &str,
) -> Option<String> {
    match (os, arch) {
        (OperatingSystem::Linux, Architecture::X86_64) => Some(format!(
            "https://storage.googleapis.com/flutter_infra_release/releases/stable/linux/flutter_linux_{}-stable.tar.xz",
            sdk_version
        )),
        (OperatingSystem::MacOS, Architecture::X86_64) => Some(format!(
            "https://storage.googleapis.com/flutter_infra_release/releases/stable/macos/flutter_macos_{}-stable.zip",
            sdk_version
        )),
        (OperatingSystem::MacOS, Architecture::Aarch64) => Some(format!(
            "https://storage.googleapis.com/flutter_infra_release/releases/stable/macos/flutter_macos_arm64_{}-stable.zip",
            sdk_version
        )),
        _ => None,
    }
}

impl GitRefsKind {
    /// Extracts a key string from `GitRefsKind`.
    fn key(&self) -> String {
        match self {
            GitRefsKind::Tag(version) => format!(
                "{major}.{minor}.{patch}.{hotfix}",
                major = version.major,
                minor = version.minor,
                patch = version.patch,
                hotfix = version.hotfix,
            ),
            GitRefsKind::Head(branch) => String::from(branch),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_download_url_linux_x86_64() {
        let url = generate_download_url(OperatingSystem::Linux, Architecture::X86_64, "3.19.3");
        assert_eq!(
            url,
            Some(String::from("https://storage.googleapis.com/flutter_infra_release/releases/stable/linux/flutter_linux_3.19.3-stable.tar.xz"))
        );
    }

    #[test]
    fn test_generate_download_url_macos_x86_64() {
        let url = generate_download_url(OperatingSystem::MacOS, Architecture::X86_64, "3.19.3");
        assert_eq!(
            url,
            Some(String::from("https://storage.googleapis.com/flutter_infra_release/releases/stable/macos/flutter_macos_3.19.3-stable.zip"))
        );
    }

    #[test]
    fn test_generate_download_url_macos_aarch64() {
        let url = generate_download_url(OperatingSystem::MacOS, Architecture::Aarch64, "3.19.3");
        assert_eq!(
            url,
            Some(String::from("https://storage.googleapis.com/flutter_infra_release/releases/stable/macos/flutter_macos_arm64_3.19.3-stable.zip"))
        );
    }

    #[test]
    fn test_generate_download_url_unsupported_combination() {
        let url = generate_download_url(OperatingSystem::Linux, Architecture::Aarch64, "3.19.3");
        assert_eq!(url, None);
    }
}
