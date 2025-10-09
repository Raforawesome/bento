# Foundry

A lightweight, secure authentication backend as a service (BaaS) built with Rust.

## Overview

Foundry provides a robust API for user authentication, session management, and access control. It's designed to be integrated into your application stack with minimal setup, offering secure authentication primitives out of the box.

## Key Features

- **User Registration & Authentication**: Secure account creation and login flows
- **Session Management**: Token-based sessions with automatic expiration
- **Role-Based Access Control**: Simple user/admin permission system
- **IP Tracking**: Records client IPs for security auditing

## Why Rust?

Foundry leverages Rust's unique advantages to deliver a service that is:

- **Memory Safe**: Built on Rust's ownership model to eliminate common security vulnerabilities
- **Concurrency Without Overhead**: Uses async/await with Tokio for efficient handling of concurrent requests
- **Predictable Performance**: Achieves consistent, low-latency response times even under heavy load

The combination of Axum's routing system and Tokio's runtime provides excellent throughput while maintaining type safety across asynchronous boundaries - ensuring both correctness and performance.

## Getting Started

```sh
# Run the server (SSR)
cargo run --features ssr

# Server will start on 0.0.0.0:8000
```

## API Endpoints

- `POST /api/v1/register` - Create a new user account
- `POST /api/v1/login` - Authenticate and receive a session token

## Storage Options

Foundry currently supports an in-memory authentication store with thread-safe access patterns. Database integrations are planned for future releases.

## License

This code is intentionally provided without a license. This is not a mistake, this is not free use.
