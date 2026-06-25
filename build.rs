use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let bindings_file = PathBuf::from(out_dir).join("genie_bindings.rs");

    let qnn_root = env::var("QNN_SDK_ROOT").ok().map(PathBuf::from);
    let has_qnn = qnn_root
        .as_ref()
        .map(|p| p.exists() && p.join("include/Genie").exists())
        .unwrap_or(false);

    if has_qnn {
        let qnn_base = qnn_root.unwrap();
        println!("cargo:rerun-if-changed=wrapper.h");
        println!("cargo:rerun-if-env-changed=QNN_SDK_ROOT");

        let lib_dir = qnn_base.join("lib/aarch64-windows-msvc");
        let lib_dir_str = lib_dir.to_string_lossy();
        println!("cargo:rustc-link-search={}", lib_dir_str);
        println!("cargo:rustc-link-lib=Genie");

        let target_dir = Path::new(&bindings_file)
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap();

        let dlls = [
            "Genie.dll",
            "QnnHtp.dll",
            "QnnSystem.dll",
            "QnnHtpNetRunExtensions.dll",
            "QnnHtpPrepare.dll",
        ];
        for dll in &dlls {
            let src = lib_dir.join(dll);
            let dest = target_dir.join(dll);
            if src.exists() {
                let _ = fs::copy(&src, &dest);
            }
        }

        let bindings = bindgen::Builder::default()
            .header("wrapper.h")
            .clang_arg(format!(
                "-I{}",
                qnn_base.join("include/Genie").to_string_lossy()
            ))
            .clang_arg(format!(
                "-I{}",
                qnn_base.join("include/QNN").to_string_lossy()
            ))
            .formatter(bindgen::Formatter::Rustfmt)
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            .generate()
            .expect("failed to generate Genie bindings");

        bindings
            .write_to_file(&bindings_file)
            .expect("Couldn't write bindings!");
    } else {
        let dummy_bindings = r#"
            pub type GenieDialogConfig_Handle_t = *const std::os::raw::c_void;
            pub type GenieDialog_Handle_t = *const std::os::raw::c_void;
            pub type GenieDialog_SentenceCode_t = std::os::raw::c_int;
            pub const GENIE_STATUS_SUCCESS: std::os::raw::c_int = 0;
            pub const GenieDialog_SentenceCode_t_GENIE_DIALOG_SENTENCE_COMPLETE: GenieDialog_SentenceCode_t = 0;
            pub const GenieDialog_SentenceCode_t_GENIE_DIALOG_SENTENCE_END: GenieDialog_SentenceCode_t = 1;
            pub const GenieDialog_SentenceCode_t_GENIE_DIALOG_SENTENCE_ABORT: GenieDialog_SentenceCode_t = 2;
            #[no_mangle]
            pub unsafe extern "C" fn GenieDialogConfig_createFromJson(_json: *const std::os::raw::c_char, _config: *mut GenieDialogConfig_Handle_t) -> std::os::raw::c_int { 0 }
            #[no_mangle]
            pub unsafe extern "C" fn GenieDialog_create(_config: GenieDialogConfig_Handle_t, _dialog: *mut GenieDialog_Handle_t) -> std::os::raw::c_int { 0 }
            #[no_mangle]
            pub unsafe extern "C" fn GenieDialogConfig_free(_config: GenieDialogConfig_Handle_t) {}
            #[no_mangle]
            pub unsafe extern "C" fn GenieDialog_free(_dialog: GenieDialog_Handle_t) {}
            #[no_mangle]
            pub unsafe extern "C" fn GenieDialog_query(
                _dialog: GenieDialog_Handle_t,
                _prompt: *const std::os::raw::c_char,
                _sentence_code: GenieDialog_SentenceCode_t,
                _callback: Option<unsafe extern "C" fn(*const std::os::raw::c_char, GenieDialog_SentenceCode_t, *const std::os::raw::c_void)>,
                _user_data: *const std::os::raw::c_void
            ) -> std::os::raw::c_int { 0 }
        "#;
        fs::write(&bindings_file, dummy_bindings).expect("Couldn't write dummy bindings!");
    }
}
