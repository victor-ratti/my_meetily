use std::sync::Arc;
use std::collections::HashMap;
use tauri::command;
use crate::analytics::{AnalyticsClient, AnalyticsConfig};

// Global analytics client
static ANALYTICS_CLIENT: std::sync::Mutex<Option<Arc<AnalyticsClient>>> = std::sync::Mutex::new(None);

#[command]
pub async fn init_analytics() -> Result<(), String> {
    let config = AnalyticsConfig {
        api_key: "phc_cohhHPgfQfnNWl33THRRpCftuRtWx2k5svtKrkpFb04".to_string(),
        host: Some("https://us.i.posthog.com".to_string()),
        enabled: true,
    };
    
    let client = Arc::new(AnalyticsClient::new(config).await);
    
    let mut guard = ANALYTICS_CLIENT.lock().unwrap();
    *guard = Some(client);
    
    Ok(())
}

#[command]
pub async fn disable_analytics() -> Result<(), String> {
    let mut guard = ANALYTICS_CLIENT.lock().unwrap();
    *guard = None;
    Ok(())
}

#[command]
pub async fn track_event(event_name: String, properties: Option<HashMap<String, String>>) -> Result<(), String> {
    let client = {
        let guard = ANALYTICS_CLIENT.lock().unwrap();
        guard.as_ref().cloned()
    };
    
    if let Some(client) = client {
        client.track_event(&event_name, properties).await
    } else {
        Err("Analytics client not initialized".to_string())
    }
}

#[command]
pub async fn identify_user(user_id: String, properties: Option<HashMap<String, String>>) -> Result<(), String> {
    let client = {
        let guard = ANALYTICS_CLIENT.lock().unwrap();
        guard.as_ref().cloned()
    };
    
    if let Some(client) = client {
        client.identify(user_id, properties).await
    } else {
        Err("Analytics client not initialized".to_string())
    }
}

#[command]
pub async fn track_meeting_started(meeting_id: String, meeting_title: String) -> Result<(), String> {
    let client = {
        let guard = ANALYTICS_CLIENT.lock().unwrap();
        guard.as_ref().cloned()
    };
    
    if let Some(client) = client {
        client.track_meeting_started(&meeting_id, &meeting_title).await
    } else {
        Err("Analytics client not initialized".to_string())
    }
}

#[command]
pub async fn track_recording_started(meeting_id: String) -> Result<(), String> {
    let client = {
        let guard = ANALYTICS_CLIENT.lock().unwrap();
        guard.as_ref().cloned()
    };
    
    if let Some(client) = client {
        client.track_recording_started(&meeting_id).await
    } else {
        Err("Analytics client not initialized".to_string())
    }
}

#[command]
pub async fn track_recording_stopped(meeting_id: String, duration_seconds: Option<u64>) -> Result<(), String> {
    let client = {
        let guard = ANALYTICS_CLIENT.lock().unwrap();
        guard.as_ref().cloned()
    };
    
    if let Some(client) = client {
        client.track_recording_stopped(&meeting_id, duration_seconds).await
    } else {
        Err("Analytics client not initialized".to_string())
    }
}

#[command]
pub async fn track_meeting_deleted(meeting_id: String) -> Result<(), String> {
    let client = {
        let guard = ANALYTICS_CLIENT.lock().unwrap();
        guard.as_ref().cloned()
    };
    
    if let Some(client) = client {
        client.track_meeting_deleted(&meeting_id).await
    } else {
        Err("Analytics client not initialized".to_string())
    }
}

#[command]
pub async fn track_settings_changed(setting_type: String, new_value: String) -> Result<(), String> {
    let client = {
        let guard = ANALYTICS_CLIENT.lock().unwrap();
        guard.as_ref().cloned()
    };
    
    if let Some(client) = client {
        client.track_settings_changed(&setting_type, &new_value).await
    } else {
        Err("Analytics client not initialized".to_string())
    }
}

#[command]
pub async fn track_feature_used(feature_name: String) -> Result<(), String> {
    let client = {
        let guard = ANALYTICS_CLIENT.lock().unwrap();
        guard.as_ref().cloned()
    };
    
    if let Some(client) = client {
        client.track_feature_used(&feature_name).await
    } else {
        Err("Analytics client not initialized".to_string())
    }
}

