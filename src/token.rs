// Copyright 2022  Lucy <lucy@absolucy.moe>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::path::PathBuf;

use crate::config::TwitchConfig;
use color_eyre::eyre::{Result, WrapErr};
use serde::{Deserialize, Serialize};
use twitch_oauth2::{AccessToken, ClientSecret, RefreshToken, ValidatedToken};

#[derive(Serialize, Deserialize)]
struct TokenCache {
	base_access_token: String,
	current_access_token: String,
	current_refresh_token: String,
}

async fn get_token_from_cache(config: &TwitchConfig) -> Result<(String, String, String)> {
	let path = PathBuf::from(".refreshed-token.json");
	if !path.exists() {
		return Ok((
			config.access_token.clone(),
			config.access_token.clone(),
			config.refresh_token.clone(),
		));
	}
	let token_cache_file = tokio::fs::read_to_string(&path)
		.await
		.wrap_err("failed to read .refreshed-token.json")?;
	if token_cache_file.trim().is_empty() {
		return Ok((
			config.access_token.clone(),
			config.access_token.clone(),
			config.refresh_token.clone(),
		));
	}
	let token_cache = serde_json::from_str::<TokenCache>(&token_cache_file)
		.wrap_err("failed to parse .refreshed-token.json")?;
	if token_cache.base_access_token == config.access_token {
		Ok((
			config.access_token.clone(),
			token_cache.current_access_token,
			token_cache.current_refresh_token,
		))
	} else {
		Ok((
			config.access_token.clone(),
			config.access_token.clone(),
			config.refresh_token.clone(),
		))
	}
}

async fn auto_refresh_token(
	http_client: reqwest::Client,
	base_token: String,
	mut validated_token: ValidatedToken,
	mut refresh_token: RefreshToken,
	client_secret: ClientSecret,
) {
	let mut access_token: AccessToken;
	loop {
		let time_to_wait = validated_token.expires_in - (validated_token.expires_in / 5);
		info!("refreshing token in {} seconds", time_to_wait.as_secs());
		tokio::time::sleep(time_to_wait).await;
		info!("refreshing token NOW!");
		let (new_access_token, _, new_refresh_token) = twitch_oauth2::refresh_token(
			&http_client,
			&refresh_token,
			&validated_token.client_id,
			&client_secret,
		)
		.await
		.expect("failed to refresh token");
		access_token = new_access_token;
		refresh_token = new_refresh_token.expect("didn't get refresh token");
		validated_token = twitch_oauth2::validate_token(&http_client, &access_token)
			.await
			.expect("failed to validate newly refreshed token");
		tokio::fs::write(
			".refreshed-token.json",
			serde_json::to_string(&TokenCache {
				base_access_token: base_token.clone(),
				current_access_token: access_token.secret().to_string(),
				current_refresh_token: refresh_token.secret().to_string(),
			})
			.unwrap(),
		)
		.await
		.unwrap();
		info!("new token cached");
	}
}

pub async fn get_token(config: &TwitchConfig) -> Result<String> {
	let (original_access_token, access_token_string, refresh_token) = get_token_from_cache(config)
		.await
		.wrap_err("failed to get token from cache")?;
	let access_token = AccessToken::new(access_token_string.clone());
	let refresh_token = RefreshToken::new(refresh_token);
	let client_secret = ClientSecret::new(config.client_secret.clone());
	let http_client = reqwest::Client::builder()
		.default_headers({
			let mut headers = reqwest::header::HeaderMap::new();
			headers.insert(
				reqwest::header::USER_AGENT,
				format!("Absolucy/{}", env!("CARGO_PKG_VERSION"))
					.parse()
					.unwrap(),
			);
			headers
		})
		.build()
		.wrap_err("failed to build reqwest client")?;
	let validated_token = twitch_oauth2::validate_token(&http_client, &access_token)
		.await
		.wrap_err("failed to validate token")?;
	tokio::spawn(auto_refresh_token(
		http_client,
		original_access_token.clone(),
		validated_token,
		refresh_token,
		client_secret,
	));
	Ok(access_token_string)
}
