use serde::Deserialize;
use futures::future;
use tokio::task::JoinError;
use err_derive::Error;

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
		"web" => web::serve(config, state).await.map_err(Into::into),
		_ => Err(InterfaceError::NotFound),
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
	
	pub async fn serve(&self, state: State) -> Result<(), JoinError> {
		let providers = self.configs.iter()
		                            .map(|(name, config)| tokio::spawn(serve_interface(name.clone(), config.clone(), state.clone())))
		                            .collect::<Vec<_>>();
		
		future::try_join_all(providers).await?;
		
		Ok(())
	}
}

#[derive(Debug, Error)]
pub enum InterfaceError {
	#[error(display = "Interface not found")] NotFound,
	#[error(display = "Interface failed to serve: {}", _0)] ServeError(Box<dyn std::error::Error>),
}

impl From<Box<dyn std::error::Error>> for InterfaceError {
	fn from(err: Box<dyn std::error::Error>) -> Self { InterfaceError::ServeError(err) }
}
