# Use the latest version of the Rust base image
FROM rust:latest

# Set the working directory in the container to /my
WORKDIR /www/

# Copy the Rust project files to the working directory
COPY . .

# Build the Rust app
RUN cargo build

COPY ./src ./src
# Set the command to run the Rust app
CMD cargo run

