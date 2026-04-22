#!/usr/bin/env python3

import hexio_sdk
import os, subprocess, argparse

def c2_loop(c2Client: hexio_sdk.HexioClient):
    while True:
        checkin = c2Client.checkin()
        

def main():
    parser = argparse.ArgumentParser(description='Test Agent for Hexio SDK')
    parser.add_argument('--c2', required=True, help='C2 server address (e.g., http://localhost:8000)')
    parser.add_argument('--password', required=True, help='Password for authentication')
    args = parser.parse_args()
    c2_address = args.c2
    password = args.password
    
    c2Client = hexio_sdk.HexioClient(c2_address, password)
    registration = c2Client.register(
        hostname="TEST",
        ip="127.0.0.1",
        user="root",
        os_info="macOS 10.15.7",
        process="testagent.py",
        pid=os.getpid(),
        arch="arm64",
        client_type="pytest",
        sleep_time=5
    )
    c2_loop(c2Client)