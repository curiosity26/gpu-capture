use zbus::{Connection, Proxy};
use tokio::runtime::Builder;
use zbus::zvariant::{Value, Str};
use std::collections::HashMap;

/// Create a screencast session via xdg-desktop-portal
pub fn create_screencast_session() -> Result<String, String> {
    let rt = Builder::new_current_thread().enable_all().build()
        .map_err(|e| format!("Failed to create tokio runtime: {}", e))?;

    rt.block_on(async {
        let conn = Connection::session().await
            .map_err(|e| format!("Failed to connect to session bus: {}", e))?;

        let proxy = Proxy::new(
            &conn,
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.ScreenCast",
        ).await
            .map_err(|e| format!("Failed to create proxy: {}", e))?;

        // Call CreateSession with token (a{sv})
        let mut options: HashMap<String, Value<'static>> = HashMap::new();
        options.insert("token".to_string(), Value::Str(Str::from("test-token")));
        let result = proxy
            .call_method("CreateSession", &options)
            .await
            .map_err(|e| format!("Failed to call CreateSession: {}", e))?;

        // Parse result to get session handle
        let session_path: String = result.body().deserialize()
            .map_err(|e| format!("Failed to deserialize result: {}", e))?;
        Ok(session_path)
    })
}

/// Start a screencast session
pub fn start_screencast(session_path: &str) -> Result<(), String> {
    let rt = Builder::new_current_thread().enable_all().build()
        .map_err(|e| format!("Failed to create tokio runtime: {}", e))?;

    rt.block_on(async {
        let conn = Connection::session().await
            .map_err(|e| format!("Failed to connect to session bus: {}", e))?;

        let proxy = Proxy::new(
            &conn,
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.ScreenCast",
        ).await
            .map_err(|e| format!("Failed to create proxy: {}", e))?;

        // Call Start with session path
        let options: HashMap<String, Value<'static>> = HashMap::new();
        let args = (session_path, "", options);
        let _result = proxy
            .call_method("Start", &args)
            .await
            .map_err(|e| format!("Failed to call Start: {}", e))?;

        Ok(())
    })
}
