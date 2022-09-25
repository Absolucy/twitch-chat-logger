// Copyright 2022  Lucy <lucy@absolucy.moe>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct Config {
	pub twitch: TwitchConfig,
	pub database: String,
	pub rollup_dir: PathBuf,
	pub port: u16,
}

#[derive(Deserialize)]
pub struct TwitchConfig {
	pub username: String,
	pub access_token: String,
	pub refresh_token: String,
	pub client_id: String,
	pub client_secret: String,
	pub channels: Vec<String>,
}
