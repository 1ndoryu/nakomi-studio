use super::entities::HostingEvent;

const HOSTING_EVENT_SECRET_KEYS: &[&str] = &["wp_admin_password", "sftp_password"];

#[must_use]
pub fn sanitize_hosting_event_details(
    details: Option<serde_json::Value>,
) -> Option<serde_json::Value> {
    details.map(|mut value| {
        redact_hosting_event_value(&mut value);
        value
    })
}

#[must_use]
pub fn sanitize_hosting_event(mut event: HostingEvent) -> HostingEvent {
    event.details = sanitize_hosting_event_details(event.details);
    event
}

fn redact_hosting_event_value(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for secret_key in HOSTING_EVENT_SECRET_KEYS {
                if map.contains_key(*secret_key) {
                    map.insert(
                        (*secret_key).to_string(),
                        serde_json::Value::String("[redacted]".to_string()),
                    );
                }
            }
            for nested_value in map.values_mut() {
                redact_hosting_event_value(nested_value);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                redact_hosting_event_value(item);
            }
        }
        _ => {}
    }
}
