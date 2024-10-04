@echo off
git pull
cargo build --release
start C:\Users\lelih\rust\audiocloud_desktop\target\release\audiocloud_desktop.exe
