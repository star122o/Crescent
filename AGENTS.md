# Crescent Project Documentation

Welcome! This is a monorepo workspace containing the core engine and its application bindings (desktop and mobile). This document provides an architectural overview and guidelines for development.

## Monorepo Architecture

The workspace is managed as a monorepo using **pnpm workspaces** for Node/JS components and **Cargo workspaces** for the Rust components.

```
crescent/
├── core/                # Core library (Rust)
├── apps/                # Application bindings (to be added)
│   ├── desktop/         # Electron desktop app
│   └── mobile/          # Expo mobile app
├── Cargo.toml           # Cargo workspace config
├── package.json         # pnpm workspace configuration
└── pnpm-workspace.yaml  # pnpm workspace definition
```

---

## Components

### 1. Core (`core/`)
- **Language**: Rust
- **Toolchain**: **Nightly Rust required** (due to `azalea` Minecraft framework and SIMD dependencies).
- **Role**: Implements the main Minecraft bot logic and behaviors using the `azalea` framework.
- **Features**:
  - `node-binding`: Enables `napi` and `napi-derive` dependencies to build bindings for the Node.js / Electron desktop application.
  - `native-binding`: Enables `uniffi` dependency to build bindings for the Expo mobile application.
- **Build / Check Commands**:
  - Check (default): `cargo check`
  - Check Node Bindings: `cargo +nightly check --features node-binding`
  - Check Native Bindings: `cargo +nightly check --features native-binding`

### 2. Desktop (`apps/desktop/`)
- **Environment**: Node.js, TypeScript, `pnpm`
- **Frontend Stack**: React, Vite, TailwindCSS
- **Shell**: Electron
- **Integration**: Accesses the core Rust engine through native Node.js addons built via **`napi`** (leveraging the `node-binding` feature of the `core` library).

### 3. Mobile (`apps/mobile/`)
- **Environment**: Node.js, TypeScript, `pnpm`
- **Framework**: Expo / React Native
- **Integration**: Accesses the core Rust engine through foreign-language interface (FFI) bindings generated via **`uniffi`** (leveraging the `native-binding` feature of the `core` library).

---

## Guidelines for AI Agents

- **Toolchain Override**: Always verify commands target `nightly` when compiling/checking the `core` with `azalea` feature enabled (e.g., `cargo +nightly check`).
- **Feature Flags**: When adding core features or Minecraft bot logic, ensure they are conditionally compiled behind `#[cfg(feature = "azalea")]` to keep default binding builds lightweight and clean.
- **Error Handling**: Custom errors should implement `std::error::Error` and be mapped appropriately for UniFFI (`derive(uniffi::Error)`) and Napi (`From<Error> for napi::Error`).
