#!/bin/bash

# Function to run the wrk command and handle interruptions
run_wrk() {
  while true; do
    # Generate a random sleep duration between 10ms and 400ms in milliseconds
    sleep_duration=$(awk -v min=10 -v max=400 'BEGIN{srand(); print int(min+rand()*(max-min+1))}')

    # Convert milliseconds to seconds for the sleep command
    sleep_duration_seconds=$(echo "scale=3; $sleep_duration / 1000" | bc)

    # Start the wrk command in the background
    wrk --timeout 10s -t 10 -c 18 -d 1000s -s request.lua http://localhost:3000 > /dev/null 2>&1 &

    # Get the PID of the background process
    wrk_pid=$!

    echo "New wrk command started with PID: $wrk_pid"

    # Sleep for the randomly generated duration in seconds
    sleep $sleep_duration_seconds

    # Kill the wrk process
    echo "Killing the wrk command (PID: $wrk_pid)"
    kill $wrk_pid
  done
}

# Run four instances of the run_wrk function in parallel
for _ in {1..4}; do
  run_wrk &
done

# Wait for Ctrl+C to exit the script
trap "echo 'Exiting...'; exit 0" SIGINT
wait
