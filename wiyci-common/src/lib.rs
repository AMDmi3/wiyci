// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: GPL-3.0-or-later

#![cfg_attr(test, feature(coverage_attribute))]

pub mod db;
pub mod migrations;
pub mod models;

pub use migrations::*;
