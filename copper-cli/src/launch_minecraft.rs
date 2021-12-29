use std::path::PathBuf;
use tokio::io::AsyncBufReadExt;

use anyhow::{anyhow, Result};
use copper::{
    assets::structs::launcher_meta::LauncherMeta,
    launcher::{AuthenticationDetails, Launcher, RamSize},
};
use log::{info, warn};

pub async fn launch_minecraft(
    username: String,
    uuid: String,
    access_token: String,
    xbox_uid: String,
    root: PathBuf,
    version_id: String,
) -> Result<()> {
    info!("Launching minecraft");

    let java_dir = if cfg!(windows) {
        java_locator::locate_file("javaw.exe")?
    } else {
        java_locator::locate_file("java")?
    };

    let java_path = PathBuf::from(java_dir).join(if cfg!(windows) { "javaw.exe" } else { "java" });

    let id = if version_id == "latest" {
        LauncherMeta::download_meta()
            .await
            .map_err(|err| anyhow!("Failed to download launcher meta: {}", err))?
            .latest
            .release
    } else {
        version_id
    };

    let authentication_details = AuthenticationDetails {
        username,
        uuid,
        access_token,
        xbox_uid,
        client_id: None,
        is_demo_user: false,
    };

    let launcher = Launcher {
        assets_directory: root.join("assets"),
        authentication_details,
        custom_resolution: None,
        game_directory: root.clone(),
        is_snapshot: false,
        jar_path: (&root)
            .join("versions")
            .join(&id)
            .join(format!("{}.jar", &id)),
        java_path,
        launcher_name: "minecraft.rs".to_string(),
        libraries_directory: root.join("libraries"),
        ram_size: RamSize {
            min: "2024".to_string(),
            max: "4048".to_string(),
        },
        version_manifest_path: root
            .join("versions")
            .join(&id)
            .join(format!("{}.json", &id)),
        version_name: id,
        client_branding: "minecraft.rs".to_string(),
    };

    let game_output = launcher
        .launch(None)
        .await
        .map_err(|err| anyhow!("Failed to launch minecraft: {}", err))?;
    let mut out_reader = game_output.stdout;
    let mut err_reader = game_output.stderr;
    let mut out_buf = vec![];
    let mut err_buf = vec![];

    while let Ok(_) = out_reader.read_until(b'\n', &mut out_buf).await {
        if out_buf.is_empty() {
            break;
        }
        let line = String::from_utf8_lossy(&out_buf);
        info!("JAVA STDOUT: {}", line);
        out_buf.clear();
    }

    while let Ok(_) = err_reader.read_until(b'\n', &mut err_buf).await {
        if err_buf.is_empty() {
            break;
        }
        let line = String::from_utf8_lossy(&err_buf);
        warn!("JAVA STDERR: {}", line);
        err_buf.clear();
    }

    game_output.exit_handle.await?;

    Ok(())
}
