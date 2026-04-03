#!/bin/bash

TCP_HOST="127.0.0.1"
TCP_PORT="8888"
UDP_HOST="127.0.0.1"
UDP_PORT="7777"

GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

tcp_cmd() {
    echo "$1" | nc -w2 $TCP_HOST $TCP_PORT
}

udp_cmd() {
    echo "$1" | nc -u -w1 $UDP_HOST $UDP_PORT
}

log() {
    echo -e "[${2:-$NC}$1${NC}] $3"
}

# --- Writers ---

tcp_writer() {
    local id=$1
    log "TCP-WRITE-$id" "$BLUE" "Starting..."
    for i in $(seq 1 5); do
        local key="tcp_key_${id}_${i}"
        local val="tcp_val_${id}_${i}"
        resp=$(tcp_cmd "SET $key $val")
        log "TCP-WRITE-$id" "$BLUE" "SET $key = $val → $resp"
        sleep 0.1
    done
}

udp_writer() {
    local id=$1
    log "UDP-WRITE-$id" "$YELLOW" "Starting..."
    for i in $(seq 1 5); do
        local key="udp_key_${id}_${i}"
        local val="udp_val_${id}_${i}"
        resp=$(udp_cmd "SET $key $val")
        log "UDP-WRITE-$id" "$YELLOW" "SET $key = $val → $resp"
        sleep 0.1
    done
}

# --- Readers ---

tcp_reader() {
    local id=$1
    log "TCP-READ-$id" "$GREEN" "Starting..."
    for i in $(seq 1 5); do
        # Read keys written by tcp_writer 1 (may not exist yet — tests missing too)
        local key="tcp_key_1_${i}"
        resp=$(tcp_cmd "GET $key")
        log "TCP-READ-$id" "$GREEN" "GET $key → $resp"
        sleep 0.15
    done
}

udp_reader() {
    local id=$1
    log "UDP-READ-$id" "$RED" "Starting..."
    for i in $(seq 1 5); do
        local key="udp_key_1_${i}"
        resp=$(udp_cmd "GET $key")
        log "UDP-READ-$id" "$RED" "GET $key → $resp"
        sleep 0.15
    done
}

# --- Cross-protocol reader (written by TCP, read by UDP and vice versa) ---

cross_reader() {
    log "CROSS" "$YELLOW" "Starting cross-protocol reads..."
    sleep 0.3  # Let writers get ahead first
    for i in $(seq 1 5); do
        tcp_resp=$(tcp_cmd "GET udp_key_1_${i}")
        udp_resp=$(udp_cmd "GET tcp_key_1_${i}")
        log "CROSS" "$YELLOW" "TCP reads UDP key udp_key_1_${i} → $tcp_resp"
        log "CROSS" "$YELLOW" "UDP reads TCP key tcp_key_1_${i} → $udp_resp"
        sleep 0.1
    done
}

ping_loop() {
    log "PING" "$NC" "Starting ping loop..."
    for _ in $(seq 1 5); do
        tcp_r=$(tcp_cmd "PING")
        udp_r=$(udp_cmd "PING")
        log "PING" "$NC" "TCP → $tcp_r | UDP → $udp_r"
        sleep 0.2
    done
}

# --- Run everything concurrently ---

echo "========================================"
echo "  Starting concurrent read/write test"
echo "========================================"

tcp_writer 1 &
tcp_writer 2 &
tcp_writer 3 &

udp_writer 1 &
udp_writer 2 &
udp_writer 3 &

tcp_reader 1 &
tcp_reader 2 &
tcp_reader 3 &

udp_reader 1 &
udp_reader 2 &
udp_reader 3 &

cross_reader &
ping_loop &

wait

echo ""
echo "========================================"
echo "  All done. Final state spot check:"
echo "========================================"

for i in $(seq 1 5); do
    echo "tcp_key_1_${i} → $(tcp_cmd "GET tcp_key_1_${i}")"
    echo "udp_key_1_${i} → $(udp_cmd "GET udp_key_1_${i}")"
done