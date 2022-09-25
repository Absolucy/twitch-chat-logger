// Copyright 2022  Lucy <lucy@absolucy.moe>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
	config::Config,
	error::{Error, Result},
	rollup::{format_message, MAX_MESSAGES_PER_PAGE},
};
use async_stream::try_stream;
use axum::{
	body::StreamBody,
	extract::{Path, State},
	http::StatusCode,
	response::IntoResponse,
	routing::get,
	Router,
};
use axum_extra::extract::Query;
use entity::messages::{Column as MessageColumn, Entity as MessageEntity};
use futures_util::Stream;
use sea_orm::{prelude::*, DatabaseConnection, EntityTrait, QueryOrder, QuerySelect};
use serde::Deserialize;
use std::{
	net::{Ipv4Addr, SocketAddr},
	sync::Arc,
};
use time::{
	format_description::well_known::{Rfc2822, Rfc3339},
	OffsetDateTime, PrimitiveDateTime, UtcOffset,
};
use tokio_util::sync::CancellationToken;

pub const MAX_MESSAGES_TO_READ: u64 = 1_000_000;

#[derive(Deserialize)]
struct QueryParams {
	#[serde(
		default,
		alias = "user",
		alias = "username",
		alias = "usernames",
		alias = "name",
		alias = "names"
	)]
	users: Vec<String>,
	#[serde(rename = "start-time", alias = "start", alias = "from")]
	start_time: Option<String>,
	#[serde(rename = "end-time", alias = "end", alias = "to")]
	end_time: Option<String>,
}

fn convert_query_to_datetime(timestamp: Option<&str>) -> Option<PrimitiveDateTime> {
	timestamp.and_then(|timestamp| {
		timestamp
			.parse::<i64>()
			.ok()
			.map(|timestamp| {
				let dt = OffsetDateTime::from_unix_timestamp(timestamp).expect("bad time");
				PrimitiveDateTime::new(dt.date(), dt.time())
			})
			.or_else(|| {
				OffsetDateTime::parse(timestamp, &Rfc3339)
					.ok()
					.map(|dt| dt.to_offset(UtcOffset::UTC))
					.map(|dt| PrimitiveDateTime::new(dt.date(), dt.time()))
			})
			.or_else(|| {
				OffsetDateTime::parse(timestamp, &Rfc2822)
					.ok()
					.map(|dt| dt.to_offset(UtcOffset::UTC))
					.map(|dt| PrimitiveDateTime::new(dt.date(), dt.time()))
			})
	})
}

fn response_stream(
	db: DatabaseConnection,
	query: Select<MessageEntity>,
) -> impl Stream<Item = Result<String>> {
	try_stream! {
		let mut message_pages = query.paginate(&db, MAX_MESSAGES_PER_PAGE);
		while let Some(messages) = message_pages.fetch_and_next().await.map_err(Error::from)?{
			for message in messages {
				yield format_message(&message);
			}
		}
	}
}

async fn search(
	State(db): State<DatabaseConnection>,
	Path(channel): Path<String>,
	Query(params): Query<QueryParams>,
) -> Result<impl IntoResponse> {
	let mut message_pages =
		MessageEntity::find().filter(MessageColumn::Channel.eq(channel.to_lowercase()));
	let mut user_query: Option<migration::SimpleExpr> = None;
	for user in params.users {
		let user = user.to_lowercase();
		user_query = match user_query {
			Some(user_query) => Some(user_query.or(MessageColumn::Username.eq(user))),
			None => Some(MessageColumn::Username.eq(user)),
		}
	}
	if let Some(user_query) = user_query {
		message_pages = message_pages.filter(user_query);
	}
	if let Some(start_time) = convert_query_to_datetime(params.start_time.as_deref()) {
		message_pages = message_pages.filter(MessageColumn::Timestamp.gte(start_time));
	}
	if let Some(end_time) = convert_query_to_datetime(params.end_time.as_deref()) {
		message_pages = message_pages.filter(MessageColumn::Timestamp.lte(end_time));
	}

	Ok((
		StatusCode::OK,
		StreamBody::new(response_stream(
			db,
			message_pages
				.order_by_desc(MessageColumn::Timestamp)
				.limit(MAX_MESSAGES_TO_READ)
				.order_by_asc(MessageColumn::Timestamp),
		)),
	))
}

pub async fn run_server(
	config: Arc<Config>,
	db: DatabaseConnection,
	cancel_token: CancellationToken,
) {
	let app = Router::with_state(db).route("/search/:channel", get(search));

	let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, config.port));
	info!("listening on {}", addr);
	axum::Server::bind(&addr)
		.serve(app.into_make_service())
		.with_graceful_shutdown(cancel_token.cancelled())
		.await
		.expect("failed to serve");
}
