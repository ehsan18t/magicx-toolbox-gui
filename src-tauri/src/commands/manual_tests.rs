use crate::error::Result;

/// Always registered: the frontend shows the Manual Tests view only when this is `true`.
#[tauri::command]
pub async fn manual_tests_available() -> Result<bool> {
    log::info!("manual_tests_available");
    Ok(cfg!(feature = "test-build"))
}

#[cfg(feature = "test-build")]
#[tauri::command]
pub async fn list_manual_tests() -> Result<Vec<crate::manual_tests::ManualTestView>> {
    log::info!("list_manual_tests");
    Ok(crate::manual_tests::list())
}

#[cfg(feature = "test-build")]
#[tauri::command]
pub async fn run_manual_test(
    app: tauri::AppHandle,
    test_id: String,
    minutes: Option<u32>,
) -> Result<crate::manual_tests::ManualTestReport> {
    log::info!("run_manual_test: '{test_id}' minutes {minutes:?}");
    crate::manual_tests::run(app, test_id, minutes).await
}

#[cfg(feature = "test-build")]
#[tauri::command]
pub async fn cancel_manual_test() -> Result<()> {
    log::info!("cancel_manual_test");
    crate::manual_tests::cancel();
    Ok(())
}
