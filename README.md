# SamaSama Pool

Trustless on-chain rotating savings and credit system (*paluwagan*) eliminating counterparty risks for community savers in the Philippines using Soroban smart contracts.

## Core Fundamentals
* **Problem:** Members of informal workplace savings circles face severe losses when early-draw winners default or managers mismanage cash.
* **Solution:** Automated escrow tracking mechanisms remove intermediaries, enforcing locked programmatic rotation cycles and automated token disbursements.
* **Timeline:** 4-Week Collaborative Bootcamp Prototype.
* **Stellar Integration Vector:** High-performance Soroban state architecture, native asset payment tracks, and deterministic authentication criteria.

## Vision and Purpose
To formalize community financial traditions across Southeast Asia, helping unbanked and underbanked professionals build secure financial standing through transparent peer-to-peer mechanisms.

## Prerequisites
* **Rust Toolchain:** `rustc 1.75.0+`
* **Soroban CLI Version:** `21.0.0+`
* **Target Architecture Configuration:** `wasm32-unknown-unknown`

## Build Directions
Compile the production-grade WebAssembly smart contract artifact:
```bash
soroban contract build