// Copyright 2022  Lucy <lucy@absolucy.moe>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[macro_use]
extern crate log;

pub mod config;
pub mod process;
pub mod rollup;
pub mod token;

use async_signals::Signals;
use color_eyre::eyre::{Result, WrapErr};
use futures_util::StreamExt;
use irc::{
	client::{data::config::Config, prelude::Capability, Client},
	proto::Command,
};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

#[tokio::main]
async fn main() -> Result<()> {
	color_eyre::install()?;
	pretty_env_logger::init();

	let config = Arc::new(
		ron::from_str::<config::Config>(
			&tokio::fs::read_to_string("config.ron")
				.await
				.wrap_err("failed to read config.ron")?,
		)
		.wrap_err("failed to parse config.ron")?,
	);

	let token = token::get_token(&config.twitch)
		.await
		.wrap_err("failed to get twitch token to log in with")?;

	let mut sql_config = ConnectOptions::new(config.database.clone());
	sql_config
		.sqlx_logging(true)
		.sqlx_logging_level(log::LevelFilter::Trace);
	let db = Database::connect(sql_config).await?;
	Migrator::up(&db, None).await?;

	let parent_cancel_token = CancellationToken::new();
	let cancel_token = parent_cancel_token.child_token();

	let (rollup_tx, rollup_rx) = mpsc::unbounded_channel();
	tokio::spawn(rollup::rollup_task(db.clone(), config.clone(), rollup_rx));

	let mut signals = Signals::new(vec![libc::SIGUSR1, libc::SIGTERM])
		.wrap_err("failed to attach to SIGTERM and SIGUSR1")?;
	tokio::spawn(async move {
		while let Some(signal) = signals.next().await {
			match signal {
				libc::SIGUSR1 => rollup_tx.send(()).expect("failed to start new rollup"),
				libc::SIGTERM => {
					parent_cancel_token.cancel();
					break;
				}
				_ => panic!("got unexpected signal {}", signal),
			}
		}
	});

	let irc_config = Config {
		server: Some("irc.chat.twitch.tv".to_string()),
		port: Some(6697),
		use_tls: Some(true),
		..Config::default()
	};
	let mut irc_client = Client::from_config(irc_config)
		.await
		.wrap_err("failed to start Twitch IRC client")?;

	// Request the tags capability
	irc_client
		.send_cap_req(&[
			Capability::Custom("twitch.tv/tags"),
			Capability::Custom("twitch.tv/commands"),
		])
		.wrap_err("failed to request tags capability")?;
	// Send our password
	irc_client
		.send(Command::PASS(format!("oauth:{}", token)))
		.wrap_err("failed to send password")?;
	// Send our username
	irc_client
		.send(Command::NICK(config.twitch.username.to_lowercase()))
		.wrap_err("failed to send username")?;
	// Join the channel
	for channel in &config.twitch.channels {
		irc_client
			.send_join(format!("#{}", channel.to_lowercase()))
			.wrap_err_with(|| format!("failed to join {}", channel))?;
	}

	let mut stream = irc_client
		.stream()
		.wrap_err("failed to get stream of Twitch IRC")?;

	let message_tx = process::spawn_message_processor(db);

	loop {
		while let Some(message) = tokio::select! {
			msg = stream.next() => {
				msg.transpose().wrap_err("failed to get next IRC message")?
			}
			_ = cancel_token.cancelled() => {
				info!("exiting");
				return Ok(());
			}
		} {
			message_tx
				.send(message)
				.wrap_err("failed to send message")?;
		}
	}
}
