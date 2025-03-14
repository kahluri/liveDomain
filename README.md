# liveDomain


A high-performance **Rust-based** domain checker that efficiently verifies if domains are **active** or **dead** using **async HTTP requests**.  

## Features  
✅ **Fast & Scalable** – Uses `tokio` for async processing.  
✅ **Batch Processing** – Limits concurrency to prevent system overload.  
✅ **Supports HTTP & HTTPS** – Checks both before marking a domain as dead.  
✅ **Logs Results** – Saves **live** and **dead** domains in `live.txt` and `dead.txt`.  

## Installation  
```sh
cargo build --release
```
```sh
./domain_checker -f subdomain.txt -v
```
Options :

-f <file> – Input file containing domains.

-v – Verbose mode.

-V – Show version.
