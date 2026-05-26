use base64::Engine;
use base64::engine::general_purpose::STANDARD;

pub async fn load_image_src(api: &wolf_api::WolfApi, path: &str) -> Option<String> {
    let path = path.trim();
    if path.is_empty() {
        return None;
    }

    if is_absolute_url(path) {
        return Some(path.to_string());
    }

    api.utils()
        .icon(path)
        .await
        .ok()
        .map(|bytes| format!("data:image/png;base64,{}", STANDARD.encode(bytes)))
}

fn is_absolute_url(value: &str) -> bool {
    value.starts_with("http://") || value.starts_with("https://")
}
