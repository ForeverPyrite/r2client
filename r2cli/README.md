# r2cli

A rudimentary CLI tool for Cloudflare R2-compatible S3 storage, built on the `r2client` Rust library.

## Features
- Upload files to an R2 bucket
- Download files from an R2 bucket
- List files and folders in an R2 bucket
- Delete a file in an R2 bucket

## Usage

Set the following environment variables:
- `R2_ACCESS_KEY`
- `R2_SECRET_KEY`
- `R2_ENDPOINT`
- `R2_BUCKET`

For ease of use within development environments, environment variables will also be sourced from `.env`

### Example commands

```sh
r2cli upload <local_file> <remote_key>
r2cli download <remote_key> <local_file>
r2cli list-files
r2cli list-folders
```

## Requirements
- Rust
- Valid Cloudflare R2 credentials

## Todo
- [ ] If you REALLY feel goofy, a TUI would be pretty sick
