alias help := default

default:
    @just --list

format:
    treefmt

format-check:
    @treefmt --fail-on-change

run:
    @cargo run

build:
    @cargo build --release
