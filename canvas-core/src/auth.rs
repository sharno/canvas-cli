use serde_json::Value;

use crate::{CanvasClient, CanvasConfig, CanvasError};

pub fn auth_check(config: &CanvasConfig) -> Result<(), CanvasError> {    
    let client = CanvasClient::new(config)?;
    let _: Value = client.get_json("/users/self")?;
    Ok(())
}
