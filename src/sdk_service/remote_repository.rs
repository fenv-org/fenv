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
use std::os::unix::fs::PermissionsExt;
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

    let extract_temp_dir = tempfile::Builder::new().prefix("fenv_extract").tempdir()?;
    let extract_path = extract_temp_dir.path();
    let destination_path = std::path::Path::new(destination);

    download_and_extract(url, extract_path).await?;
    move_extracted_contents(extract_path, destination_path)?;

    debug!(
        "Successfully downloaded and extracted Flutter SDK to: {}",
        destination
    );

    Ok(())
}

async fn download_and_extract(url: &str, extract_path: &std::path::Path) -> anyhow::Result<()> {
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
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} [{bytes_per_sec}]: {msg}: Remaining {eta}")
        .unwrap()
        .progress_chars("#>-"));
    pb.set_message(format!("Downloading '{}'", url));
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();
    let mut buffer = Vec::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        buffer.extend_from_slice(&chunk);
        downloaded += chunk.len() as u64;
        pb.set_position(downloaded);
    }

    pb.finish_with_message("Download completed");

    debug!("Now extracting to: {}", extract_path.display());
    if url.ends_with(".zip") {
        unzip_from_memory(&buffer, extract_path)?;
    } else if url.ends_with(".tar.xz") {
        untar_xz_from_memory(&buffer, extract_path)?;
    } else {
        return Err(anyhow::anyhow!("Unsupported archive format"));
    }

    Ok(())
}

fn unzip_from_memory(data: &[u8], extract_path: &std::path::Path) -> anyhow::Result<()> {
    let cursor = std::io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)?;
    let total_files = archive.len();
    let mut extracted_files = 0;

    let pb = ProgressBar::new(total_files as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} files: {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name();
        pb.set_message(format!("Extracting '{}'", name));
        let outpath = extract_path.join(name);
        if name.ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;

            // Set file permissions from zip file
            if let Some(mode) = file.unix_mode() {
                let mut permissions = std::fs::metadata(&outpath)?.permissions();
                permissions.set_mode(mode);
                std::fs::set_permissions(&outpath, permissions)?;
            }
        }
        extracted_files += 1;
        pb.set_position(extracted_files as u64);
    }
    pb.finish_with_message("Extraction completed");
    Ok(())
}

fn untar_xz_from_memory(data: &[u8], extract_path: &std::path::Path) -> anyhow::Result<()> {
    let mut xz_reader = XzDecoder::new(data);
    let mut tar_data = Vec::new();
    std::io::copy(&mut xz_reader, &mut tar_data)?;

    let cursor = std::io::Cursor::new(&tar_data);
    let mut archive = tar::Archive::new(cursor);
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

    let cursor = std::io::Cursor::new(&tar_data);
    let mut archive = tar::Archive::new(cursor);
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

            // Set file permissions from tar file
            if let Ok(mode) = entry.header().mode() {
                let mut permissions = std::fs::metadata(&outpath)?.permissions();
                permissions.set_mode(mode);
                std::fs::set_permissions(&outpath, permissions)?;
            }
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
    debug!(
        "Starting to move contents from {:?} to {:?}",
        extract_path, destination_path
    );

    // 1. Remove destination directory if it exists
    if destination_path.exists() {
        debug!(
            "Removing existing destination directory: {:?}",
            destination_path
        );
        std::fs::remove_dir_all(destination_path)?;
    }

    // 2. Check for flutter directory
    let flutter_dir = extract_path.join("flutter");
    debug!("Checking for flutter directory at {:?}", flutter_dir);

    if flutter_dir.exists() {
        // 3. If flutter directory exists, move it to destination
        debug!("Moving flutter directory to destination");
        std::fs::rename(&flutter_dir, destination_path)?;
    } else {
        // 4. If flutter directory doesn't exist, move all contents from extract path
        debug!("Moving all contents from extract path to destination");
        std::fs::rename(extract_path, destination_path)?;
    }

    debug!("Successfully moved contents to {:?}", destination_path);

    #[cfg(debug_assertions)]
    {
        debug!("Contents of destination directory:");
        for entry in std::fs::read_dir(destination_path)? {
            let entry = entry?;
            let path = entry.path();
            let file_type = if path.is_dir() { "dir" } else { "file" };
            debug!("  {}: {:?}", file_type, path);
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
