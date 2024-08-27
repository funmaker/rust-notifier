use serde::Deserialize;
use futures::future;
use anyhow::Result;
use thiserror::Error;

mod web;

use crate::utils::{Map, Json};
use crate::state::State;


#[derive(Deserialize)]
struct AnyInterfaceConfig {
	enabled: bool,
}

async fn serve_interface(name: String, config: Json, state: State) {
	match serde_json::from_value(config.clone()) {
		Ok(AnyInterfaceConfig{ enabled }) if !enabled => return,
		Err(err) => return eprintln!("Unable to serve {} interface: {}", name, err.to_string()),
		_ => {},
	}
	
	// Add new interfaces here
	let result = match &*name {
		"web" => web::serve(config, state).await,
		_ => Err(InterfaceNotFound.into()),
	};
	
	if let Err(err) = result {
		eprintln!("Unable to serve {} interface: {}", name, err.to_string());
	}
}

pub struct Interfaces {
	configs: Map<Json>,
}

impl Interfaces {
	pub fn new(configs: Map<Json>) -> Self {
		Interfaces{ configs }
	}
	
	pub async fn serve(&self, state: State) -> Result<()> {
		let providers = self.configs.iter()
		                            .map(|(name, config)| tokio::spawn(serve_interface(name.clone(), config.clone(), state.clone())))
		                            .collect::<Vec<_>>();
		
		future::try_join_all(providers).await?;
		
		Ok(())
	}
}

#[derive(Debug, Error)]
#[error("Interface not found")]
pub struct InterfaceNotFound;
