use base64::{engine::general_purpose, Engine as _};
use futures_util::StreamExt;
use reqwest::multipart::{Form, Part};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_machine_uid::MachineUidExt;

const OPENAI_USER_HISTORY_LIMIT: usize = 3;
const OPENAI_WARMUP_DEFAULT_MODEL: &str = "gpt-5-nano-2025-08-07";
const OPENAI_WARMUP_MAX_OUTPUT_TOKENS: i64 = 16;
const OPENAI_SHORT_MAX_OUTPUT_TOKENS: i64 = 200;
const OPENAI_SHORT_SCREENSHOT_MAX_OUTPUT_TOKENS: i64 = 800;
const OPENAI_SHORT_CODE_INTENT_MAX_OUTPUT_TOKENS: i64 = 450;
const OPENAI_MEDIUM_MAX_OUTPUT_TOKENS: i64 = 600;
const OPENAI_AUTO_MAX_OUTPUT_TOKENS: i64 = 800;
static OPENAI_WARMUP_STARTED: AtomicBool = AtomicBool::new(false);
static OPENAI_WARMUP_DISABLED: AtomicBool = AtomicBool::new(false);
static OPENAI_CHAT_REQUEST_STARTED: AtomicBool = AtomicBool::new(false);

fn normalize_openai_response_length(value: Option<&str>) -> String {
    match value.unwrap_or("short").trim().to_lowercase().as_str() {
        "short" => "short".to_string(),
        "medium" => "medium".to_string(),
        "auto" => "auto".to_string(),
        _ => "short".to_string(),
    }
}

fn has_code_intent(message: &str) -> bool {
    let normalized = message.to_lowercase();
    let original = message;

    // Obvious code markers.
    let code_markers = [
        "```",
        "`",
        "import",
        "export",
        "function",
        "const",
        "let",
        "=>",
        ".tsx",
        ".ts",
        ".js",
    ];
    if code_markers.iter().any(|marker| normalized.contains(marker)) {
        return true;
    }

    if original.contains("useEffect(")
        || original.contains("useState(")
        || original.contains("useMemo(")
        || original.contains("useCallback(")
    {
        return true;
    }
    if normalized.contains("useeffect(")
        || normalized.contains("usestate(")
        || normalized.contains("usememo(")
        || normalized.contains("usecallback(")
    {
        return true;
    }

    // Code-intent verbs/phrases (RU + EN).
    let code_intent_phrases = [
        "напиши",
        "реализуй",
        "покажи код",
        "пример кода",
        "кодом",
        "исправь",
        "почини",
        "отрефактори",
        "оптимизируй",
        "допиши",
        "write code",
        "implement",
        "fix",
        "refactor",
        "optimize",
        "example code",
    ];
    code_intent_phrases
        .iter()
        .any(|phrase| normalized.contains(phrase))
}

fn get_app_endpoint() -> Result<String, String> {
    if let Ok(endpoint) = env::var("APP_ENDPOINT") {
        return Ok(endpoint);
    }

    match option_env!("APP_ENDPOINT") {
        Some(endpoint) => Ok(endpoint.to_string()),
        None => Err("APP_ENDPOINT environment variable not set. Please ensure it's set during the build process.".to_string())
    }
}

fn get_api_access_key() -> Result<String, String> {
    if let Ok(key) = env::var("API_ACCESS_KEY") {
        return Ok(key);
    }

    match option_env!("API_ACCESS_KEY") {
        Some(key) => Ok(key.to_string()),
        None => Err("API_ACCESS_KEY environment variable not set. Please ensure it's set during the build process.".to_string())
    }
}

// Secure storage functions
fn get_secure_storage_path(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data directory: {}", e))?;

    Ok(app_data_dir.join("secure_storage.json"))
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct SecureStorage {
    license_key: Option<String>,
    instance_id: Option<String>,
    selected_pluely_model: Option<String>,
    openai_api_key: Option<String>,
}

fn read_secure_storage(app: &AppHandle) -> Result<SecureStorage, String> {
    let storage_path = get_secure_storage_path(app)?;
    if !storage_path.exists() {
        return Ok(SecureStorage::default());
    }

    let content = fs::read_to_string(&storage_path)
        .map_err(|e| format!("Failed to read storage file: {}", e))?;

    serde_json::from_str(&content).map_err(|e| format!("Failed to parse storage file: {}", e))
}

fn write_secure_storage(app: &AppHandle, storage: &SecureStorage) -> Result<(), String> {
    let storage_path = get_secure_storage_path(app)?;
    let content = serde_json::to_string(storage)
        .map_err(|e| format!("Failed to serialize storage: {}", e))?;
    fs::write(&storage_path, content).map_err(|e| format!("Failed to write storage file: {}", e))
}

fn get_openai_api_key_from_storage(app: &AppHandle) -> Result<Option<String>, String> {
    let storage = read_secure_storage(app)?;
    Ok(storage
        .openai_api_key
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty()))
}

fn resolve_openai_warmup_model(app: &AppHandle) -> String {
    read_secure_storage(app)
        .ok()
        .and_then(|storage| storage.selected_pluely_model)
        .and_then(|json| serde_json::from_str::<Model>(&json).ok())
        .filter(|model| model.provider.eq_ignore_ascii_case("openai"))
        .map(|model| model.model.trim().to_string())
        .filter(|model| !model.is_empty())
        .unwrap_or_else(|| OPENAI_WARMUP_DEFAULT_MODEL.to_string())
}

pub async fn warmup_openai_connection(app: AppHandle, http_client: reqwest::Client) {
    if OPENAI_WARMUP_DISABLED.load(Ordering::SeqCst) {
        return;
    }

    if OPENAI_WARMUP_STARTED
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return;
    }

    if OPENAI_CHAT_REQUEST_STARTED.load(Ordering::SeqCst) {
        if cfg!(debug_assertions) {
            eprintln!("[warmup] skipped: chat request already started");
        }
        return;
    }

    let api_key = match get_openai_api_key_from_storage(&app) {
        Ok(Some(value)) => value,
        Ok(None) => {
            if cfg!(debug_assertions) {
                eprintln!("[warmup] skipped: OpenAI API key is not configured");
            }
            return;
        }
        Err(error) => {
            if cfg!(debug_assertions) {
                eprintln!("[warmup] skipped: failed to read OpenAI API key: {}", error);
            }
            return;
        }
    };

    let model = resolve_openai_warmup_model(&app);
    let started_at = Instant::now();
    if cfg!(debug_assertions) {
        eprintln!("[warmup] start: model={}", model);
    }

    let request_body = serde_json::json!({
        "model": model,
        "input": "ping",
        "max_output_tokens": OPENAI_WARMUP_MAX_OUTPUT_TOKENS,
    });

    let result = http_client
        .post("https://api.openai.com/v1/responses")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request_body)
        .timeout(std::time::Duration::from_secs(8))
        .send()
        .await;

    let duration_ms = started_at.elapsed().as_millis();
    match result {
        Ok(response) => {
            if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                OPENAI_WARMUP_DISABLED.store(true, Ordering::SeqCst);
                eprintln!("[warmup] disabled due to 429");
            }
            if cfg!(debug_assertions) {
                eprintln!(
                    "[warmup] done: success={}, status={}, duration_ms={}",
                    response.status().is_success(),
                    response.status(),
                    duration_ms
                );
            }
        }
        Err(error) => {
            if cfg!(debug_assertions) {
                eprintln!("[warmup] failed: duration_ms={}, error={}", duration_ms, error);
            }
        }
    }
}

