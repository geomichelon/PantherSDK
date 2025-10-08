//! FastAPI app (pointer module)
//! The FastAPI application lives in the Python package at `python/panthersdk`.
//! This module documents the entrypoint for reference.

/// Path to the FastAPI app factory used by Uvicorn: `panthersdk.api:create_app`.
pub const FASTAPI_ENTRYPOINT: &str = "panthersdk.api:create_app";

