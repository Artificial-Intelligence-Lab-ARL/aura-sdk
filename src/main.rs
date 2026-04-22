use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Instant;

pub mod genie {
    #![allow(
        non_upper_case_globals,
        non_camel_case_types,
        non_snake_case,
        dead_code
    )]
    include!(concat!(env!("OUT_DIR"), "/genie_bindings.rs"));
}

struct Stats {
    done: AtomicBool,
    token_count: AtomicUsize,
    start_time: std::sync::Mutex<Option<Instant>>,
}

extern "C" fn query_callback(
    response: *const c_char,
    sentence_code: genie::GenieDialog_SentenceCode_t,
    user_data: *const c_void,
) {
    let stats = unsafe { &*(user_data as *const Stats) };

    if !response.is_null() {
        let mut start_lock = stats.start_time.lock().unwrap();
        if start_lock.is_none() {
            *start_lock = Some(Instant::now());
        }

        let c_str = unsafe { CStr::from_ptr(response) };
        if let Ok(s) = c_str.to_str() {
            print!("{}", s);
            use std::io::Write;
            std::io::stdout().flush().unwrap();
            stats.token_count.fetch_add(1, Ordering::SeqCst);
        }
    }

    if sentence_code == genie::GenieDialog_SentenceCode_t_GENIE_DIALOG_SENTENCE_END
        || sentence_code == genie::GenieDialog_SentenceCode_t_GENIE_DIALOG_SENTENCE_COMPLETE
        || sentence_code == genie::GenieDialog_SentenceCode_t_GENIE_DIALOG_SENTENCE_ABORT
    {
        stats.done.store(true, Ordering::SeqCst);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    println!("Rust: Initializing...");
    std::io::stdout().flush().ok();

    let args: Vec<String> = std::env::args().collect();
    let prompt = if args.len() > 1 {
        &args[1]
    } else {
        "Explain quantum physics in one sentence."
    };

    let config_path = "genie_config.json";
    if !std::path::Path::new(config_path).exists() {
        return Err(format!("missing {} in {:?}", config_path, std::env::current_dir()?).into());
    }
    let config_json = std::fs::read_to_string(config_path)?;
    let c_config_json = CString::new(config_json).unwrap();
    let mut config_handle: genie::GenieDialogConfig_Handle_t = ptr::null();

    let status = unsafe {
        genie::GenieDialogConfig_createFromJson(c_config_json.as_ptr(), &mut config_handle)
    };
    if status != genie::GENIE_STATUS_SUCCESS as i32 {
        return Err(format!("GenieDialogConfig_createFromJson failed: 0x{:X}", status).into());
    }

    let mut dialog_handle: genie::GenieDialog_Handle_t = ptr::null();
    let status = unsafe { genie::GenieDialog_create(config_handle, &mut dialog_handle) };
    if status != genie::GENIE_STATUS_SUCCESS as i32 {
        return Err(format!("GenieDialog_create failed: 0x{:X}", status).into());
    }

    let stats = Stats {
        done: AtomicBool::new(false),
        token_count: AtomicUsize::new(0),
        start_time: std::sync::Mutex::new(None),
    };

    println!("\nPrompt: {}", prompt);
    println!("Response:");

    let total_start = Instant::now();
    let c_prompt = CString::new(prompt).unwrap();

    unsafe {
        genie::GenieDialog_query(
            dialog_handle,
            c_prompt.as_ptr(),
            genie::GenieDialog_SentenceCode_t_GENIE_DIALOG_SENTENCE_COMPLETE,
            Some(query_callback),
            &stats as *const _ as *const c_void,
        );
    }

    while !stats.done.load(Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    println!();

    let total_duration = total_start.elapsed();
    let first_token_duration = stats
        .start_time
        .lock()
        .unwrap()
        .map(|t| t.duration_since(total_start));
    let tokens = stats.token_count.load(Ordering::SeqCst);

    println!("\n--- Statistics ---");
    println!("Generated tokens: {}", tokens);
    println!("Total time: {:.2?}", total_duration);
    if let Some(ftd) = first_token_duration {
        println!("Time to first token (TTFT): {:.2?}", ftd);
        let generation_time = total_duration.saturating_sub(ftd);
        if generation_time.as_secs_f64() > 0.0 {
            let tps = tokens as f64 / generation_time.as_secs_f64();
            println!("Tokens per second (TPS): {:.2}", tps);
        }
    }

    unsafe {
        genie::GenieDialog_free(dialog_handle);
        genie::GenieDialogConfig_free(config_handle);
    }
    Ok(())
}
