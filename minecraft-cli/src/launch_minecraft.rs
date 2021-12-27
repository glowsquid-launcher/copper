use std::path::PathBuf;

use log::{info, warn};
use minecraft_rs::{
    assets::structs::launcher_meta::LauncherMeta,
    launcher::{AuthenticationDetails, Launcher, RamSize},
};

pub async fn launch_minecraft(
    username: String,
    uuid: String,
    access_token: String,
    xbox_uid: String,
    root: PathBuf,
) {
    info!("Launching minecraft");
    let java_dir = if cfg!(windows) {
        java_locator::locate_file("javaw.exe").unwrap()
    } else {
        java_locator::locate_file("java").unwrap()
    };
    let version_id = LauncherMeta::download_meta()
        .await
        .expect("Failed to download launcher meta")
        .latest
        .release;
    let java_path = PathBuf::from(java_dir).join(if cfg!(windows) { "javaw.exe" } else { "java" });
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
            .join(&version_id)
            .join(format!("{}.jar", &version_id)),
        java_path,
        launcher_name: "minecraft.rs".to_string(),
        libraries_directory: root.join("libraries"),
        ram_size: RamSize {
            min: "2024".to_string(),
            max: "4048".to_string(),
        },
        version_manifest_path: root
            .join("versions")
            .join(&version_id)
            .join(format!("{}.json", &version_id)),
        version_name: version_id,
        client_branding: "minecraft.rs".to_string(),
    };

    let game_output = launcher.launch(None).await;
    let mut out_reader = game_output.stdout;
    let mut err_reader = game_output.stderr;

    while let Some(line) = out_reader.next_line().await.unwrap() {
        info!("JAVA STDOUT: {}", line);
    }
    while let Some(line) = err_reader.next_line().await.unwrap() {
        warn!("JAVA STDERR: {}", line);
    }
    game_output.exit_handle.await.unwrap();
}