pub async fn get_stored_credentials(
    app: &AppHandle,
) -> Result<(String, String, Option<Model>), String> {
    let storage_path = get_secure_storage_path(app)?;

    if !storage_path.exists() {
        return Err("No license found. Please activate your license first.".to_string());
    }

    let content = fs::read_to_string(&storage_path)
        .map_err(|e| format!("Failed to read storage file: {}", e))?;

    let storage: SecureStorage = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse storage file: {}", e))?;

    let license_key = storage
        .license_key
        .ok_or("License key not found".to_string())?;
    let instance_id = storage
        .instance_id
        .ok_or("Instance ID not found".to_string())?;

    let selected_model: Option<Model> = storage
        .selected_pluely_model
        .and_then(|json_str| serde_json::from_str(&json_str).ok());

    Ok((license_key, instance_id, selected_model))
}

#[tauri::command]
pub async fn set_openai_api_key(app: AppHandle, key: String) -> Result<(), String> {
    let trimmed = key.trim().to_string();
    if trimmed.is_empty() {
        return Err("OpenAI API key cannot be empty.".to_string());
    }

    let mut storage = read_secure_storage(&app)?;
    storage.openai_api_key = Some(trimmed);
    write_secure_storage(&app, &storage)
}

#[tauri::command]
pub async fn clear_openai_api_key(app: AppHandle) -> Result<(), String> {
    let mut storage = read_secure_storage(&app)?;
    storage.openai_api_key = None;
    write_secure_storage(&app, &storage)
}

#[tauri::command]
pub async fn get_openai_api_key_status(app: AppHandle) -> bool {
    get_openai_api_key_from_storage(&app).ok().flatten().is_some()
}

// Audio API Structs
#[derive(Debug, Serialize, Deserialize)]
pub struct AudioResponse {
    success: bool,
    transcription: Option<String>,
    error: Option<String>,
}

// Chat API Structs
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    user_message: String,
    system_prompt: Option<String>,
    image_base64: Option<serde_json::Value>, // Can be string or array
    history: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    success: bool,
    message: Option<String>,
    error: Option<String>,
}

