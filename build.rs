fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winresource::WindowsResource::new();
        res.set(
            "FileDescription",
            "AutoSCUNET - SCU Network Auto Login Tool",
        );
        res.set("ProductName", "AutoSCUNET");
        res.set("LegalCopyright", "Copyright (C) 2024 East Monster");
        res.compile().unwrap();
    }
}
