use std::error::Error;

use dunce::canonicalize;
use log::trace;

use crate::assets::structs::version::{Action, GameRule, JvmRule, Value, Version};
use crate::assets::structs::version::{GameClass, JvmClass};
use crate::launcher::Launcher;

#[cfg(target_os = "windows")]
use winsafe::IsWindows10OrGreater;
pub struct GameArguments;
pub struct JavaArguments;

impl GameArguments {
    // If the rules are not met the function returns ""
    pub fn parse_class_argument(launcher_arguments: &Launcher, argument: &GameClass) -> String {
        trace!("Parsing class argument: {:?}", argument);

        let rules_passed = argument.rules.iter().any(|rule| {
            trace!("Checking rule: {:?}", rule);
            trace!(
                "Rule matched: {:?}",
                Self::check_rule(rule, launcher_arguments)
            );
            Self::check_rule(rule, launcher_arguments)
        });

        if !rules_passed {
            return "".to_string();
        }

        Self::parse_string_argument(
            launcher_arguments,
            match &argument.value {
                Value::String(str) => str.to_string(),
                Value::StringArray(array) => array.join(" "),
            },
        )
    }

    pub fn parse_string_argument(launcher_arguments: &Launcher, argument: String) -> String {
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

    fn match_dynamic_argument(launcher_arguments: &Launcher, dynamic_argument: &str) -> String {
        //! This is based of the 1.18 JSON. This assumes that all accounts are microsoft accounts
        //! (As Mojang accounts are being deprecated and soon erased from existence).

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
            "game_directory" => canonicalize(launcher_arguments.game_directory.to_owned()).unwrap().to_str().unwrap().to_owned(),
            "assets_root" => canonicalize(launcher_arguments.assets_directory.to_owned()).unwrap().to_str().unwrap().to_owned(),
            "assets_index_name" => launcher_arguments.version_name.to_owned(),
            "auth_uuid" => launcher_arguments.authentication_details.uuid.to_owned(),
            "auth_access_token" => launcher_arguments.authentication_details.access_token.to_owned(),
            "clientid" => client_id,
            "auth_xuid" => launcher_arguments.authentication_details.xbox_uid.to_owned(),
            // we assume that the user is a microsoft account
            "user_type" => "msa".to_string(),
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

    fn check_rule(rule: &GameRule, launcher_arguments: &Launcher) -> bool {
        // based of the 1.18 json
        match rule.action {
            Action::Allow => {
                if let Some(_) = rule.features.is_demo_user {
                    return launcher_arguments.authentication_details.is_demo_user
                } else if let Some(_) = rule.features.has_custom_resolution {
                    return launcher_arguments.custom_resolution.is_some()
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
    pub async fn parse_string_argument(
        launcher_arguments: &Launcher,
        version_manifest: &Version,
        argument: String,
    ) -> Result<String, Box<dyn Error>> {
        let classpath = Self::create_classpath(version_manifest, launcher_arguments).await?;

        Ok(argument
            .replace(
                "${natives_directory}",
                //TODO: Add compat with 1.16.5 which uses <version>/natives
                &canonicalize(&launcher_arguments.libraries_directory)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
            )
            .replace("${launcher_name}", &launcher_arguments.client_branding)
            .replace("${launcher_version}", &launcher_arguments.launcher_name)
            .replace(
                "${classpath}",
                classpath
                    .join(if cfg!(windows) { ";" } else { ":" })
                    .as_str(),
            ))
    }

    pub async fn parse_class_argument(
        launcher_arguments: &Launcher,
        version_manifest: &Version,
        argument: &JvmClass,
    ) -> Result<String, Box<dyn Error>> {
        for rule in &argument.rules {
            if !Self::check_rule(rule) {
                return Ok("".to_string());
            }
        }

        Self::parse_string_argument(
            launcher_arguments,
            version_manifest,
            match &argument.value {
                Value::String(str) => str.to_string(),
                Value::StringArray(array) => array.join(" "),
            },
        )
        .await
    }

    fn check_rule(rule: &JvmRule) -> bool {
        let mut current_allow = false;

        match rule.action {
            Action::Allow => {
                if let Some(name) = &rule.os.name {
                    current_allow = match &*name.to_owned() {
                        "osx" =>  cfg!(target_os = "macos"),
                        #[cfg(target_os = "windows")]
                        "windows" => {
                                if let Some(ver) = &rule.os.version {
                                    if ver != "^10\\." {
                                        panic!("unrecognised windows version: {:?}, please report to https://github.com/glowsquid-launcher/minecraft-rs/issues with the version you are using", ver);
                                    }

                                    return IsWindows10OrGreater().unwrap_or(false);
                                } else {
                                    true
                                }
                        },
                        #[cfg(not(target_os = "windows"))]
                        "windows" => false,
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

    async fn create_classpath(
        version_manifest: &Version,
        launcher_arguments: &Launcher,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let mut cp = vec![];

        for library in version_manifest
            .libraries
            .as_ref()
            .ok_or("no libraries found")?
        {
            if let Some(rules) = &library.rules {
                if !Version::check_library_rules(rules) {
                    continue;
                }
            }

            cp.push(
                canonicalize(
                    launcher_arguments
                        .libraries_directory
                        .join(library.downloads.artifact.path.as_ref().unwrap()),
                )
                .expect("failed to resolve library path")
                .to_str()
                .unwrap()
                .to_owned(),
            );

            if let Some(classifiers) = &library.downloads.classifiers {
                match std::env::consts::OS {
                    "windows" => {
                        if let Some(windows) = &classifiers.natives_windows {
                            cp.push(
                                canonicalize(
                                    launcher_arguments
                                        .libraries_directory
                                        .join(windows.path.as_ref().unwrap()),
                                )
                                .expect("failed to resolve library path")
                                .to_str()
                                .unwrap()
                                .to_owned(),
                            );
                        } else {
                            continue;
                        }
                    }
                    "macos" => {
                        if let Some(macos) = &classifiers.natives_macos {
                            cp.push(
                                canonicalize(
                                    launcher_arguments
                                        .libraries_directory
                                        .join(macos.path.as_ref().unwrap()),
                                )
                                .expect("failed to resolve library path")
                                .to_str()
                                .unwrap()
                                .to_owned(),
                            );
                        } else if let Some(osx) = &classifiers.natives_osx {
                            cp.push(
                                canonicalize(
                                    launcher_arguments
                                        .libraries_directory
                                        .join(osx.path.as_ref().unwrap()),
                                )
                                .expect("failed to resolve library path")
                                .to_str()
                                .unwrap()
                                .to_owned(),
                            )
                        } else {
                            continue;
                        }
                    }
                    "linux" => {
                        if let Some(linux) = &classifiers.natives_linux {
                            cp.push(
                                canonicalize(
                                    launcher_arguments
                                        .libraries_directory
                                        .join(linux.path.as_ref().unwrap()),
                                )
                                .expect("failed to resolve library path")
                                .to_str()
                                .unwrap()
                                .to_owned(),
                            );
                        } else {
                            continue;
                        }
                    }
                    _ => continue,
                };
            }
        }

        cp.push(
            canonicalize(&launcher_arguments.jar_path)
                .expect("failed to resolve minecraft jar path")
                .to_str()
                .unwrap()
                .to_owned(),
        );

        Ok(cp)
    }
}
