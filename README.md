# Gitkban
A Rust application for automatically updating new pull requests that lack a body by associating them with an equivalent Kanbanize ticket. This project helps streamline the process of managing pull requests and Kanbanize tickets by inserting the Kanbanize ticket's body into the pull request description.

## Features
- Automatically detects and updates pull requests with missing body descriptions.
- Links pull requests with their corresponding Kanbanize tickets using branch numbers.
- Inserts the Kanbanize ticket's body content into the pull request description.

## Prerequisites
Before you get started with this application, make sure you have the following prerequisites in place:
- Cargo: You need to have cargo (Rust) installed.

## Installation
1. You can install this application directly from the command line
```bash
cargo install --git https://github.com/fdns/gitkban
```
2. You need to set the following environment variables:
```bash
KANBANIZE_BASE_PATH
KANBANIZE_API_KEY
GITHUB_PERSONAL_TOKEN
GITHUB_TRACK_USER
GITHUB_OWNER_FILTER
```