// Model API Structs
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Model {
    provider: String,
    name: String,
    id: String,
    model: String,
    description: String,
    modality: String,
    #[serde(rename = "isAvailable")]
    is_available: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelsResponse {
    models: Vec<Model>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemPromptResponse {
    prompt_name: String,
    system_prompt: String,
}

// Pluely Prompts API
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluelyPrompt {
    title: String,
    prompt: String,
    #[serde(rename = "modelId")]
    model_id: String,
    #[serde(rename = "modelName")]
    model_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluelyPromptsResponse {
    prompts: Vec<PluelyPrompt>,
    total: i32,
    #[serde(rename = "last_updated")]
    last_updated: Option<String>,
}

// API Response Configuration Structs
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponseConfig {
    url: String,
    user_token: String,
    model: String,
    body: String,
    customer_id: Option<i64>,
    customer_email: Option<String>,
    customer_name: Option<String>,
    license_key: String,
    instance_id: String,
    #[serde(rename = "user_audio")]
    user_audio: Option<UserAudioConfig>,
    errors: Option<Vec<ApiConfigError>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiConfigError {
    includes: String,
    error: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserAudioHeader {
    key: String,
    value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserAudioConfig {
    url: String,
    #[serde(rename = "fallback_url")]
    fallback_url: Option<String>,
    model: String,
    #[serde(rename = "fallback_model")]
    fallback_model: Option<String>,
    #[serde(rename = "user_token")]
    user_token: String,
    #[serde(rename = "fallback_user_token")]
    fallback_user_token: Option<String>,
    headers: Option<Vec<UserAudioHeader>>,
}

// Audio API Command
#[tauri::command]
pub async fn transcribe_audio(
    app: AppHandle,
    audio_base64: String,
    transcription_prompt: Option<String>,
    openai_whisper_token: Option<String>,
    openai_whisper_model: Option<String>,
) -> Result<AudioResponse, String> {
    let audio_bytes = decode_audio_base64(&audio_base64)?;
    let client = reqwest::Client::new();

    if let Some(token) = openai_whisper_token
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let model = openai_whisper_model
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("whisper-1");

        return perform_user_audio_transcription(
            &client,
            "https://api.openai.com/v1/audio/transcriptions",
            token,
            model,
            None,
            &audio_bytes,
            transcription_prompt.as_deref(),
        )
        .await
        .map(|transcription| AudioResponse {
            success: true,
            transcription: Some(transcription),
            error: None,
        });
    }

    let (_, _, selected_model) = get_stored_credentials(&app).await?;
    let provider = selected_model.as_ref().map(|model| model.provider.clone());
    let model = selected_model.as_ref().map(|model| model.model.clone());

    let api_config = fetch_api_response_config(&app, provider.clone(), model.clone()).await?;
    let user_audio_config = api_config.user_audio.as_ref().ok_or_else(|| {
        "Audio transcription is not configured for this workspace. Please contact support."
            .to_string()
    })?;

    let error_provider = provider.clone();
    let error_model = model.clone();
    match perform_user_audio_transcription(
        &client,
        &user_audio_config.url,
        &user_audio_config.user_token,
        &user_audio_config.model,
        user_audio_config.headers.as_ref(),
        &audio_bytes,
        transcription_prompt.as_deref(),
    )
    .await
    {
        Ok(transcription) => Ok(AudioResponse {
            success: true,
            transcription: Some(transcription),
            error: None,
        }),
        Err(primary_error) => {
            let fallback_error_message = if let (Some(fallback_url), Some(fallback_token)) = (
                user_audio_config.fallback_url.as_ref(),
                user_audio_config.fallback_user_token.as_ref(),
            ) {
                let fallback_model = user_audio_config
                    .fallback_model
                    .as_ref()
                    .unwrap_or(&user_audio_config.model);

                match perform_user_audio_transcription(
                    &client,
                    fallback_url,
                    fallback_token,
                    fallback_model,
                    user_audio_config.headers.as_ref(),
                    &audio_bytes,
                    transcription_prompt.as_deref(),
                )
                .await
                {
                    Ok(transcription) => {
                        return Ok(AudioResponse {
                            success: true,
                            transcription: Some(transcription),
                            error: None,
                        });
                    }
                    Err(fallback_error) => Some(fallback_error),
                }
            } else {
                Some("fallback not configured".to_string())
            };

            tracing::warn!(
                primary_error = %primary_error,
                fallback_error = %fallback_error_message
                    .as_deref()
                    .unwrap_or("not attempted"),
                "Audio transcription failed for all configured endpoints"
            );
            tauri::async_runtime::spawn({
                let app = app.clone();
                let error_msg = if let Some(fallback_err) = fallback_error_message {
                    format!("Primary: {} | Fallback: {}", primary_error, fallback_err)
                } else {
                    primary_error.clone()
                };
                async move {
                    report_api_error(
                        app,
                        error_msg,
                        "/api/transcribe".to_string(),
                        error_model,
                        error_provider,
                    )
                    .await;
                }
            });
            Err("Transcription failed. Please try again.".to_string())
        }
    }
}

// Helper function to fetch API response configuration
async fn fetch_api_response_config(
    app: &AppHandle,
    provider: Option<String>,
    model: Option<String>,
) -> Result<ApiResponseConfig, String> {
    // Get environment variables
    let app_endpoint = get_app_endpoint()?;
    let api_access_key = get_api_access_key()?;
    let machine_id: String = app.machine_uid().get_machine_uid().unwrap().id.unwrap();

    // Get stored credentials
    let (license_key, instance_id, _) = get_stored_credentials(app).await?;

    // Make HTTP request to response endpoint
    let client = reqwest::Client::new();
    let url = format!("{}/api/response", app_endpoint);

    let mut request = client
        .get(&url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_access_key))
        .header("license_key", &license_key)
        .header("instance", &instance_id)
        .header("machine_id", &machine_id);

    // Add optional headers
    if let Some(p) = provider {
        request = request.header("provider", p);
    }
    if let Some(m) = model {
        request = request.header("model", m);
    }

    let response = request.send().await.map_err(|e| {
        let error_msg = format!("{}", e);
        if error_msg.contains("url (") {
            let parts: Vec<&str> = error_msg.split(" for url (").collect();
            if parts.len() > 1 {
                format!("Failed to fetch API config: {}", parts[0])
            } else {
                format!("Failed to fetch API config: {}", error_msg)
            }
        } else {
            format!("Failed to fetch API config: {}", error_msg)
        }
    })?;

    // Check if the response is successful
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown server error".to_string());

        // Try to parse error as JSON to get a more specific error message
        if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&error_text) {
            if let Some(error_msg) = error_json.get("error").and_then(|e| e.as_str()) {
                return Err(format!("Server error ({}): {}", status, error_msg));
            } else if let Some(message) = error_json.get("message").and_then(|m| m.as_str()) {
                return Err(format!("Server error ({}): {}", status, message));
            }
        }

        return Err(format!("Server error ({}): {}", status, error_text));
    }
    let api_config: ApiResponseConfig = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse API config response: {}", e))?;
    Ok(api_config)
}

fn map_api_error_message(error_rules: &[ApiConfigError], sources: &[String]) -> String {
    for source in sources {
        for rule in error_rules {
            if !rule.includes.is_empty() && source.contains(&rule.includes) {
                return rule.error.clone();
            }
        }
    }

    if let Some(default_rule) = error_rules
        .iter()
        .find(|rule| rule.includes.trim().is_empty())
    {
        return default_rule.error.clone();
    }

    error_rules
        .first()
        .map(|rule| rule.error.clone())
        .unwrap_or_else(|| {
            "Something went wrong. Please try switching to a different model or contact support."
                .to_string()
        })
}

fn decode_audio_base64(audio_base64: &str) -> Result<Vec<u8>, String> {
    let trimmed = audio_base64.trim();
    let base64_str = if let Some(idx) = trimmed.find(',') {
        &trimmed[idx + 1..]
    } else {
        trimmed
    };

    general_purpose::STANDARD
        .decode(base64_str)
        .map_err(|e| format!("Failed to decode audio data: {}", e))
}

async fn perform_user_audio_transcription(
    client: &reqwest::Client,
    url: &str,
    token: &str,
    model: &str,
    headers: Option<&Vec<UserAudioHeader>>,
    audio_bytes: &[u8],
    transcription_prompt: Option<&str>,
) -> Result<String, String> {
    let audio_part = Part::bytes(audio_bytes.to_vec())
        .file_name("audio.wav")
        .mime_str("audio/wav")
        .map_err(|e| format!("Failed to prepare audio payload: {}", e))?;

    let mut form = Form::new()
        .part("file", audio_part)
        .text("model", model.to_string());

    if is_openai_whisper_transcription(url, model) {
        if let Some(prompt) = transcription_prompt
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            form = form.text("prompt", prompt.to_string());
        }
    }

    if let Some(extra_headers) = headers {
        for header in extra_headers {
            let key = header.key.trim();
            if key.is_empty() {
                continue;
            }

            form = form.text(key.to_string(), header.value.clone());
        }
    }

    let response = client
        .post(url)
        .bearer_auth(token)
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("Transcription request failed to send: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unable to read transcription error response".to_string());
        return Err(format!(
            "Transcription request returned {} with body: {}",
            status, error_text
        ));
    }

    let body_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read transcription response: {}", e))?;

    if body_text.trim().is_empty() {
        return Err("Transcription response was empty".to_string());
    }

    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body_text) {
        if let Some(text) = json.get("text").and_then(|value| value.as_str()) {
            return Ok(text.to_string());
        }

        if let Some(text) = json
            .get("transcription")
            .and_then(|value| value.as_str())
            .or_else(|| json.get("result").and_then(|value| value.as_str()))
        {
            return Ok(text.to_string());
        }

        return Ok(json.to_string());
    }

    Ok(body_text)
}

fn is_openai_whisper_transcription(url: &str, model: &str) -> bool {
    model == "whisper-1"
        && url.contains("api.openai.com")
        && url.contains("/v1/audio/transcriptions")
}

