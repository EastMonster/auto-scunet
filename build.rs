fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set("FileDescription", "AutoSCUNET - SCU Network Auto Login Tool");
        res.set("ProductName", "AutoSCUNET");
        res.set("LegalCopyright", "Copyright (C) 2024 East Monster");
        res.compile().unwrap();
    }
}