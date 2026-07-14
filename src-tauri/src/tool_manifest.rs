pub(crate) struct ToolDownload {
    pub(crate) url: &'static str,
    pub(crate) sha256: &'static str,
    pub(crate) sha256_sums_url: Option<&'static str>,
    pub(crate) sha256_sums_file: Option<&'static str>,
    pub(crate) dynamic_latest: bool,
    pub(crate) min_version: Option<&'static str>,
}

pub(crate) struct ToolManifest {
    pub(crate) platform_tools: ToolDownload,
    pub(crate) scrcpy: ToolDownload,
}

pub(crate) const DEFAULT_TOOL_MANIFEST: ToolManifest = ToolManifest {
    platform_tools: ToolDownload {
        url: "https://dl.google.com/android/repository/platform-tools-latest-darwin.zip",
        sha256: "",
        sha256_sums_url: None,
        sha256_sums_file: None,
        dynamic_latest: true,
        min_version: Some("37.0.0"),
    },
    scrcpy: ToolDownload {
        url: "https://github.com/Genymobile/scrcpy/releases/download/v4.0/scrcpy-macos-aarch64-v4.0.tar.gz",
        sha256: "f5167fe047fe4a2ae2c2ea8634c7145a4d64d0b6005f24bb45639a965b8c60d4",
        sha256_sums_url: None,
        sha256_sums_file: None,
        dynamic_latest: false,
        min_version: Some("4.0.0"),
    },
};
