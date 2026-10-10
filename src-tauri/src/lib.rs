use std::sync::Mutex;
use tauri::{Emitter, Manager};

#[cfg(not(target_os = "android"))]
mod desktop {
    use super::*;
    use std::{
        io::{BufRead, BufReader, Write},
        process::{Child, ChildStdin, Command, Stdio},
        thread,
    };
    pub struct Engine(pub Mutex<Option<(Child, ChildStdin)>>);
    impl Drop for Engine {
        fn drop(&mut self) {
            if let Ok(mut guard) = self.0.lock() {
                if let Some((mut child, _)) = guard.take() {
                    let _ = child.kill();
                    let _ = child.wait();
                }
            }
        }
    }
    #[tauri::command]
    pub fn engine_start(
        app: tauri::AppHandle,
        state: tauri::State<'_, Engine>,
    ) -> Result<(), String> {
        let mut guard = state.0.lock().map_err(|e| e.to_string())?;
        if guard.is_some() {
            return Ok(());
        }
        let dir = app
            .path()
            .resource_dir()
            .map_err(|e| e.to_string())?
            .join("resources/engine");
        let mut child = Command::new(dir.join(if cfg!(windows) { "rapfi.exe" } else { "rapfi" }))
            .current_dir(&dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Rapfi start: {e}"))?;
        let stdin = child.stdin.take().ok_or("missing stdin")?;
        let stdout = child.stdout.take().ok_or("missing stdout")?;
        let stderr = child.stderr.take().ok_or("missing stderr")?;
        let handle = app.clone();
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines().flatten() {
                let _ = handle.emit("rapfi-stdout", line);
            }
        });
        thread::spawn(move || {
            for line in BufReader::new(stderr).lines().flatten() {
                eprintln!("[rapfi] {line}");
            }
        });
        *guard = Some((child, stdin));
        Ok(())
    }
    #[tauri::command]
    pub fn engine_send(state: tauri::State<'_, Engine>, command: String) -> Result<(), String> {
        if command.contains(['\n', '\r']) {
            return Err("invalid command".into());
        }
        let mut guard = state.0.lock().map_err(|e| e.to_string())?;
        let (_, stdin) = guard.as_mut().ok_or("engine not running")?;
        writeln!(stdin, "{command}")
            .and_then(|_| stdin.flush())
            .map_err(|e| e.to_string())
    }
    #[tauri::command]
    pub fn engine_stop(state: tauri::State<'_, Engine>) -> Result<(), String> {
        let mut guard = state.0.lock().map_err(|e| e.to_string())?;
        if let Some((mut child, _)) = guard.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        Ok(())
    }
}

#[cfg(target_os = "android")]
mod android {
    use super::*;
    use std::{
        ffi::{c_char, CStr, CString},
        sync::OnceLock,
    };
    static APP: OnceLock<tauri::AppHandle> = OnceLock::new();
    static STARTED: Mutex<bool> = Mutex::new(false);
    unsafe extern "C" {
        fn rapfi_bridge_start(config: *const c_char, cb: extern "C" fn(*const c_char)) -> i32;
        fn rapfi_bridge_send(command: *const c_char);
    }
    extern "C" fn on_output(line: *const c_char) {
        if line.is_null() {
            return;
        }
        if let Some(app) = APP.get() {
            let text = unsafe { CStr::from_ptr(line) }
                .to_string_lossy()
                .into_owned();
            let _ = app.emit("rapfi-stdout", text);
        }
    }
    #[tauri::command]
    pub fn engine_start(app: tauri::AppHandle) -> Result<(), String> {
        let mut started = STARTED.lock().map_err(|e| e.to_string())?;
        if *started {
            return Ok(());
        }
        let dir = app
            .path()
            .app_data_dir()
            .map_err(|e| e.to_string())?
            .join("engine");
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        if std::fs::read_to_string(dir.join(".asset-version"))
            .ok()
            .as_deref()
            != Some(env!("RAPFI_ASSET_VERSION"))
        {
            for (name, bytes) in [
                (
                    "config.toml",
                    include_bytes!("../resources/engine/config.toml").as_slice(),
                ),
                (
                    "mix9svqfreestyle_bsmix.bin.lz4",
                    include_bytes!("../resources/engine/mix9svqfreestyle_bsmix.bin.lz4").as_slice(),
                ),
                (
                    "mix9svqstandard_bs15.bin.lz4",
                    include_bytes!("../resources/engine/mix9svqstandard_bs15.bin.lz4").as_slice(),
                ),
                (
                    "mix9svqrenju_bs15_black.bin.lz4",
                    include_bytes!("../resources/engine/mix9svqrenju_bs15_black.bin.lz4")
                        .as_slice(),
                ),
                (
                    "mix9svqrenju_bs15_white.bin.lz4",
                    include_bytes!("../resources/engine/mix9svqrenju_bs15_white.bin.lz4")
                        .as_slice(),
                ),
                (
                    "model210901.bin",
                    include_bytes!("../resources/engine/model210901.bin").as_slice(),
                ),
            ] {
                let dest = dir.join(name);
                std::fs::write(dest, bytes).map_err(|e| e.to_string())?;
            }
            std::fs::write(dir.join(".asset-version"), env!("RAPFI_ASSET_VERSION"))
                .map_err(|e| e.to_string())?;
        }
        std::env::set_current_dir(&dir).map_err(|e| e.to_string())?;
        let config = CString::new(dir.join("config.toml").to_string_lossy().as_bytes())
            .map_err(|e| e.to_string())?;
        APP.set(app).ok();
        let result = unsafe { rapfi_bridge_start(config.as_ptr(), on_output) };
        if result != 0 {
            return Err(format!("Rapfi native initialization failed: {result}"));
        }
        *started = true;
        Ok(())
    }
    #[tauri::command]
    pub fn engine_send(command: String) -> Result<(), String> {
        let command = CString::new(command).map_err(|e| e.to_string())?;
        unsafe {
            rapfi_bridge_send(command.as_ptr());
        }
        Ok(())
    }
    #[tauri::command]
    pub fn engine_stop() -> Result<(), String> {
        // Rapfi is process-global on Android; stop active search, retain engine instance.
        unsafe {
            rapfi_bridge_send(c"YXSTOP".as_ptr());
        }
        Ok(())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(not(target_os = "android"))]
    {
        let backend = std::env::var("GDK_BACKEND").unwrap_or_default();
        let wayland = if backend.is_empty() {
            std::env::var_os("WAYLAND_DISPLAY").is_some()
        } else {
            backend.split(',').next() == Some("wayland")
        };
        if wayland {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
    }
    let builder = tauri::Builder::default();
    #[cfg(not(target_os = "android"))]
    let builder = builder.manage(desktop::Engine(Mutex::new(None)));
    #[cfg(not(target_os = "android"))]
    let builder = builder.invoke_handler(tauri::generate_handler![
        desktop::engine_start,
        desktop::engine_send,
        desktop::engine_stop
    ]);
    #[cfg(target_os = "android")]
    let builder = builder.invoke_handler(tauri::generate_handler![
        android::engine_start,
        android::engine_send,
        android::engine_stop
    ]);
    builder
        .run(tauri::generate_context!())
        .expect("tauri startup failed");
}
