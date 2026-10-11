//! Integration tests: the real axum router against a real PostgreSQL
//! (Testcontainers; one container per test run, a fresh database per test).
//! Requires Docker; on Colima run with
//! `DOCKER_HOST=unix://$HOME/.colima/default/docker.sock`.

mod common;

mod account;
mod auth;
mod cms;
mod counters;
mod forum;
mod http;
mod moderation;
mod permissions;
mod revisions;
mod seed;
mod sessions;
