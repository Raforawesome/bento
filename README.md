<div align="center">
  <img src="src/webui/public/bento-dark-64.webp" alt="Bento Logo" width="64" height="64">
</div>

# Bento

A lightweight, secure backend as a service (BaaS) built with Rust.

## Overview

Bento provides a robust API for user authentication, database features, file storage, and message passing. It's designed to be integrated into your application stack with minimal setup, offering secure and convenient authentication/storage/messaging primitives out of the box.

## Key Features

- **User Registration & Authentication**: Secure account creation and login flows
- **Session Management**: Sessions with unique IDs automatic expiration
- **Role-Based Access Control**: Simple user/admin permission system
- **Database Features (SQL & Non-SQL)**: Upcoming
- **File Storage**: Upcoming
- **Message Passing**: Upcoming

## Frontend & API

Bento ships with a Leptos frontend by default, providing a full-stack solution out of the box. Leptos is a Rust web framework
that offers fine-grained reactivity and server-side rendering, making for a performant and well-integrated default.

For developers who want to build custom frontends (React, Vue, mobile apps, etc.), the REST API can be exposed by enabling the `rest-api` feature flag:

```sh
cargo leptos watch --features rest-api
```
(or similar for other commands, such as `build`)

### API Endpoints (when `rest-api` feature is enabled)

- `POST /api/v1/register` - Create a new user account (needs admin privileges)
- `POST /api/v1/login` - Authenticate and receive a session token

## Getting Started

To run this server, either run the binary or download the source and run the following:
```sh
# Install cargo leptos if you haven't already
cargo install --locked cargo-leptos

# Run the server (SSR)
cargo leptos watch
```
This runs a dev server with hot reloading. To build an optimized production build, run:
```sh
cargo leptos build --release
```

## Tech Stack (Credits)

Bento is built in Rust. This is mostly because I simply prefer the language, but also 
comes with advantages such as performant, native code and memory/type safety.

This project would not be possible without the following awesome open-source projects:
- [The Rust Programming Language](https://www.rust-lang.org/)
- [Axum](https://github.com/tokio-rs/axum) (web server framework)
- [Leptos](https://leptos.dev/) (frontend framework)
- [Tailwind CSS](https://tailwindcss.com/) (self-explanatory?)
- [Lucide](https://lucide.dev/) (icons)
