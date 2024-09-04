use std::env::set_var;
use dotenvy_macro::dotenv;

pub fn init() {
    #[cfg(target_os = "macos")]
    embed_plist::embed_info_plist!("Info.plist");

    set_var("AWS_ACCESS_KEY_ID", dotenv!("AWS_ACCESS_KEY_ID"));
    set_var("AWS_SECRET_ACCESS_KEY", dotenv!("AWS_SECRET_ACCESS_KEY"));
    set_var("AWS_REGION", dotenv!("AWS_REGION"));

    set_var("LIVEKIT_URL", dotenv!("LIVEKIT_URL"));
    set_var("LIVEKIT_API_KEY", dotenv!("LIVEKIT_API_KEY"));
    set_var("LIVEKIT_API_SECRET", dotenv!("LIVEKIT_API_SECRET"));
}
