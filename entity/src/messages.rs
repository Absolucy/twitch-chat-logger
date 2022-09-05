// Copyright 2022  Lucy <lucy@absolucy.moe>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "messages")]
pub struct Model {
	#[sea_orm(primary_key, auto_increment = false)]
	pub id: Uuid,
	#[sea_orm(column_type = "Text")]
	pub channel: String,
	#[sea_orm(column_name = "room-id")]
	pub room_id: i64,
	#[sea_orm(column_name = "user-id")]
	pub user_id: i64,
	#[sea_orm(column_type = "Text")]
	pub username: String,
	#[sea_orm(column_type = "Text")]
	pub message: String,
	pub timestamp: TimeDateTime,
	pub deleted: bool,
	#[sea_orm(column_name = "deleted-at")]
	pub deleted_at: Option<TimeDateTime>,
	#[sea_orm(column_name = "replying-to")]
	pub replying_to: Option<Uuid>,
	pub subscriber: bool,
	pub moderator: bool,
	pub vip: bool,
	#[sea_orm(column_type = "Text", nullable)]
	pub emotes: Option<String>,
	#[sea_orm(column_type = "Text", nullable)]
	pub badges: Option<String>,
	#[sea_orm(column_name = "user-type", column_type = "Text", nullable)]
	pub user_type: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
	fn def(&self) -> RelationDef {
		panic!("No RelationDef")
	}
}

impl ActiveModelBehavior for ActiveModel {}
