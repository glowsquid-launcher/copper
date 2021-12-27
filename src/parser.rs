use log::trace;

use crate::assets::structs::version_manifest::{Action, GameRule, JvmRule, Value};
use crate::assets::structs::version_manifest::{GameClass, JvmClass};
use crate::launcher::LauncherArgs;
#[cfg(target_os = "windows")]
use winsafe::IsWindows10OrGreater;
pub struct GameArguments;
pub struct JavaArguments;

impl GameArguments {
    // If the rules are not met the function returns ""
    pub fn parse_class_argument(launcher_arguments: &LauncherArgs, argument: &GameClass) -> String {
        trace!("Parsing class argument: {:?}", argument);
        let rules_result = argument.rules.iter().any(|rule| {
            trace!("Checking rule: {:?}", rule);
            !Self::match_rule(rule, launcher_arguments)
        });

        if !rules_result {
            return "".to_string();
        }

        match &argument.value {
            crate::assets::structs::version_manifest::Value::String(argument) => {
                trace!("Parsing singular class argument: {:?}", &argument);
                Self::parse_string_argument(launcher_arguments, argument.to_string())
            }

            crate::assets::structs::version_manifest::Value::StringArray(arguments) => {
                trace!("Parsing multi-argument class argument: {:?}", &arguments);
                let args = arguments
                    .iter()
                    .map(|argument| {
                        trace!("Parsing argument in multi-argument: {:?}", &argument);
                        Self::parse_string_argument(launcher_arguments, argument.to_string())
                    })
                    .collect::<Vec<_>>();

                args.join(" ")
            }
        };

        todo!()
    }

    pub fn parse_string_argument(launcher_arguments: &LauncherArgs, argument: String) -> String {
        trace!("Parsing string argument: {:?}", &argument);
        return if argument.starts_with("${") && argument.ends_with("}") {
            let dynamic_argument = &argument[2..argument.len() - 1].to_string();

            Self::match_dynamic_argument(launcher_arguments, dynamic_argument).to_string()
        } else if argument == "--clientId" {
            if let Some(_) = &launcher_arguments.authentication_details.client_id {
                argument
            } else {
                "".to_string() // dont put in argument if there is no client id
            }
        } else {
            argument
        };
    }

    fn match_dynamic_argument(launcher_arguments: &LauncherArgs, dynamic_argument: &str) -> String {
        // This is based of the 1.18 JSON. This assumes that all accounts are microsoft accounts (As Mojang accounts are being deprecated and soon :crab:ed out of existence).

        trace!("Matching dynamic argument: {:?}", &dynamic_argument);
        let client_id = launcher_arguments
            .authentication_details
            .client_id
            .as_ref()
            .unwrap_or(&"".to_string())
            .clone();

        match dynamic_argument {
            "auth_player_name" => launcher_arguments.authentication_details.username.to_owned(),
            "version_name" => launcher_arguments.version_name.to_owned(),
            "game_directory" => launcher_arguments.game_directory.to_owned(),
            "assets_root" => launcher_arguments.assets_directory.to_owned(),
            "asset_index_name" => launcher_arguments.version_name.to_owned(),
            "auth_uuid" => launcher_arguments.authentication_details.uuid.to_owned(),
            "auth_access_token" => launcher_arguments.authentication_details.access_token.to_owned(),
            "clientid" => client_id,
            "auth_xuid" => launcher_arguments.authentication_details.xbox_uid.to_owned(),
            // we assume that the user is a microsoft account
            "user_type" => "microsoft".to_string(),
            "version_type" => if launcher_arguments.is_snapshot { "snapshot".to_string() } else { "release".to_string() },
            "resolution_width" if launcher_arguments.custom_resolution.is_some() => {
                launcher_arguments.custom_resolution.as_ref().unwrap().width.to_string()
            },
            "resolution_height" if launcher_arguments.custom_resolution.is_some() => {
                launcher_arguments.custom_resolution.as_ref().unwrap().height.to_string()
            },
            _ => panic!("unrecognised game argument {}, please report to https://github.com/glowsquid-launcher/minecraft-rs/issues", dynamic_argument)
        }
    }

    fn match_rule(rule: &GameRule, launcher_arguments: &LauncherArgs) -> bool {
        // based of the 1.18 json
        match rule.action {
            Action::Allow => {
                if let Some(_) = rule.features.is_demo_user {
                    launcher_arguments.authentication_details.is_demo_user
                } else if let Some(_) = rule.features.has_custom_resolution {
                    launcher_arguments.custom_resolution.is_some()
                } else {
                    panic!("unrecognised rule action, please report to https://glowsquid-launcher/minecraft-rs/issues with the version you are using");
                }
            }
            // no disallows yet
            Action::Disallow => panic!("no disallows have been implemented yet. Please report to https://github.com/glowsquid-launcher/minecraft-rs/issues with the version you are using"),
        }
    }
}

impl JavaArguments {
    pub fn parse_string_argument(launcher_arguments: &LauncherArgs, argument: String) -> String {
        argument
            .replace(
                "${natives_directory}",
                //TODO: Add compat with 1.16.5 which uses <version>/natives
                &launcher_arguments.libraries_directory,
            )
            .replace("${launcher_name}", &launcher_arguments.version_name)
            .replace("${launcher_version}", &launcher_arguments.launcher_name)
    }

    pub fn parse_class_argument(launcher_arguments: &LauncherArgs, argument: &JvmClass) -> String {
        for rule in &argument.rules {
            if !Self::match_rule(rule, launcher_arguments) {
                return "".to_string();
            }
        }

        Self::parse_string_argument(
            launcher_arguments,
            match &argument.value {
                Value::String(str) => str.to_string(),
                Value::StringArray(array) => array.join(" "),
            },
        )
    }

    // launcher arguments may be needed in the future
    fn match_rule(rule: &JvmRule, _launcher_arguments: &LauncherArgs) -> bool {
        let mut current_allow = false;

        match rule.action {
            Action::Allow => {
                if let Some(name) = &rule.os.name {
                    current_allow = match &*name.to_owned() {
                        "osx" =>  cfg!(target_os = "macos"),
                        "windows" => {
                             if cfg!(target_os = "windows") {
                                if let Some(ver) = &rule.os.version {
                                    if ver != "^10\\." {
                                        panic!("unrecognised windows version: {:?}, please report to https://github.com/glowsquid-launcher/minecraft-rs/issues with the version you are using", ver);
                                    }
                                    #[cfg(target_os = "windows")]
                                    return IsWindows10OrGreater();
                                    #[cfg(not(target_os = "windows"))]
                                    false
                                } else {
                                    true
                                }
                            } else {
                                false
                            }
                        },
                        "linux" => cfg!(target_os = "linux"),
                        _ => panic!("unrecognised os name {}, please report to https://github.com/glowsquid-launcher/minecraft-rs/issues with the version you are using", name),
                    };
                }

                if current_allow == false {
                    return false;
                }

                if let Some(arch) = &rule.os.arch {
                    match &*arch.to_owned() {
                        "x86" => current_allow = cfg!(target_arch = "x86"),
                        _ => panic!("unrecognised os arch {}, please report to https://github.com/glowsquid-launcher/minecraft-rs/issues with the version you are using ", arch),
                    }
                }
            }
            Action::Disallow => {
                panic!("no disallows have been implemented yet. Please report to https://github.com/glowsquid-launcher/minecraft-rs/issues with the version you are using");
            }
        }
        current_allow
    }
}
