use thiserror::Error;

#[derive(Error, Debug)]
/// Errors relating to downloading and parsing a minecraft version manifest
pub enum VersionError {
    #[error("version.serde_error(error={0})")]
    /// serde_json failed to serialize/deserialize an error
    SerdeError(#[from] serde_json::Error),

    #[error("version.no_path_parent")]
    /// The save path doesn't have a parent this happens if you do not specify the file name
    /// usually
    NoPathParent,

    #[error("version.io_error(error={0})")]
    /// An error happened during an IO operation
    IoError(#[from] std::io::Error),

    #[error("version.no_asset_index")]
    /// an asset index was not provided by the version manifest
    ///
    /// This usually happens when you forget to merge e.g the fabric manifest with the base one
    NoAssetIndex,

    #[error("version.request_error(error={0})")]
    /// An error happened with reqwest.
    RequestError(#[from] reqwest::Error),

    #[error("version.no_libs")]
    /// No libraries were provided by the version manifest
    ///
    /// This usually happens when you forget to merge e.g the fabric manifest with the base one
    NoLibs,

    #[error("version.unsupported_os")]
    /// The OS the app is running on is unsupported. This shouldn't happen. If it does, please file
    /// a bug report
    UnsupportedOs,

    #[error("versions.no_downloads")]
    /// No downloads were provided by the version manifest
    ///
    /// This usually happens when you forget to merge e.g the fabric manifest with the base one
    NoDownloads,

    #[error("version.download_error(error={0})")]
    /// An error happened during a download
    DownloadErr(#[from] DownloadError),

    #[error("version.join_error")]
    /// An error happened when trying to join/wait for a threads output
    JoinError(#[from] tokio::task::JoinError),
}

#[derive(Error, Debug)]
pub enum DownloadError {
    #[error("download.no_path_parent")]
    /// The save path doesn't have a parent this happens if you do not specify the file name
    /// usually
    NoPathParent,

    #[error("download.io_error(error={0})")]
    /// An error happened during an IO operation
    IoError(#[from] std::io::Error),

    #[error("download.request_error(error={0})")]
    /// An error happened with reqwest.
    RequestError(#[from] reqwest::Error),
}

#[derive(Debug, Error)]
pub enum LauncherError {
    #[error("launcher.io_error(error={0})")]
    /// An error happened during an IO operation
    IoError(#[from] std::io::Error),

    #[error("launcher.serde_error(error={0})")]
    /// serde_json failed to serialize/deserialize an error
    SerdeError(#[from] serde_json::Error),

    #[error("launcher.argument_parse_error(error={0})")]
    /// An error happened when parsing arguments
    JavaArgumentParseError(#[from] JavaArgumentsError),

    #[error("launcher.cannot_get_stdout")]
    /// Cannot get the stdout stream from the minecraft process
    CannotGetStdout,

    #[error("launcher.cannot_get_stderr")]
    /// Cannot get the stderr stream from the minecraft process
    CannotGetStderr,

    #[error("Launcher.no_main")]
    /// a main class was not provided by the version manifest
    ///
    /// This usually happens when you forget to merge e.g A manifest that doesn't have a modified
    /// main class with the base one
    NoMainClass,

    #[error("Launcher.no_args")]
    /// arguments were not provided by the version manifest
    ///
    /// This usually happens when you forget to merge e.g A manifest that doesn't have any new args with the base one
    NoArgs,
}

#[derive(Debug, Error)]
pub enum JavaArgumentsError {
    #[error("arguments.no_libs")]
    /// libs were not provided by the version manifest
    ///
    /// This usually happens when you forget to merge e.g A manifest that doesn't have any new libs with the base one
    NoLibrariesFound,

    #[error("arguments.not_valid_utf8_path")]
    /// A path is not valid UTF-8.
    NotValidUtf8Path,

    #[error("launcher.io_error(error={0})")]
    /// An error happened during an IO operation
    IoError(#[from] std::io::Error),

    #[error("launcher.no_download_artifact_path")]
    /// a download artifact path was not provided by the version manifest
    ///
    /// This usually happens when you forget to merge e.g A manifest that doesn't have a modified
    /// download manifest path with the base one
    NoDownloadArtifactPath,

    #[error("launcher.no_libs_path")]
    /// No lib path was found
    ///
    /// this _shouldnt_ happen, but incase it does, this exists
    NoLibsPath,

    #[error("launcher.unrecognised_os")]
    /// The OS in the arguments is not recognized. This shouldn't happen, if it does, file a bug
    UnrecognisedOs,

    #[error("launcher.unrecognised_os_arch")]
    /// The OS arch in the arguments is not recognized. This shouldn't happen, if it does, file a bug
    UnrecognisedOsArch,

    #[error("launcher.no_dissalows")]
    /// No disallows are currently implemented. Please file a bug if this error happens
    NoDissalows,

    #[error("launcher.no_custom_resolution")]
    /// No custom resolution was provided
    ///
    /// this _should NEVER_ happen, but incase it does, this exists. Please file a bug report.
    NoCustomResolutionProvided,

    #[error("launcher.unrecognised_game_argument(arg={0})")]
    /// The launcher encountered a game argument it doesn't know about
    ///
    /// If this happens, report it as a bug
    UnrecognisedGameArgument(String),

    #[error("launcher.unrecognised_allow_rule")]
    /// The launcher encountered an allow rule it doesn't know about
    ///
    /// If this happens, report it as a bug
    UnrecognisedAllowRule,

    #[error("launcher.unrecognised_disallow_rule")]
    /// The launcher encountered a disallow rule it doesn't know about
    ///
    /// If this happens, report it as a bug
    UnrecognisedDisallowRule,
}

#[derive(Error, Debug)]
pub enum SaveError {
    #[error("save.io_error(error={0})")]
    /// An error happened during an IO operation
    IoError(#[from] std::io::Error),

    #[error("save.serde_error(error={0})")]
    /// serde_json failed to serialize/deserialize an error
    SerdeError(#[from] serde_json::Error),

    #[error("save.no_parent_path")]
    /// A path didn't have a parent
    ///
    /// This can happen if you forgot to include the file name
    NoParentPath,

    #[error("save.not_valid_utf8_path")]
    /// A path is not valid UTF-8.
    NotValidUtf8Path,
}