#[command]
pub async fn is_analytics_enabled() -> bool {
    let guard = ANALYTICS_CLIENT.lock().unwrap();
    guard.as_ref().map_or(false, |client| client.is_enabled())
}

// Enhanced analytics commands
#[command]
pub async fn start_analytics_session(user_id: String) -> Result<String, String> {
    let client = {
        let guard = ANALYTICS_CLIENT.lock().unwrap();
        guard.as_ref().cloned()
    };
    
    if let Some(client) = client {
        client.start_session(user_id).await
    } else {
        Err("Analytics client not initialized".to_string())
    }
}

#[command]
pub async fn end_analytics_session() -> Result<(), String> {
    let client = {
        let guard = ANALYTICS_CLIENT.lock().unwrap();
        guard.as_ref().cloned()
    };
    
    if let Some(client) = client {
        client.end_session().await
    } else {
        Err("Analytics client not initialized".to_string())
    }
}

#[command]
pub async fn track_daily_active_user() -> Result<(), String> {
    let client = {
        let guard = ANALYTICS_CLIENT.lock().unwrap();
        guard.as_ref().cloned()
    };
    
    if let Some(client) = client {
        client.track_daily_active_user().await
    } else {
        Err("Analytics client not initialized".to_string())
    }
}

#[command]
pub async fn track_user_first_launch() -> Result<(), String> {
    let client = {
        let guard = ANALYTICS_CLIENT.lock().unwrap();
        guard.as_ref().cloned()
    };
    
    if let Some(client) = client {
        client.track_user_first_launch().await
    } else {
        Err("Analytics client not initialized".to_string())
    }
}

#[command]
pub async fn track_meeting_ended(
    transcription_provider: String,
    transcription_model: String,
    total_duration_seconds: Option<f64>,
    active_duration_seconds: f64,
    pause_duration_seconds: f64,
    microphone_device_type: String,
    system_audio_device_type: String,
    chunks_processed: u64,
    transcript_segments_count: u64,
    had_fatal_error: bool,
) -> Result<(), String> {
    let client = {
        let guard = ANALYTICS_CLIENT.lock().unwrap();
        guard.as_ref().cloned()
    };

    if let Some(client) = client {
        client.track_meeting_ended(
            &transcription_provider,
            &transcription_model,
            total_duration_seconds,
            active_duration_seconds,
            pause_duration_seconds,
            &microphone_device_type,
            &system_audio_device_type,
            chunks_processed,
            transcript_segments_count,
            had_fatal_error,
        ).await
    } else {
        Err("Analytics client not initialized".to_string())
    }
}

// Analytics consent tracking commands
#[command]
pub async fn track_analytics_enabled() -> Result<(), String> {
    let client = {
        let guard = ANALYTICS_CLIENT.lock().unwrap();
        guard.as_ref().cloned()
    };

    if let Some(client) = client {
        client.track_analytics_enabled().await
    } else {
        Err("Analytics client not initialized".to_string())
    }
}

#[command]
pub async fn track_analytics_disabled() -> Result<(), String> {
    let client = {
        let guard = ANALYTICS_CLIENT.lock().unwrap();
        guard.as_ref().cloned()
    };

    if let Some(client) = client {
        client.track_analytics_disabled().await
    } else {
        Err("Analytics client not initialized".to_string())
    }
}

#[command]
pub async fn track_analytics_transparency_viewed() -> Result<(), String> {
    let client = {
        let guard = ANALYTICS_CLIENT.lock().unwrap();
        guard.as_ref().cloned()
    };

    if let Some(client) = client {
        client.track_analytics_transparency_viewed().await
    } else {
        Err("Analytics client not initialized".to_string())
    }
}

#[command]
pub async fn is_analytics_session_active() -> bool {
    let client = {
        let guard = ANALYTICS_CLIENT.lock().unwrap();
        guard.as_ref().cloned()
    };
    
    if let Some(client) = client {
        client.is_session_active().await
    } else {
        false
    }
}