fn is_openai_chat_completions_endpoint(url: &str, provider: Option<&str>) -> bool {
    if provider == Some("openai") && url.contains("/v1/chat/completions") {
        return true;
    }

    Url::parse(url)
        .ok()
        .map(|parsed| parsed.host_str() == Some("api.openai.com") && parsed.path() == "/v1/chat/completions")
        .unwrap_or(false)
}

fn is_openai_endpoint(url: &str) -> bool {
    Url::parse(url)
        .ok()
        .map(|parsed| parsed.host_str() == Some("api.openai.com"))
        .unwrap_or(false)
}

fn to_responses_content(content: &serde_json::Value) -> serde_json::Value {
    match content {
        serde_json::Value::String(text) => serde_json::json!([{
            "type": "input_text",
            "text": text
        }]),
        serde_json::Value::Array(items) => {
            let mut normalized: Vec<serde_json::Value> = Vec::new();

            for item in items {
                let item_type = item.get("type").and_then(|value| value.as_str());

                match item_type {
                    Some("text") | Some("input_text") => {
                        if let Some(text) = item.get("text").and_then(|value| value.as_str()) {
                            normalized.push(serde_json::json!({
                                "type": "input_text",
                                "text": text
                            }));
                        }
                    }
                    Some("image_url") | Some("input_image") => {
                        let image_url = item
                            .get("image_url")
                            .and_then(|value| value.as_str())
                            .or_else(|| {
                                item.get("image_url")
                                    .and_then(|value| value.get("url"))
                                    .and_then(|value| value.as_str())
                            });

                        if let Some(url) = image_url {
                            normalized.push(serde_json::json!({
                                "type": "input_image",
                                "image_url": url
                            }));
                        }
                    }
                    _ => {
                        if let Some(text) = item.get("text").and_then(|value| value.as_str()) {
                            normalized.push(serde_json::json!({
                                "type": "input_text",
                                "text": text
                            }));
                        }
                    }
                }
            }

            if normalized.is_empty() {
                serde_json::json!([])
            } else {
                serde_json::Value::Array(normalized)
            }
        }
        _ => serde_json::json!([]),
    }
}

fn to_responses_assistant_content(content: &serde_json::Value) -> serde_json::Value {
    match content {
        serde_json::Value::String(text) => serde_json::json!([{
            "type": "output_text",
            "text": text
        }]),
        serde_json::Value::Array(items) => {
            let mut normalized: Vec<serde_json::Value> = Vec::new();

            for item in items {
                let item_type = item.get("type").and_then(|value| value.as_str());
                match item_type {
                    Some("text") | Some("input_text") | Some("output_text") => {
                        if let Some(text) = item.get("text").and_then(|value| value.as_str()) {
                            normalized.push(serde_json::json!({
                                "type": "output_text",
                                "text": text
                            }));
                        }
                    }
                    _ => {
                        if let Some(text) = item.get("text").and_then(|value| value.as_str()) {
                            normalized.push(serde_json::json!({
                                "type": "output_text",
                                "text": text
                            }));
                        }
                    }
                }
            }

            serde_json::Value::Array(normalized)
        }
        _ => serde_json::json!([]),
    }
}

fn to_openai_responses_input(messages: &[serde_json::Value]) -> Vec<serde_json::Value> {
    let mut input: Vec<serde_json::Value> = Vec::new();

    for message in messages {
        let role = message
            .get("role")
            .and_then(|value| value.as_str())
            .unwrap_or("user");

        if role == "system" {
            continue;
        }

        let content = message.get("content").cloned().unwrap_or(serde_json::json!(""));
        let mapped_content = if role == "assistant" {
            to_responses_assistant_content(&content)
        } else {
            to_responses_content(&content)
        };

        input.push(serde_json::json!({
            "role": role,
            "content": mapped_content
        }));
    }

    input
}

fn extract_stream_text_delta(parsed: &serde_json::Value) -> Option<String> {
    if let Some(choices) = parsed.get("choices").and_then(|value| value.as_array()) {
        if let Some(first_choice) = choices.first() {
            if let Some(delta) = first_choice.get("delta") {
                if let Some(content) = delta.get("content").and_then(|value| value.as_str()) {
                    return Some(content.to_string());
                }
            }
        }
    }

    if parsed
        .get("type")
        .and_then(|value| value.as_str())
        == Some("response.output_text.delta")
    {
        if let Some(delta) = parsed.get("delta").and_then(|value| value.as_str()) {
            return Some(delta.to_string());
        }
    }

    None
}

fn extract_usage_metrics(parsed: &serde_json::Value) -> Option<serde_json::Value> {
    if let Some(usage) = parsed.get("usage") {
        if !usage.is_null() {
            return Some(usage.clone());
        }
    }

    if let Some(usage) = parsed
        .get("response")
        .and_then(|response| response.get("usage"))
    {
        if !usage.is_null() {
            return Some(usage.clone());
        }
    }

    None
}

