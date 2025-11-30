dev:
    cargo watch -x run
    wait

log:
    tail -f ~/.local/share/yap/yap.log

dev-log:
    tmux new-session -d -s yap 'just log' \; split-window -h 'just dev' \; attach