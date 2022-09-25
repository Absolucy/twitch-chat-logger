// Copyright 2022  Lucy <lucy@absolucy.moe>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::config::Config;
use ahash::AHashMap;
use color_eyre::eyre::{Result, WrapErr};
use entity::messages::{Column as MessageColumn, Entity as MessageEntity, Model as Message};
use sea_orm::{prelude::*, DatabaseConnection, EntityTrait, QueryOrder};
use std::sync::Arc;
use time::{macros::format_description, Date, Duration, OffsetDateTime, PrimitiveDateTime, Time};
use tokio::{
	fs::File,
	io::{AsyncWriteExt, BufWriter},
	sync::mpsc,
};

/// Only read 128MB worth of messages at one time
pub const MAX_MESSAGES_PER_PAGE: usize = (128 * 1024 * 1024) / std::mem::size_of::<Message>();

fn get_text_log_file_name(channel: &str, dt: PrimitiveDateTime) -> Result<String> {
	Ok(format!(
		"{}_{}.txt",
		channel,
		dt.format(format_description!("[year]-[month]-[day]"))
			.wrap_err("failed to format time")?
	))
}

pub async fn rollup_task(
	db: DatabaseConnection,
	config: Arc<Config>,
	mut rx: mpsc::UnboundedReceiver<()>,
) {
	loop {
		let todays_date = OffsetDateTime::now_utc();
		let next_midnight = todays_date.replace_time(Time::MIDNIGHT) + Duration::days(1);
		let time_til_next_midnight = next_midnight - todays_date;

		info!(
			"Waiting {} seconds until next rollup",
			time_til_next_midnight.whole_seconds()
		);
		tokio::select! {
			_ = tokio::time::sleep(time_til_next_midnight.unsigned_abs()) => {},
			_ = rx.recv() => {},
		}
		info!(
			"Starting rollup for {}",
			todays_date
				.format(format_description!("[year]-[month]-[day]"))
				.unwrap()
		);
		if let Err(error) = rollup_everything(&db, &config, todays_date.date())
			.await
			.wrap_err_with(|| {
				format!(
					"rollup for {} failed",
					todays_date
						.format(format_description!("[year]-[month]-[day]"))
						.unwrap()
				)
			}) {
			error!("Rollup failed: {:?}", error);
		}
	}
}

pub fn format_message(message: &Message) -> String {
	if let Some(deleted_at) = message.deleted_at {
		format!(
			"[{}] <{}; deleted at {}> {}\n",
			message
				.timestamp
				.format(format_description!("[hour]:[minute]:[second]"))
				.expect("failed to format time"),
			message.username,
			deleted_at
				.format(format_description!("[hour]:[minute]:[second]"))
				.expect("failed to format time"),
			message.message
		)
	} else {
		format!(
			"[{}] <{}> {}\n",
			message
				.timestamp
				.format(format_description!("[hour]:[minute]:[second]"))
				.expect("failed to format time"),
			message.username,
			message.message
		)
	}
}

async fn rollup_everything(db: &DatabaseConnection, config: &Config, date: Date) -> Result<()> {
	let mut files = AHashMap::<String, BufWriter<File>>::new();
	let mut messages_saved = AHashMap::<String, usize>::new();
	let start_of_day = date.with_time(Time::MIDNIGHT);
	let end_of_day = start_of_day + Duration::days(1) - Duration::nanoseconds(1);

	let mut message_pages = MessageEntity::find()
		.filter(MessageColumn::Timestamp.between(start_of_day, end_of_day))
		.order_by_asc(MessageColumn::Timestamp)
		.paginate(db, MAX_MESSAGES_PER_PAGE);

	while let Some(messages) = message_pages
		.fetch_and_next()
		.await
		.wrap_err("failed to get messages")?
	{
		for message in messages {
			let file_name = get_text_log_file_name(&message.channel, message.timestamp)
				.wrap_err("failed to get text log file name for message")?;
			let file = files.entry(file_name.clone()).or_insert_with(|| {
				let file =
					std::fs::File::create(config.rollup_dir.join(&file_name).with_extension("log"))
						.expect("failed to create log file");
				BufWriter::new(File::from_std(file))
			});
			messages_saved
				.entry(message.channel.clone())
				.and_modify(|x| *x += 1)
				.or_insert(1);
			let chat_message = format_message(&message);
			file.write_all(&chat_message.into_bytes())
				.await
				.wrap_err("failed to write to log file")?;
		}
	}

	for (user, mut file) in files.drain() {
		file.flush()
			.await
			.wrap_err_with(|| format!("failed to flush buffer for log file for {}", user))?;
		file.into_inner()
			.sync_all()
			.await
			.wrap_err_with(|| format!("failed to sync log file for {}", user))?;
		info!("Rolled up user '{}' for {}", user, date);
	}

	for (user, messages) in messages_saved {
		info!("Rolled up {} messages for {}", messages, user);
	}

	Ok(())
}