#[tauri::command]
pub async fn chat_stream_response(
    app: AppHandle,
    http_client: State<'_, reqwest::Client>,
    user_message: String,
    system_prompt: Option<String>,
    image_base64: Option<serde_json::Value>,
    history: Option<String>,
    provider: Option<String>,
    model: Option<String>,
    response_length: Option<String>,
    screenshot_mode: Option<bool>,
) -> Result<String, String> {
    OPENAI_CHAT_REQUEST_STARTED.store(true, Ordering::SeqCst);
    let screenshot_mode = screenshot_mode.unwrap_or(false);
    let user_message_code_intent = has_code_intent(&user_message);
    let requested_provider = provider.clone().map(|value| value.to_lowercase());
    let requested_model = model.clone();
    let is_direct_openai_request = requested_provider.as_deref() == Some("openai");

    let mut effective_provider: Option<String>;
    let mut effective_model: Option<String>;
    let mut api_config: ApiResponseConfig;

    if is_direct_openai_request {
        let openai_api_key = get_openai_api_key_from_storage(&app)?.ok_or_else(|| {
            "OpenAI API key is not configured. Please add it in Settings.".to_string()
        })?;
        let selected_model = requested_model
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("gpt-5-nano")
            .to_string();

        effective_provider = Some("openai".to_string());
        effective_model = Some(selected_model.clone());
        api_config = ApiResponseConfig {
            url: "https://api.openai.com/v1/chat/completions".to_string(),
            user_token: openai_api_key,
            model: selected_model,
            body: "{}".to_string(),
            customer_id: None,
            customer_email: None,
            customer_name: None,
            license_key: String::new(),
            instance_id: String::new(),
            user_audio: None,
            errors: None,
        };
    } else {
        // Get stored credentials to get selected model
        let (_, _, selected_model) = get_stored_credentials(&app).await?;
        let (configured_provider, configured_model) = selected_model
            .as_ref()
            .map_or((None, None), |m| (Some(m.provider.clone()), Some(m.model.clone())));

        // Fetch API configuration
        api_config =
            fetch_api_response_config(&app, configured_provider.clone(), configured_model.clone())
                .await?;
        effective_provider = configured_provider;
        effective_model = configured_model;
    }

    if is_openai_endpoint(&api_config.url) {
        let openai_api_key = get_openai_api_key_from_storage(&app)?.ok_or_else(|| {
            "OpenAI API key is not configured. Please add it in Settings.".to_string()
        })?;
        api_config.user_token = openai_api_key;
        if effective_provider.is_none() {
            effective_provider = Some("openai".to_string());
        }
    }

    if effective_model.is_none() {
        effective_model = Some(api_config.model.clone());
    }

    // Parse the body from API config to merge with our request
    let mut extra_body: serde_json::Value = if !api_config.body.is_empty() {
        serde_json::from_str(&api_config.body).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    let is_openai_responses =
        is_openai_chat_completions_endpoint(&api_config.url, effective_provider.as_deref());

    // Build conversation messages (history + current user message)
    let mut conversation_messages: Vec<serde_json::Value> = Vec::new();
    let mut included_user_history_count = 0usize;
    let mut included_assistant_history_count = 0usize;

    // Add history if provided
    if let Some(history_str) = history {
        if let Ok(history_messages) = serde_json::from_str::<Vec<serde_json::Value>>(&history_str) {
            if is_openai_responses {
                let mut user_indices: Vec<usize> = history_messages
                    .iter()
                    .enumerate()
                    .filter_map(|(index, message)| {
                        (message.get("role").and_then(|role| role.as_str()) == Some("user"))
                            .then_some(index)
                    })
                    .collect();

                if user_indices.len() > OPENAI_USER_HISTORY_LIMIT {
                    let keep_from = user_indices.len() - OPENAI_USER_HISTORY_LIMIT;
                    user_indices = user_indices.split_off(keep_from);
                }

                let user_index_set: HashSet<usize> = user_indices.into_iter().collect();
                let latest_assistant_index = history_messages
                    .iter()
                    .enumerate()
                    .rev()
                    .find_map(|(index, message)| {
                        (message
                            .get("role")
                            .and_then(|role| role.as_str())
                            == Some("assistant"))
                        .then_some(index)
                    });

                for (index, message) in history_messages.into_iter().enumerate() {
                    if user_index_set.contains(&index) {
                        included_user_history_count += 1;
                        conversation_messages.push(message);
                    } else if latest_assistant_index == Some(index) {
                        included_assistant_history_count = 1;
                        conversation_messages.push(message);
                    }
                }
            } else {
                conversation_messages.extend(history_messages);
            }
        }
    }

    // Build user message content
    let mut user_content: Vec<serde_json::Value> = Vec::new();

    // Add text content
    user_content.push(serde_json::json!({
        "type": "text",
        "text": user_message
    }));

    // Add image content if provided
    if let Some(image_data) = image_base64 {
        if image_data.is_string() {
            // Single image
            if let Some(image_b64) = image_data.as_str() {
                user_content.push(serde_json::json!({
                    "type": "image_url",
                    "image_url": {
                        "url": format!("data:image/jpeg;base64,{}", image_b64)
                    }
                }));
            }
        } else if image_data.is_array() {
            // Multiple images
            if let Some(images) = image_data.as_array() {
                for image in images {
                    if let Some(img_str) = image.as_str() {
                        user_content.push(serde_json::json!({
                            "type": "image_url",
                            "image_url": {
                                "url": format!("data:image/jpeg;base64,{}", img_str)
                            }
                        }));
                    }
                }
            }
        }
    }

    // Add user message
    conversation_messages.push(serde_json::json!({
        "role": "user",
        "content": user_content
    }));

    if is_openai_responses {
        eprintln!(
            "chat_stream_response openai history: included_user_history={}, included_assistant_history={}, limit={}, total_input_messages={}",
            included_user_history_count,
            included_assistant_history_count,
            OPENAI_USER_HISTORY_LIMIT,
            conversation_messages.len()
        );
    }

    let request_url = if is_openai_responses {
        api_config
            .url
            .replacen("/v1/chat/completions", "/v1/responses", 1)
    } else {
        api_config.url.clone()
    };
    let openai_response_length = if is_openai_responses {
        Some(normalize_openai_response_length(response_length.as_deref()))
    } else {
        None
    };

    // Build request body
    let mut request_body = if is_openai_responses {
        let selected_length = openai_response_length.as_deref().unwrap_or("short");
        let mut short_mode_max_tokens = OPENAI_SHORT_MAX_OUTPUT_TOKENS;
        if selected_length == "short" {
            if screenshot_mode {
                short_mode_max_tokens = OPENAI_SHORT_SCREENSHOT_MAX_OUTPUT_TOKENS;
            } else if user_message_code_intent {
                short_mode_max_tokens = OPENAI_SHORT_CODE_INTENT_MAX_OUTPUT_TOKENS;
            }
        }
        let short_mode_reasoning_effort = if api_config.model.contains("gpt-5.4") {
            "low"
        } else {
            "minimal"
        };
        let (max_output_tokens, reasoning_effort): (Option<i64>, &str) = match selected_length {
            "short" => (Some(short_mode_max_tokens), short_mode_reasoning_effort),
            "medium" => (Some(OPENAI_MEDIUM_MAX_OUTPUT_TOKENS), "medium"),
            "auto" => (Some(OPENAI_AUTO_MAX_OUTPUT_TOKENS), "medium"),
            _ => (Some(OPENAI_SHORT_MAX_OUTPUT_TOKENS), "minimal"),
        };

        let mut body = serde_json::json!({
            "model": api_config.model,
            "input": to_openai_responses_input(&conversation_messages),
            "stream": true
        });

        if let Some(sys_prompt) = system_prompt.as_deref() {
            if !sys_prompt.trim().is_empty() {
                if let Some(obj) = body.as_object_mut() {
                    obj.insert(
                        "instructions".to_string(),
                        serde_json::Value::String(sys_prompt.to_string()),
                    );
                }
            }
        }

        if let Some(obj) = body.as_object_mut() {
            obj.insert(
                "reasoning".to_string(),
                serde_json::json!({ "effort": reasoning_effort }),
            );
            if let Some(token_limit) = max_output_tokens {
                obj.insert(
                    "max_output_tokens".to_string(),
                    serde_json::json!(token_limit),
                );
            }
        }

        body
    } else {
        let mut messages: Vec<serde_json::Value> = Vec::new();
        if let Some(sys_prompt) = system_prompt.as_deref() {
            if !sys_prompt.trim().is_empty() {
                messages.push(serde_json::json!({
                    "role": "system",
                    "content": sys_prompt
                }));
            }
        }
        messages.extend(conversation_messages.clone());

        serde_json::json!({
            "model": api_config.model,
            "messages": messages,
            "stream": true
        })
    };

    // Merge extra body parameters from API config
    if let Some(extra_obj) = extra_body.as_object_mut() {
        let extra_has_openai_max_output_tokens = extra_obj.contains_key("max_output_tokens");
        if let Some(req_obj) = request_body.as_object_mut() {
            for (key, value) in extra_obj.iter() {
                if is_openai_responses && key == "messages" {
                    continue;
                }
                if is_openai_responses && key == "max_tokens" {
                    if !extra_has_openai_max_output_tokens {
                        req_obj.insert("max_output_tokens".to_string(), value.clone());
                    }
                    continue;
                }
                req_obj.insert(key.clone(), value.clone());
            }
        }
    }

    if is_openai_responses {
        let selected_length = openai_response_length.as_deref().unwrap_or("short");
        let max_output_tokens_log = request_body
            .get("max_output_tokens")
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string());
        let reasoning_effort_log = request_body
            .get("reasoning")
            .and_then(|value| value.get("effort"))
            .and_then(|value| value.as_str())
            .unwrap_or("none");
        let code_intent_log = if selected_length == "short" {
            user_message_code_intent.to_string()
        } else {
            "n/a".to_string()
        };
        eprintln!(
            "chat_stream_response openai response controls: response_length={}, screenshot_mode={}, code_intent={}, max_output_tokens={}, reasoning_effort={}",
            selected_length,
            screenshot_mode,
            code_intent_log,
            max_output_tokens_log,
            reasoning_effort_log
        );
    }

    // Make HTTP request to the configured endpoint with streaming
    let request_started_at = Instant::now();
    let client = http_client.inner().clone();
    let error_rules = api_config.errors.clone().unwrap_or_default();

    let response = match client
        .post(&request_url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_config.user_token))
        .json(&request_body)
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            let mut sources = vec![e.to_string()];
            if let Ok(url) = Url::parse(&request_url) {
                sources.push(url.to_string());
            }
            let final_message = map_api_error_message(&error_rules, &sources);
            tauri::async_runtime::spawn({
                let app = app.clone();
                let provider = effective_provider.clone();
                let model = effective_model.clone();
                let error_msg = e.to_string();
                async move {
                    report_api_error(app, error_msg, "/api/chat".to_string(), model, provider)
                        .await;
                }
            });
            return Err(final_message);
        }
    };

    // Check if the response is successful
    if !response.status().is_success() {
        let status = response.status();
        let response_headers = response.headers().clone();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown server error".to_string());
        eprintln!("OPENAI ERROR STATUS: {}", status);
        eprintln!("OPENAI ERROR BODY: {}", error_text);

        let mut sources = vec![error_text.clone(), status.to_string()];

        // Try to parse error as JSON to get a more specific error message
        if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&error_text) {
            if let Some(error_msg) = error_json.get("error").and_then(|e| e.as_str()) {
                sources.push(error_msg.to_string());
            }
            if let Some(message) = error_json.get("message").and_then(|m| m.as_str()) {
                sources.push(message.to_string());
            }
        }

        let is_openai_rate_limited =
            is_openai_endpoint(&request_url) && status == reqwest::StatusCode::TOO_MANY_REQUESTS;
        let retry_after_seconds = response_headers
            .get("retry-after")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.trim().parse::<u64>().ok());

        if is_openai_rate_limited {
            let header_value = |name: &str| -> String {
                response_headers
                    .get(name)
                    .and_then(|value| value.to_str().ok())
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "n/a".to_string())
            };

            eprintln!(
                "chat_stream_response openai rate_limit_429: x_request_id={}, retry_after={}, x_ratelimit_limit_requests={}, x_ratelimit_remaining_requests={}, x_ratelimit_limit_tokens={}, x_ratelimit_remaining_tokens={}, x_ratelimit_reset_requests={}, x_ratelimit_reset_tokens={}",
                header_value("x-request-id"),
                header_value("retry-after"),
                header_value("x-ratelimit-limit-requests"),
                header_value("x-ratelimit-remaining-requests"),
                header_value("x-ratelimit-limit-tokens"),
                header_value("x-ratelimit-remaining-tokens"),
                header_value("x-ratelimit-reset-requests"),
                header_value("x-ratelimit-reset-tokens"),
            );
        }

        let final_message = if is_openai_rate_limited {
            if let Some(wait_seconds) = retry_after_seconds {
                format!(
                    "Rate limit reached. Please wait {} seconds and try again.",
                    wait_seconds
                )
            } else {
                "Rate limit reached. Please wait a few seconds and try again.".to_string()
            }
        } else {
            map_api_error_message(&error_rules, &sources)
        };
        tauri::async_runtime::spawn({
            let app = app.clone();
            let provider = effective_provider.clone();
            let model = effective_model.clone();
            let error_msg = format!("{}: {}", status, error_text);
            async move {
                report_api_error(app, error_msg, "/api/chat".to_string(), model, provider).await;
            }
        });

        let duration_ms = request_started_at.elapsed().as_millis();
        eprintln!(
            "chat_stream_response metrics: provider={}, model={}, endpoint={}, duration_ms={}, first_chunk_ms=none, usage=null, status={}",
            effective_provider.as_deref().unwrap_or("unknown"),
            api_config.model,
            request_url,
            duration_ms,
            status
        );
        return Err(final_message);
    }

    // Handle streaming response
    let mut stream = response.bytes_stream();
    let mut full_response = String::new();
    let mut buffer = String::new();
    let mut usage: Option<serde_json::Value> = None;
    let mut stream_started = false;
    let mut first_chunk_at_ms: Option<u128> = None;

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(bytes) => {
                let chunk_str = String::from_utf8_lossy(&bytes);
                buffer.push_str(&chunk_str);

                // Process complete lines
                let lines: Vec<&str> = buffer.split('\n').collect();
                let incomplete_line = lines.last().unwrap_or(&"").to_string();

                for line in &lines[..lines.len() - 1] {
                    // Process all but the last (potentially incomplete) line
                    let trimmed_line = line.trim();

                    if trimmed_line.starts_with("data: ") {
                        let json_str = trimmed_line.strip_prefix("data: ").unwrap_or("");

                        if json_str == "[DONE]" {
                            break;
                        }

                        if !json_str.is_empty() {
                            // Try to parse the JSON and extract content
                            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_str)
                            {
                                if usage.is_none() {
                                    usage = extract_usage_metrics(&parsed);
                                }

                                if let Some(content) = extract_stream_text_delta(&parsed) {
                                    if first_chunk_at_ms.is_none() {
                                        let first_ms = request_started_at.elapsed().as_millis();
                                        first_chunk_at_ms = Some(first_ms);
                                        eprintln!(
                                            "[stream][backend] first chunk at +{}ms",
                                            first_ms
                                        );
                                    }
                                    full_response.push_str(&content);
                                    // Emit just the content to frontend
                                    let _ = app.emit("chat_stream_chunk", content);
                                    stream_started = true;
                                }
                            }
                        }
                    }
                }

                // Update buffer with incomplete line
                buffer = incomplete_line;
            }
            Err(e) => {
                let sources = vec![e.to_string()];
                let final_message = map_api_error_message(&error_rules, &sources);
                tauri::async_runtime::spawn({
                    let app = app.clone();
                    let provider = effective_provider.clone();
                    let model = effective_model.clone();
                    let error_msg = e.to_string();
                    async move {
                        report_api_error(app, error_msg, "/api/chat".to_string(), model, provider)
                            .await;
                    }
                });
                return Err(final_message);
            }
        }
    }

    // Emit completion event
    let _ = app.emit("chat_stream_complete", &full_response);

    if stream_started && !full_response.is_empty() {
        tauri::async_runtime::spawn({
            let activity_app = app.clone();
            let activity_model = api_config.model.clone();
            let activity_app_version = app.package_info().version.to_string();
            let captured_metrics = usage.clone();
            async move {
                let _ = user_activity(
                    activity_app,
                    captured_metrics,
                    activity_model,
                    activity_app_version,
                )
                .await;
            }
        });
    }

    let duration_ms = request_started_at.elapsed().as_millis();
    let usage_log = usage
        .as_ref()
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".to_string());
    let first_chunk_log = first_chunk_at_ms
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string());
    eprintln!(
        "chat_stream_response metrics: provider={}, model={}, endpoint={}, duration_ms={}, first_chunk_ms={}, usage={}",
        effective_provider.as_deref().unwrap_or("unknown"),
        api_config.model,
        request_url,
        duration_ms,
        first_chunk_log,
        usage_log
    );

    Ok(full_response)
}

