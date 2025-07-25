use std::env;

// Enforces that only one version feature can be enabled at a time.
fn main() {
    let features: Vec<String> = env::vars()
        .filter_map(|(key, _)| {
            if let Some(stripped) = key.strip_prefix("CARGO_FEATURE_") {
                Some(stripped.to_lowercase().replace('_', "-"))
            } else {
                None
            }
        })
        .filter(|feature| { feature.starts_with("v-")})
        .collect();

    if features.len() > 1 {
        panic!(
            "Only one feature can be enabled at a time. Enabled features: {:?}",
            features
        );
    }
    if features.len() == 0 {
        panic!(
            "At least one version feature must be enabled."
        );
    }
}