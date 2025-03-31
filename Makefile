BUILD_DIR ?= $(SRC_ROOT)/build
OS := $(shell uname | tr '[:upper:]' '[:lower:]')
ARCH := $(shell uname -m)
BUILD_IN_TILT := false

# set ARCH var based on output
ifeq ($(ARCH),x86_64)
	ARCH = amd64
endif
ifeq ($(ARCH),aarch64)
	ARCH = arm64
endif

.DEFAULT_GOAL := build

.PHONY: build
build:
	cargo build

.PHONY: run
run:
	cargo run

.PHONY: template-env
template-env:
	echo "Creating .env file from .env.example"
	cp .env.example .env
	echo "Done... Update your .env now with the correct values"