async fn user_activity(
    app: AppHandle,
    activity_metrics: Option<serde_json::Value>,
    configured_model: String,
    app_version: String,
) -> Result<(), String> {
    let app_endpoint = match get_app_endpoint() {
        Ok(value) => value,
        Err(_) => return Ok(()),
    };

    let api_access_key = match get_api_access_key() {
        Ok(value) => value,
        Err(_) => return Ok(()),
    };

    let (license_key, instance_id, stored_model) = match get_stored_credentials(&app).await {
        Ok(values) => values,
        Err(_) => return Ok(()),
    };

    let machine_id = match app.machine_uid().get_machine_uid() {
        Ok(id) => id.id.unwrap_or_else(String::new),
        Err(_) => String::new(),
    };

    if machine_id.is_empty() {
        return Ok(());
    }

    let ai_model = stored_model
        .as_ref()
        .map(|model| model.model.clone())
        .unwrap_or(configured_model);

    let mut payload = serde_json::json!({
        "license": license_key,
        "instance": instance_id,
        "machine_id": machine_id,
        "app_version": app_version,
        "ai_model": ai_model,
    });

    if let Some(metrics) = activity_metrics {
        if let Some(obj) = payload.as_object_mut() {
            const METRIC_FIELD_BYTES: [u8; 5] = [117, 115, 97, 103, 101];
            if let Ok(field) = std::str::from_utf8(&METRIC_FIELD_BYTES) {
                obj.insert(field.to_string(), metrics);
            }
        }
    }

    let activity_url = format!("{}/api/activity", app_endpoint.trim_end_matches('/'));
    let client = reqwest::Client::new();

    let _ = client
        .post(&activity_url)
        .header("Authorization", format!("Bearer {}", api_access_key))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await;

    Ok(())
}

