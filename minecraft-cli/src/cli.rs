use std::path::PathBuf;

use anyhow::Result;
use log::info;
use minecraft_rs::{
    assets::structs::launcher_meta::LauncherMeta,
    launcher::{launch, AuthenticationDetailsBuilder, LauncherArgsBuilder, RamSize},
};
use structopt::StructOpt;

use crate::download_deps::download_deps;

pub async fn handle_args(args: Args) -> Result<()> {
    match args {
        Args::DownloadDependencies {
            root,
            version: version_id,
        } => download_deps(root, version_id).await?,
        Args::Launch {
            root,
            version: _version_id,
            access_token,
            username,
            uuid,
            xbox_uid,
        } => {
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

            let java_path =
                PathBuf::from(java_dir).join(if cfg!(windows) { "javaw.exe" } else { "java" });

            let authentication_details = AuthenticationDetailsBuilder::default()
                .access_token(access_token)
                .client_id(None)
                .is_demo_user(false)
                .username(username)
                .uuid(uuid)
                .xbox_uid(xbox_uid)
                .build()
                .expect("Failed to build authentication details");

            let launcher_args = LauncherArgsBuilder::default()
                .assets_directory(root.join("assets"))
                .authentication_details(authentication_details)
                .custom_resolution(None)
                .game_directory(&root)
                .is_snapshot(false)
                .jar_path(
                    root.join("versions")
                        .join(&version_id)
                        .join(format!("{}.jar", &version_id)),
                )
                .java_path(java_path)
                .launcher_name("minecraft.rs")
                .libraries_directory(root.join("libraries"))
                .ram_size(RamSize {
                    min: "2048".to_string(),
                    max: "4056".to_string(),
                })
                .version_manifest_path(
                    root.join("versions")
                        .join(&version_id)
                        .join(format!("{}.json", &version_id)),
                )
                .version_name(&version_id)
                .client_branding("minecraft.rs")
                .build()
                .expect("Failed to build launcher args");

            launch(launcher_args, None).await;
        }
    }
    Ok(())
}

#[derive(Debug, StructOpt)]
pub enum Args {
    /// Download minecrafts dependencies.
    ///
    /// This includes assets, libraries, the client jar, the client manifest, the version manifest, and more
    DownloadDependencies {
        /// The root .minecraft folder.
        ///
        /// This is where everything will be downloaded to
        #[structopt(short, long)]
        root: String,
        /// The minecraft version.
        ///
        /// This can be any minecraft version (including snapshot versions) and can be "latest" for the latest release
        #[structopt(short, long)]
        version: String,
    },
    /// Launch minecraft
    Launch {
        /// The root .minecraft folder.
        #[structopt(short, long, parse(from_os_str))]
        root: PathBuf,
        /// The minecraft version.
        ///
        /// This can be any minecraft version (including snapshot versions) and can be "latest" for the latest release
        #[structopt(short, long)]
        version: String,

        /// Your access token.
        #[structopt(short, long)]
        access_token: String,

        #[structopt(short, long)]
        username: String,

        #[structopt(short = "id", long)]
        uuid: String,

        #[structopt(short, long)]
        xbox_uid: String,
    },
}
