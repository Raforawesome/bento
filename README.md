# Bento

A lightweight, secure authentication backend as a service (BaaS) built with Rust.

## Overview

Bento provides a robust API for user authentication, database features, file storage, and message passing. It's designed to be integrated into your application stack with minimal setup, offering secure authentication primitives out of the box.

## Key Features

- **User Registration & Authentication**: Secure account creation and login flows
- **Session Management**: Token-based sessions with automatic expiration
- **Role-Based Access Control**: Simple user/admin permission system
- **Database Features (SQL & Non-SQL)**: Upcoming
- **File Storage**: Upcoming
- **Message Passing**: Upcoming

## Why Rust?

Bento leverages Rust's unique advantages to deliver a service that is:

- **Memory Safe**: Built on Rust's ownership model to eliminate common security vulnerabilities
- **Concurrency Without Overhead**: Uses async/await with Tokio for efficient handling of concurrent requests
- **Predictable Performance**: Achieves consistent, low-latency response times even under heavy load

The combination of Axum's routing system and Tokio's runtime provides excellent throughput while maintaining type safety across asynchronous boundaries - ensuring both correctness and performance.

## Getting Started

To run this server, either run the binary or download the source and run the following:
```sh
# Install cargo leptos if you haven't already
cargo install --locked cargo-leptos

# Run the server (SSR)
cargo leptos watch
```

## API Endpoints

- `POST /api/v1/register` - Create a new user account
- `POST /api/v1/login` - Authenticate and receive a session token

## Storage Options
Bento currently supports an in-memory authentication store with thread-safe public methods. Database integrations are planned for future releases.