async fn report_api_error(
    app: AppHandle,
    error_message: String,
    endpoint: String,
    model: Option<String>,
    provider: Option<String>,
) {
    let app_endpoint = match get_app_endpoint() {
        Ok(value) => value,
        Err(_) => return,
    };

    let api_access_key = match get_api_access_key() {
        Ok(value) => value,
        Err(_) => return,
    };

    let (license_key, instance_id, stored_model) = match get_stored_credentials(&app).await {
        Ok(values) => values,
        Err(_) => return,
    };

    let machine_id = match app.machine_uid().get_machine_uid() {
        Ok(id) => id.id.unwrap_or_default(),
        Err(_) => return,
    };

    if machine_id.is_empty() {
        return;
    }

    let app_version = app.package_info().version.to_string();

    let final_model = model
        .or_else(|| stored_model.as_ref().map(|m| m.model.clone()))
        .unwrap_or_default();

    let final_provider = provider
        .or_else(|| stored_model.as_ref().map(|m| m.provider.clone()))
        .unwrap_or_default();

    let payload = serde_json::json!({
        "machine_id": machine_id,
        "error_message": error_message,
        "app_version": app_version,
        "instance": instance_id,
        "license_key": license_key,
        "endpoint": endpoint,
        "model": final_model,
        "provider": final_provider
    });

    let error_url = format!("{}/api/error", app_endpoint.trim_end_matches('/'));
    let client = reqwest::Client::new();

    tracing::debug!("Reporting API error: {:?}", payload);

    if let Err(e) = client
        .post(&error_url)
        .header("Authorization", format!("Bearer {}", api_access_key))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
    {
        tracing::warn!("Failed to report API error: {}", e);
    }
}

