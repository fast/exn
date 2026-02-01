// Copyright 2025 FastLabs Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Library Boundary Example - Flat Errors at Public API
//!
//! This example shows how a library can:
//! - Use `exn::Result` internally for context-rich errors.
//! - Downcast internal errors at the public API boundary to produce a flat, machine-friendly
//!   `LibError`.
//! - Return `Exn<LibError>` so context is preserved in frames while internal errors stay private.

use std::error::Error;

use derive_more::Display;
use exn::Exn;
use exn::Frame;
use exn::Result;
use exn::ResultExt;
use exn::bail;

fn main() {
    demo(429);
    eprintln!();
    demo(404);
}

fn demo(user_id: u64) {
    eprintln!("Start demo for user: {user_id}");

    let mut attempt = 0;
    loop {
        match library::fetch_profile(user_id) {
            Ok(profile) => {
                eprintln!("{}: {}", profile.user_id, profile.plan);
                return;
            }
            Err(err) => {
                // Retry for errors the library marks as retryable.
                if attempt < 3 && err.is_retryable() {
                    eprintln!("{}", err);
                    eprintln!("Retryable error, attempting retry #{}", attempt + 1);
                    eprintln!();
                    attempt += 1;
                    continue;
                }

                let action = match err.kind() {
                    library::LibErrorKind::NotFound => "Return 404",
                    library::LibErrorKind::RateLimited => "Retried too many times, aborting",
                    library::LibErrorKind::Internal => "Internal server error",
                };
                eprintln!("Action: {action}");
                eprintln!("Error: {err:?}");
                return;
            }
        }
    }
}

mod library {
    use super::*;

    #[derive(Debug)]
    pub struct Profile {
        pub user_id: u64,
        pub plan: String,
    }

    #[derive(Debug, Display, Clone, Copy, PartialEq, Eq)]
    pub enum LibErrorKind {
        NotFound,
        RateLimited,
        Internal,
    }

    #[derive(Debug, Display)]
    #[display("{kind}: {message}")]
    pub struct LibError {
        kind: LibErrorKind,
        message: String,
    }

    impl LibError {
        pub fn kind(&self) -> LibErrorKind {
            self.kind
        }

        pub fn is_retryable(&self) -> bool {
            matches!(self.kind, LibErrorKind::RateLimited)
        }

        fn not_found(resource: &'static str, id: u64) -> Self {
            Self {
                kind: LibErrorKind::NotFound,
                message: format!("{resource} {id} not found"),
            }
        }

        fn rate_limited() -> Self {
            Self {
                kind: LibErrorKind::RateLimited,
                message: "rate limited by upstream".to_string(),
            }
        }

        fn internal(message: impl Into<String>) -> Self {
            Self {
                kind: LibErrorKind::Internal,
                message: message.into(),
            }
        }
    }

    impl Error for LibError {}

    /// Public API: returns `Exn<LibError>` while keeping internal errors private.
    pub fn fetch_profile(user_id: u64) -> Result<Profile, LibError> {
        // Explicit boundary mapping: downcast internal errors into a flat LibError.
        service::fetch_profile(user_id).map_err(map_to_lib_error)
    }

    fn map_to_lib_error(err: Exn<service::ServiceError>) -> Exn<LibError> {
        let lib_error = if let Some(db_error) = find_error::<db::DbError>(&err) {
            match db_error {
                db::DbError::NotFound { user_id } => LibError::not_found("user", *user_id),
                db::DbError::ConnectionDropped => LibError::rate_limited(),
            }
        } else if let Some(http_error) = find_error::<http::HttpError>(&err) {
            match http_error {
                http::HttpError::RateLimited => LibError::rate_limited(),
                http::HttpError::Unavailable => LibError::internal("upstream service unavailable"),
            }
        } else {
            LibError::internal("unexpected library error")
        };

        // Context stays in frames; only LibError is public.
        err.raise(lib_error)
    }

    fn find_error<T: Error + 'static>(exn: &Exn<impl Error + Send + Sync>) -> Option<&T> {
        fn walk<T: Error + 'static>(frame: &Frame) -> Option<&T> {
            if let Some(err) = frame.error().downcast_ref::<T>() {
                return Some(err);
            }
            frame.children().iter().find_map(walk::<T>)
        }

        walk(exn.frame())
    }

    mod service {
        use super::*;

        pub fn fetch_profile(user_id: u64) -> Result<Profile, ServiceError> {
            let make_error = || ServiceError(format!("failed to fetch profile for user {user_id}"));

            let user = db::load_user(user_id).or_raise(make_error)?;
            let plan = http::fetch_plan(user.plan_id).or_raise(make_error)?;

            Ok(Profile {
                user_id: user.user_id,
                plan: plan.name,
            })
        }

        #[derive(Debug, Display)]
        #[display("{_0}")]
        pub struct ServiceError(String);
        impl Error for ServiceError {}
    }

    mod db {
        use super::*;

        pub fn load_user(user_id: u64) -> Result<UserRow, DbError> {
            match user_id {
                404 => bail!(DbError::NotFound { user_id }),
                500 => bail!(DbError::ConnectionDropped),
                _ => Ok(UserRow {
                    user_id,
                    plan_id: user_id,
                }),
            }
        }

        pub struct UserRow {
            pub user_id: u64,
            pub plan_id: u64,
        }

        #[derive(Debug, Display)]
        pub enum DbError {
            #[display("no row for user_id {user_id}")]
            NotFound { user_id: u64 },
            #[display("database connection dropped")]
            ConnectionDropped,
        }
        impl Error for DbError {}
    }

    mod http {
        use super::*;

        pub fn fetch_plan(plan_id: u64) -> Result<Plan, HttpError> {
            match plan_id {
                429 => bail!(HttpError::RateLimited),
                503 => bail!(HttpError::Unavailable),
                _ => Ok(Plan {
                    name: format!("plan-{plan_id}"),
                }),
            }
        }

        pub struct Plan {
            pub name: String,
        }

        #[derive(Debug, Display)]
        pub enum HttpError {
            #[display("HTTP 429: too many requests")]
            RateLimited,
            #[display("HTTP 503: service unavailable")]
            Unavailable,
        }
        impl Error for HttpError {}
    }
}

// Output when running `cargo run -p examples --example library-boundary`:
//
// Start demo for user: 429
// RateLimited: rate limited by upstream
// Retryable error, attempting retry #1
//
// RateLimited: rate limited by upstream
// Retryable error, attempting retry #2
//
// RateLimited: rate limited by upstream
// Retryable error, attempting retry #3
//
// Action: Retried too many times, aborting
// Error: RateLimited: rate limited by upstream, at examples/src/library-boundary.rs:149:13
// |
// |-> failed to fetch profile for user 429, at examples/src/library-boundary.rs:170:55
// |
// |-> HTTP 429: too many requests, at examples/src/library-boundary.rs:218:24
//
// Start demo for user: 404
// Action: Return 404
// Error: NotFound: user 404 not found, at examples/src/library-boundary.rs:149:13
// |
// |-> failed to fetch profile for user 404, at examples/src/library-boundary.rs:169:47
// |
// |-> no row for user_id 404, at examples/src/library-boundary.rs:189:24
