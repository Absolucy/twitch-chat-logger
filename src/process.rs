// Copyright 2022  Lucy <lucy@absolucy.moe>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use entity::messages::{
	ActiveModel as MessageActiveModel, Column as MessageColumn, Entity as MessageEntity,
};
use irc::proto::{message::Tag, Command, Message};
use sea_orm::{prelude::*, ActiveValue::Set, DatabaseConnection, EntityTrait, Unchanged};
use std::collections::HashMap;
use time::{Duration, OffsetDateTime, PrimitiveDateTime};
use tokio::sync::mpsc;
use uuid::Uuid;

pub fn spawn_message_processor(db: DatabaseConnection) -> mpsc::UnboundedSender<Message> {
	let (tx, rx) = mpsc::unbounded_channel();
	tokio::spawn(message_processor(rx, db));
	tx
}

async fn message_processor(mut rx: mpsc::UnboundedReceiver<Message>, db: DatabaseConnection) {
	while let Some(message) = rx.recv().await {
		debug!("{:?}", message);
		let tags = message
			.tags
			.clone()
			.unwrap_or_default()
			.into_iter()
			.map(|Tag(key, value)| (key, value))
			.collect::<std::collections::HashMap<_, _>>();
		match &message.command {
			Command::NOTICE(target, msg) => {
				handle_notice(target, msg).await;
			}
			Command::PRIVMSG(channel, msg) => {
				let username = match message.source_nickname() {
					Some(username) => username,
					None => continue,
				};
				let channel = channel.strip_prefix('#').unwrap_or(channel.as_str());
				handle_privmsg(&db, channel, username, msg, tags).await;
			}
			Command::Raw(command, value) => match command.as_str() {
				"CLEARMSG" => {
					handle_clearmsg(&db, tags).await;
				}
				_ => {
					debug!("Unhandled message: [{}] {:?}", command, value);
					continue;
				}
			},
			_ => continue,
		}
	}
}

async fn handle_notice(_target: &str, msg: &str) {
	if msg
		.trim()
		.eq_ignore_ascii_case("login authentication failed")
		|| msg.trim().eq_ignore_ascii_case("login unsuccessful failed")
	{
		error!("Twitch authentication failed");
	} else if msg.trim().eq_ignore_ascii_case("improperly formatted auth") {
		error!("failed to log in. this is weird");
	}
}

async fn handle_privmsg(
	db: &DatabaseConnection,
	channel: &str,
	username: &str,
	msg: &str,
	tags: HashMap<String, Option<String>>,
) {
	let id = match tags.get("id") {
		Some(Some(id)) => Uuid::parse_str(id).unwrap(),
		_ => {
			warn!("message was missing ID tag");
			return;
		}
	};
	let room_id = match tags.get("room-id") {
		Some(Some(id)) => id.parse::<i64>().unwrap(),
		_ => {
			warn!("message was missing room ID tag");
			return;
		}
	};
	let user_id = match tags.get("user-id") {
		Some(Some(id)) => id.parse::<i64>().unwrap(),
		_ => {
			warn!("message was missing user ID tag");
			return;
		}
	};
	let timestamp = match tags.get("tmi-sent-ts") {
		Some(Some(timestamp)) => {
			let timestamp = timestamp.parse::<i64>().expect("failed to parse timestamp");
			let date_time = OffsetDateTime::UNIX_EPOCH + Duration::milliseconds(timestamp);
			PrimitiveDateTime::new(date_time.date(), date_time.time())
		}
		_ => {
			warn!("message {} was missing timestamp tag", id);
			return;
		}
	};
	let replying_to = match tags.get("reply-parent-msg-id") {
		Some(Some(id)) => Some(Uuid::parse_str(id).unwrap()),
		_ => None,
	};
	debug!("[#{}] {}: {}", channel, username, msg);
	let model = MessageActiveModel {
		id: Set(id),
		channel: Set(channel.to_string()),
		room_id: Set(room_id),
		user_id: Set(user_id),
		username: Set(username.to_string()),
		message: Set(msg.to_string()),
		timestamp: Set(timestamp),
		replying_to: Set(replying_to),
		subscriber: Set(tags.contains_key("subscriber")),
		moderator: Set(tags.contains_key("mod")),
		vip: Set(tags.contains_key("vip")),
		emotes: Set(tags.get("emotes").cloned().flatten()),
		badges: Set(tags.get("badges").cloned().flatten()),
		user_type: Set(tags.get("user-type").cloned().flatten()),
		..Default::default()
	};
	model
		.insert(db)
		.await
		.expect("failed to insert message into database");
}

async fn handle_clearmsg(db: &DatabaseConnection, tags: HashMap<String, Option<String>>) {
	let timestamp = match tags.get("tmi-sent-ts") {
		Some(Some(timestamp)) => {
			let timestamp = timestamp.parse::<i64>().expect("failed to parse timestamp");
			let date_time = OffsetDateTime::UNIX_EPOCH + Duration::milliseconds(timestamp);
			PrimitiveDateTime::new(date_time.date(), date_time.time())
		}
		_ => {
			warn!("CLEARMSG was missing timestamp tag");
			return;
		}
	};
	let target_msg_id = match tags.get("target-msg-id") {
		Some(Some(id)) => Uuid::parse_str(id).unwrap(),
		_ => {
			warn!("CLEARMSG was missing target id");
			return;
		}
	};
	let model = MessageActiveModel {
		id: Unchanged(target_msg_id),
		deleted: Set(true),
		deleted_at: Set(Some(timestamp)),
		..Default::default()
	};
	error!("Message deleted: {}", target_msg_id);
	MessageEntity::update(model)
		.exec(db)
		.await
		.expect("failed to update message");
	debug!("message {} was deleted", target_msg_id);
}
