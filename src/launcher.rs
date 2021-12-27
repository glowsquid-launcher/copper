use std::process::Command;

use crate::assets::structs::version_manifest::VersionManifest;
use crate::parser::JavaArguments;
use crate::{assets, parser::GameArguments};
use derive_builder::Builder;
use log::{debug, trace};

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
    pub authentication_details: AuthenticationDetails,
    pub custom_resolution: Option<CustomResolution>,
    pub jar_path: String,
    pub game_directory: String,
    pub assets_directory: String,
    pub version_manifest_path: String,
    pub is_snapshot: bool,
    pub version_name: String,
    pub ram_size: RamSize,
    pub java_path: String,
}

pub async fn launch(launcher_arguments: LauncherArgs, version_manifest: VersionManifest) {
    trace!("Launching minecraft");

    let game_args = parse_game_arguments(&launcher_arguments, &version_manifest);
    debug!("Game arguments: {:?}", &game_args);
    let java_args = parse_java_arguments(&launcher_arguments, &version_manifest);

    let command = Command::new("java");
}

fn parse_java_arguments(
    launcher_arguments: &LauncherArgs,
    version_manifest: &VersionManifest,
) -> Vec<String> {
    let mut args: Vec<String> = vec![];

    for arg in &version_manifest.arguments.jvm {
        let formatted_arg = match arg {
            assets::structs::version_manifest::JvmElement::JvmClass(argument) => {
                JavaArguments::parse_class_argument(&launcher_arguments, argument)
            }
            assets::structs::version_manifest::JvmElement::String(argument) => {
                JavaArguments::parse_string_argument(&launcher_arguments, argument.to_string())
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
