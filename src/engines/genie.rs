use crate::genie;
use anyhow::{Context, Result};
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::{c_char, c_void};
use std::path::Path;
use std::ptr;

pub struct GenieEngine {
    config_handle: genie::GenieDialogConfig_Handle_t,
    dialog_handle: genie::GenieDialog_Handle_t,
}

impl GenieEngine {
    pub fn new(config_path: &Path) -> Result<Self> {
        if !config_path.exists() {
            return Err(anyhow::anyhow!("Config file not found: {:?}", config_path));
        }
        let config_dir = config_path.parent().unwrap_or(Path::new("."));
        let config_filename = config_path.file_name().context("Invalid config filename")?;
        std::env::set_current_dir(config_dir).context("Failed to change working directory")?;
        let config_json = std::fs::read_to_string(config_filename)?;
        let c_config_json = CString::new(config_json).unwrap();
        let mut config_handle: genie::GenieDialogConfig_Handle_t = ptr::null();
        let status = unsafe {
            genie::GenieDialogConfig_createFromJson(c_config_json.as_ptr(), &mut config_handle)
        };
        if status != genie::GENIE_STATUS_SUCCESS as i32 {
            return Err(anyhow::anyhow!("Config creation failed: 0x{:X}", status));
        }
        let mut dialog_handle: genie::GenieDialog_Handle_t = ptr::null();
        let status = unsafe { genie::GenieDialog_create(config_handle, &mut dialog_handle) };
        if status != genie::GENIE_STATUS_SUCCESS as i32 {
            unsafe { genie::GenieDialogConfig_free(config_handle) };
            return Err(anyhow::anyhow!("Genie creation failed: 0x{:X}", status));
        }
        Ok(Self {
            config_handle,
            dialog_handle,
        })
    }

    pub fn query_sync(
        &self,
        prompt: &str,
        max_tokens: usize,
        mut callback: impl FnMut(&str),
    ) -> Result<()> {
        let mut formatted_prompt = prompt.to_string();
        if !formatted_prompt.contains("<|user|>") {
            formatted_prompt = format!("<|user|>\n{}<|end|>\n<|assistant|>\n", formatted_prompt);
        }
        let c_prompt = CString::new(formatted_prompt).unwrap();

        struct GenieSyncContext<'a> {
            callback: &'a mut dyn FnMut(&str),
            done: std::sync::atomic::AtomicBool,
            token_count: usize,
            max_tokens: usize,
            consecutive_whitespace: usize,
        }

        let mut ctx = GenieSyncContext {
            callback: &mut callback,
            done: std::sync::atomic::AtomicBool::new(false),
            token_count: 0,
            max_tokens,
            consecutive_whitespace: 0,
        };

        extern "C" fn genie_sync_callback(
            response: *const c_char,
            sentence_code: crate::genie::GenieDialog_SentenceCode_t,
            user_data: *const c_void,
        ) {
            if user_data.is_null() {
                return;
            }
            let ctx = unsafe { &mut *(user_data as *mut GenieSyncContext) };
            if ctx.done.load(std::sync::atomic::Ordering::SeqCst) {
                return;
            }
            let mut should_finish = false;
            if !response.is_null() {
                let c_str = unsafe { CStr::from_ptr(response) };
                if let Ok(s) = c_str.to_str() {
                    let is_stop = s.contains("<|end|>")
                        || s.contains("<|user|>")
                        || s.contains("<|endoftext|>")
                        || s.contains("</s>");
                    if is_stop {
                        should_finish = true;
                    } else {
                        if s.trim().is_empty() && !s.is_empty() {
                            ctx.consecutive_whitespace += 1;
                            if ctx.consecutive_whitespace > 5 && ctx.token_count > 0 {
                                should_finish = true;
                            }
                        } else {
                            ctx.consecutive_whitespace = 0;
                        }
                        if !should_finish {
                            (ctx.callback)(s);
                            ctx.token_count += 1;
                            if ctx.token_count >= ctx.max_tokens {
                                should_finish = true;
                            }
                        }
                    }
                }
            }
            if sentence_code == crate::genie::GenieDialog_SentenceCode_t_GENIE_DIALOG_SENTENCE_END
                || sentence_code
                    == crate::genie::GenieDialog_SentenceCode_t_GENIE_DIALOG_SENTENCE_COMPLETE
                || sentence_code
                    == crate::genie::GenieDialog_SentenceCode_t_GENIE_DIALOG_SENTENCE_ABORT
                || should_finish
            {
                ctx.done.store(true, std::sync::atomic::Ordering::SeqCst);
            }
        }

        unsafe {
            genie::GenieDialog_query(
                self.dialog_handle,
                c_prompt.as_ptr(),
                genie::GenieDialog_SentenceCode_t_GENIE_DIALOG_SENTENCE_COMPLETE,
                Some(genie_sync_callback),
                &mut ctx as *mut _ as *const c_void,
            );
        }

        while !ctx.done.load(std::sync::atomic::Ordering::SeqCst) {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        Ok(())
    }
}

impl Drop for GenieEngine {
    fn drop(&mut self) {
        unsafe {
            genie::GenieDialog_free(self.dialog_handle);
            genie::GenieDialogConfig_free(self.config_handle);
        }
    }
}
