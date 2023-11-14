FROM ghcr.io/steamdeckhomebrew/holo-toolchain-rust:latest

RUN pacman -S --noconfirm cmake make clang git pkg-config libx11

ENTRYPOINT [ "/backend/entrypoint.sh" ]