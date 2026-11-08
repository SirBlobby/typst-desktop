#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "linux")]
fn apply_linux_graphics_defaults() {
    let defaults = [
        ("WEBKIT_DISABLE_DMABUF_RENDERER", "1"),
        ("__NV_DISABLE_EXPLICIT_SYNC", "1"),
    ];
    for (name, value) in defaults {
        if std::env::var_os(name).is_none() {
            std::env::set_var(name, value);
        }
    }
}

fn main() {
    #[cfg(target_os = "linux")]
    apply_linux_graphics_defaults();

    typst_desktop_lib::run()
}
