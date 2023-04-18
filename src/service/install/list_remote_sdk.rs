use anyhow::Result;

use crate::model::remote_flutter_sdk::RemoteFlutterSdk;

use super::git_command::GitCommand;

pub(crate) fn list_remote_sdks(git_command: &Box<dyn GitCommand>) -> Result<Vec<RemoteFlutterSdk>> {
    let mut sdks = git_command.list_remote_sdks_by_tags()?;
    sdks.extend(git_command.list_remote_sdks_by_branches()?);
    Ok(sdks)
}
