# Use the official Alpine base image
FROM alpine:3.19.1

# Install necessary runtime packages
RUN apk add --no-cache \
    libgcc \
    openssl

# Create a new directory for the application
WORKDIR /app

# Copy the pre-built binary from the build stage
COPY liatrio-dora-api .

# Expose the port the app runs on
EXPOSE 3000

# Run the application
CMD ["./liatrio-dora-api"]
