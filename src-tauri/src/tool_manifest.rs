pub(crate) struct ToolManifest {
    pub(crate) platform_tools_url: &'static str,
    pub(crate) scrcpy_release_api: &'static str,
    pub(crate) allowed_scrcpy_asset_suffixes: &'static [&'static str],
}

pub(crate) const DEFAULT_TOOL_MANIFEST: ToolManifest = ToolManifest {
    platform_tools_url: "https://dl.google.com/android/repository/platform-tools-latest-darwin.zip",
    scrcpy_release_api: "https://api.github.com/repos/Genymobile/scrcpy/releases/latest",
    allowed_scrcpy_asset_suffixes: &[".zip", ".tar.gz", ".tgz"],
};
