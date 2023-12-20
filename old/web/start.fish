#! /bin/env fish

wezterm start --cwd ./server -- watchexec -e rs -r -c cargo run &
wezterm start --cwd ./server -- podman-compose up &
wezterm start --cwd ./front -- trunk serve
