// SPDX-FileCopyrightText: Copyright 2026 Dmitry Marakasov <amdmi3@amdmi3.ru>
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![cfg_attr(test, feature(coverage_attribute))]

pub mod api;
pub mod db;
pub mod migrations;
pub mod models;

pub use migrations::*;