// Models API Command
#[tauri::command]
pub async fn fetch_models(app: AppHandle) -> Result<Vec<Model>, String> {
    // Get environment variables
    let app_endpoint = get_app_endpoint()?;
    let api_access_key = get_api_access_key()?;

    let (license_key, instance_id) = match get_stored_credentials(&app).await {
        Ok((lk, id, _)) => (lk, id),
        Err(_) => ("".to_string(), "".to_string()),
    };
    let machine_id = app
        .machine_uid()
        .get_machine_uid()
        .ok()
        .and_then(|uid| uid.id)
        .unwrap_or_else(|| "".to_string());
    let app_version = app.package_info().version.to_string();

    // Make HTTP request to models endpoint
    let client = reqwest::Client::new();
    let url = format!("{}/api/models", app_endpoint);

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_access_key))
        .header("license_key", &license_key)
        .header("instance", &instance_id)
        .header("machine_id", &machine_id)
        .header("app_version", &app_version)
        .send()
        .await
        .map_err(|e| {
            let error_msg = format!("{}", e);
            if error_msg.contains("url (") {
                // Remove the URL part from the error message
                let parts: Vec<&str> = error_msg.split(" for url (").collect();
                if parts.len() > 1 {
                    format!("Failed to make models request: {}", parts[0])
                } else {
                    format!("Failed to make models request: {}", error_msg)
                }
            } else {
                format!("Failed to make models request: {}", error_msg)
            }
        })?;

    // Check if the response is successful
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown server error".to_string());

        // Try to parse error as JSON to get a more specific error message
        if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&error_text) {
            if let Some(error_msg) = error_json.get("error").and_then(|e| e.as_str()) {
                return Err(format!("Server error ({}): {}", status, error_msg));
            } else if let Some(message) = error_json.get("message").and_then(|m| m.as_str()) {
                return Err(format!("Server error ({}): {}", status, message));
            }
        }

        return Err(format!("Server error ({}): {}", status, error_text));
    }

    let models_response: ModelsResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse models response: {}", e))?;

    Ok(models_response.models)
}

// Fetch Pluely Prompts API
#[tauri::command]
pub async fn fetch_prompts() -> Result<PluelyPromptsResponse, String> {
    let app_endpoint = get_app_endpoint()?;
    let api_access_key = get_api_access_key()?;

    let client = reqwest::Client::new();
    let url = format!("{}/api/prompts", app_endpoint);

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_access_key))
        .send()
        .await
        .map_err(|e| {
            let error_msg = format!("{}", e);
            if error_msg.contains("url (") {
                let parts: Vec<&str> = error_msg.split(" for url (").collect();
                if parts.len() > 1 {
                    format!("Failed to make prompts request: {}", parts[0])
                } else {
                    format!("Failed to make prompts request: {}", error_msg)
                }
            } else {
                format!("Failed to make prompts request: {}", error_msg)
            }
        })?;

    // Check if the response is successful
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown server error".to_string());

        if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&error_text) {
            if let Some(error_msg) = error_json.get("error").and_then(|e| e.as_str()) {
                return Err(format!("Server error ({}): {}", status, error_msg));
            } else if let Some(message) = error_json.get("message").and_then(|m| m.as_str()) {
                return Err(format!("Server error ({}): {}", status, message));
            }
        }

        return Err(format!("Server error ({}): {}", status, error_text));
    }

    let prompts_response: PluelyPromptsResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse prompts response: {}", e))?;

    Ok(prompts_response)
}

// Create System Prompt API Command
#[tauri::command]
pub async fn create_system_prompt(
    app: AppHandle,
    user_prompt: String,
) -> Result<SystemPromptResponse, String> {
    // Get environment variables
    let app_endpoint = get_app_endpoint()?;
    let api_access_key = get_api_access_key()?;
    let (license_key, instance_id, _) = get_stored_credentials(&app).await?;
    let machine_id: String = app.machine_uid().get_machine_uid().unwrap().id.unwrap();
    let app_version: String = app.package_info().version.to_string();
    // Make HTTP request to models endpoint
    let client = reqwest::Client::new();
    let url = format!("{}/api/prompt", app_endpoint);

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_access_key))
        .header("license_key", &license_key)
        .header("instance", &instance_id)
        .header("machine_id", &machine_id)
        .header("app_version", &app_version)
        .json(&serde_json::json!({
            "user_prompt": user_prompt
        }))
        .send()
        .await
        .map_err(|e| {
            let error_msg = format!("{}", e);
            if error_msg.contains("url (") {
                // Remove the URL part from the error message
                let parts: Vec<&str> = error_msg.split(" for url (").collect();
                if parts.len() > 1 {
                    format!("Failed to make models request: {}", parts[0])
                } else {
                    format!("Failed to make models request: {}", error_msg)
                }
            } else {
                format!("Failed to make models request: {}", error_msg)
            }
        })?;

    // Check if the response is successful
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown server error".to_string());

        // Try to parse error as JSON to get a more specific error message
        if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&error_text) {
            if let Some(error_msg) = error_json.get("error").and_then(|e| e.as_str()) {
                return Err(format!("Server error ({}): {}", status, error_msg));
            } else if let Some(message) = error_json.get("message").and_then(|m| m.as_str()) {
                return Err(format!("Server error ({}): {}", status, message));
            }
        }

        return Err(format!("Server error ({}): {}", status, error_text));
    }

    let system_prompt_response: SystemPromptResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse system prompt response: {}", e))?;

    Ok(system_prompt_response)
}

// Helper command to check if license is available
#[tauri::command]
pub async fn check_license_status(app: AppHandle) -> Result<bool, String> {
    match get_stored_credentials(&app).await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[allow(dead_code)]
#[tauri::command]
pub async fn get_activity(app: AppHandle) -> Result<serde_json::Value, String> {
    let app_endpoint = get_app_endpoint()?;
    let api_access_key = get_api_access_key()?;

    let (license_key, instance_id, _) = get_stored_credentials(&app).await?;

    let machine_id = match app.machine_uid().get_machine_uid() {
        Ok(id) => id.id.unwrap_or_default(),
        Err(_) => String::new(),
    };

    if machine_id.is_empty() {
        return Err("Machine identifier unavailable".to_string());
    }

    let app_version = app.package_info().version.to_string();

    let client = reqwest::Client::new();
    let activity_url = format!("{}/api/activity", app_endpoint.trim_end_matches('/'));

    let response = client
        .get(&activity_url)
        .header("Authorization", format!("Bearer {}", api_access_key))
        .header("license_key", &license_key)
        .header("instance_name", &instance_id)
        .header("machine_id", machine_id)
        .header("app_version", app_version)
        .send()
        .await
        .map_err(|e| {
            let error_msg = format!("{}", e);
            if error_msg.contains("url (") {
                let parts: Vec<&str> = error_msg.split(" for url (").collect();
                if parts.len() > 1 {
                    format!("Failed to request activity: {}", parts[0])
                } else {
                    format!("Failed to request activity: {}", error_msg)
                }
            } else {
                format!("Failed to request activity: {}", error_msg)
            }
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown server error".to_string());

        if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&error_text) {
            if let Some(message) = error_json
                .get("message")
                .and_then(|m| m.as_str())
                .or_else(|| error_json.get("error").and_then(|m| m.as_str()))
            {
                return Err(format!("Server error ({}): {}", status, message));
            }
        }

        return Err(format!("Server error ({}): {}", status, error_text));
    }

    response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("Failed to parse activity response: {}", e))
}
