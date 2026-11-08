mod authors;
pub mod create;
mod files;
pub mod index;

use std::time::Duration;

const REGISTRY_URL: &str = "https://packages.typst.org";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);
const USER_AGENT: &str = "TypstDesktop (templates)";

fn http_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(REQUEST_TIMEOUT)
        .user_agent(USER_AGENT)
        .build()
}

fn unreachable_registry() -> String {
    "Could not reach Typst Universe. Check your connection and try again.".to_string()
}
