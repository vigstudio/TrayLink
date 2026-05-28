fn main() {
    tauri_build::build();
    #[cfg(all(target_os = "macos", debug_assertions))]
    {
        println!(
            "cargo:warning=TrayLink dev: nếu Accessibility không hoạt động sau rebuild, chạy `npm run sign:dev` rồi Quit/Mở lại app."
        );
    }
}
