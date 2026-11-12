# syntax=docker/dockerfile:1
#
# Production image for the API. Build from the backend directory:
#   docker build -t personal-management-api backend
#
# Stage 1 compiles a release binary; stage 2 ships only that binary on a minimal base image.

# --- Build stage -------------------------------------------------------------------------------
# Same Debian release as the runtime image, so the binary links against the same glibc.
FROM rust:1-bookworm AS build
WORKDIR /app

# Compile the `sqlx::query!` macros against the committed `.sqlx` metadata: there is no database
# during the image build.
ENV SQLX_OFFLINE=true

COPY Cargo.toml Cargo.lock build.rs ./
COPY .sqlx ./.sqlx
COPY migrations ./migrations
COPY src ./src

# Cache mounts keep downloaded crates and compiled dependencies between builds. The target
# directory is a cache mount, so the binary is copied out of it in the same step.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --locked \
    && cp target/release/personal-management-api /app/server

# --- Runtime stage -----------------------------------------------------------------------------
# Distroless: glibc and CA certificates (for TLS to the database and Google), no shell or package
# manager. The `nonroot` variant runs as an unprivileged user.
FROM gcr.io/distroless/cc-debian12:nonroot
WORKDIR /app

# Migrations are embedded in the binary by `sqlx::migrate!` and applied at start-up.
COPY --from=build /app/server /app/server

# The platform usually sets PORT; 8080 is the app's default and matches .env.example.
ENV PORT=8080
EXPOSE 8080

ENTRYPOINT ["/app/server"]
