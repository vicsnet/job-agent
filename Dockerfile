FROM rust:latest AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && echo "fn main() {}" > src/main.rs 
RUN cargo build --release
RUN rm -rf src

COPY src ./src

RUN touch src/main.rs
RUN cargo build --release


FROM ubuntu:24.04 AS runtime
RUN apt-get update && apt-get install -y \
    libpq-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/job-agent .

EXPOSE 8080
CMD ["./job-agent"]