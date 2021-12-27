use std::path::PathBuf;
use std::process::Stdio;

use crate::assets::structs::version_manifest::VersionManifest;
use crate::parser::JavaArguments;
use crate::{assets, parser::GameArguments};
use derive_builder::Builder;
use log::{debug, trace};
use tokio::fs;
use tokio::process::Command;

#[derive(Default, Builder, Debug, Clone)]
pub struct AuthenticationDetails {
    pub session_id: String,
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub xbox_uid: String,
    pub client_id: Option<String>,
    pub is_demo_user: bool,
}

#[derive(Default, Builder, Debug, Clone)]
pub struct CustomResolution {
    pub width: i32,
    pub height: i32,
}

#[derive(Default, Clone, Builder, Debug)]
#[builder(setter(into), pattern = "mutable")]
pub struct RamSize {
    pub min: String,
    pub max: String,
}

#[derive(Default, Clone, Builder, Debug)]
#[builder(setter(into), pattern = "mutable")]
pub struct LauncherArgs {
    /// the authentication details (username, uuid, access token, xbox uid, etc)
    pub authentication_details: AuthenticationDetails,
    /// a custom resolution to use instead of the default
    pub custom_resolution: Option<CustomResolution>,
    /// the minecraft jar file path
    pub jar_path: PathBuf,
    /// the root .minecraft folder
    pub game_directory: PathBuf,
    /// the assets directory, this is the root of the assets folder
    pub assets_directory: PathBuf,
    /// the libraries directory, this is the root of the libraries folder
    pub libraries_directory: PathBuf,
    /// the path to <version>.json
    pub version_manifest_path: PathBuf,
    /// is this version a snapshot
    pub is_snapshot: bool,
    /// the version name/client branding
    pub version_name: String,
    /// the min/max amount of ram to use
    pub ram_size: RamSize,
    /// the path to javaw.exe
    pub java_path: PathBuf,
    /// the launcher name (e.g glowsquid)
    pub launcher_name: String,
}

pub async fn launch(launcher_arguments: LauncherArgs, version_manifest: Option<VersionManifest>) {
    trace!("Launching minecraft");

    let version_manifest = match version_manifest {
        Some(manifest) => manifest,
        None => serde_json::from_str(
            &fs::read_to_string(launcher_arguments.version_manifest_path.clone())
                .await
                .expect("Failed to read version manifest"),
        )
        .expect("Failed to parse version manifest"),
    };

    let game_args = parse_game_arguments(&launcher_arguments, &version_manifest);
    debug!("Game arguments: {:?}", &game_args);
    let java_args = parse_java_arguments(&launcher_arguments, &version_manifest);
    debug!("Java arguments: {:?}", &java_args);

    let process = Command::new(&launcher_arguments.java_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .args(&java_args)
        .args(&game_args)
        .spawn()
        .expect("Failed to start minecraft");
}

fn parse_java_arguments(
    launcher_arguments: &LauncherArgs,
    version_manifest: &VersionManifest,
) -> Vec<String> {
    let mut args: Vec<String> = vec![];

    for arg in &version_manifest.arguments.jvm {
        let formatted_arg = match arg {
            assets::structs::version_manifest::JvmElement::JvmClass(argument) => {
                JavaArguments::parse_class_argument(&launcher_arguments, version_manifest, argument)
            }
            assets::structs::version_manifest::JvmElement::String(argument) => {
                JavaArguments::parse_string_argument(
                    &launcher_arguments,
                    version_manifest,
                    argument.to_string(),
                )
            }
        };

        args.push(formatted_arg)
    }

    args
}

fn parse_game_arguments(
    launcher_arguments: &LauncherArgs,
    version_manifest: &VersionManifest,
) -> Vec<String> {
    let mut args: Vec<String> = vec![];

    for arg in &version_manifest.arguments.game {
        let formatted_arg = match arg {
            assets::structs::version_manifest::GameElement::GameClass(argument) => {
                GameArguments::parse_class_argument(&launcher_arguments, argument)
            }
            assets::structs::version_manifest::GameElement::String(argument) => {
                GameArguments::parse_string_argument(&launcher_arguments, argument.to_string())
            }
        };

        args.push(formatted_arg)
    }

    args
}
