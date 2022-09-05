// Copyright 2022  Lucy <lucy@absolucy.moe>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use sea_orm_migration::prelude::*;

#[async_std::main]
async fn main() {
	cli::run_cli(migration::Migrator).await;
}
