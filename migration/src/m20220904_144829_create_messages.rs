// Copyright 2022  Lucy <lucy@absolucy.moe>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		// Replace the sample below with your own migration scripts
		manager
			.create_table(
				Table::create()
					.table(Messages::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(Messages::Id)
							.uuid()
							.not_null()
							.primary_key()
							.unique_key(),
					)
					.col(ColumnDef::new(Messages::Channel).text().not_null())
					.col(ColumnDef::new(Messages::RoomId).big_unsigned().not_null())
					.col(ColumnDef::new(Messages::UserId).big_unsigned().not_null())
					.col(ColumnDef::new(Messages::Username).text().not_null())
					.col(ColumnDef::new(Messages::Message).text().not_null())
					.col(ColumnDef::new(Messages::Timestamp).timestamp().not_null())
					.col(
						ColumnDef::new(Messages::Deleted)
							.boolean()
							.not_null()
							.default(false),
					)
					.col(ColumnDef::new(Messages::DeletedAt).timestamp())
					.col(ColumnDef::new(Messages::ReplyingTo).uuid())
					.col(
						ColumnDef::new(Messages::Subscriber)
							.boolean()
							.not_null()
							.default(false),
					)
					.col(
						ColumnDef::new(Messages::Moderator)
							.boolean()
							.not_null()
							.default(false),
					)
					.col(
						ColumnDef::new(Messages::Vip)
							.boolean()
							.not_null()
							.default(false),
					)
					.col(ColumnDef::new(Messages::Emotes).text())
					.col(ColumnDef::new(Messages::Badges).text())
					.col(ColumnDef::new(Messages::UserType).text())
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		// Replace the sample below with your own migration scripts
		manager
			.drop_table(Table::drop().table(Messages::Table).to_owned())
			.await
	}
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Messages {
	#[iden = "messages"]
	Table,
	Id,
	Channel,
	#[iden = "room-id"]
	RoomId,
	#[iden = "user-id"]
	UserId,
	Username,
	Message,
	Timestamp,
	Deleted,
	#[iden = "deleted-at"]
	DeletedAt,
	#[iden = "replying-to"]
	ReplyingTo,
	Subscriber,
	Moderator,
	Vip,
	Emotes,
	Badges,
	#[iden = "user-type"]
	UserType,
}
