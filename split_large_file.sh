#!/bin/bash

# Memory-efficient script to split large text files into 100MB chunks
# Uses dd command for streaming, minimal RAM usage
# Usage: ./split_large_file.sh <filename>

set -e  # Exit on any error

# Check if filename and chunk size are provided
if [ $# -lt 2 ]; then
    echo "Usage: $0 <filename> <chunk_size_mb>"
    echo "Example: $0 large_file.txt 9"
    echo "Example: $0 /path/to/large_file.txt 50"
    exit 1
fi

# Input file and chunk size
INPUT_FILE="$1"
CHUNK_SIZE_MB="$2"

# Validate chunk size parameter
if ! [[ "$CHUNK_SIZE_MB" =~ ^[0-9]+$ ]] || [ "$CHUNK_SIZE_MB" -lt 1 ]; then
    echo "Error: Chunk size must be a positive integer (MB)"
    echo "Example: $0 large_file.txt 9"
    exit 1
fi

# Get the directory of the input file
OUTPUT_DIR=$(dirname "$INPUT_FILE")

# Check if file exists
if [ ! -f "$INPUT_FILE" ]; then
    echo "Error: File '$INPUT_FILE' not found!"
    exit 1
fi

# Output directory is the same as input file directory

# Get file size
FILE_SIZE=$(stat -c%s "$INPUT_FILE")
FILE_SIZE_MB=$((FILE_SIZE / 1024 / 1024))

echo "File: $INPUT_FILE"
echo "Size: ${FILE_SIZE_MB}MB (${FILE_SIZE} bytes)"
echo "Output directory: $OUTPUT_DIR (same as input file)"
echo "Splitting into ${CHUNK_SIZE_MB}MB chunks using streaming method..."

# Extract filename components
FILENAME=$(basename "$INPUT_FILE")
BASENAME="${FILENAME%.*}"
EXTENSION="${FILENAME##*.}"

# Handle files without extensions
if [ "$BASENAME" = "$EXTENSION" ]; then
    EXTENSION=""
else
    EXTENSION=".$EXTENSION"
fi

# Generate timestamp
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

# Chunk size: parameter in MB converted to bytes
CHUNK_SIZE=$((CHUNK_SIZE_MB * 1024 * 1024))

# Calculate number of chunks needed
TOTAL_CHUNKS=$(((FILE_SIZE + CHUNK_SIZE - 1) / CHUNK_SIZE))

echo "Will create approximately $TOTAL_CHUNKS chunks"
echo ""

# Process file in chunks using dd
CHUNK=1
BYTES_READ=0

while [ $BYTES_READ -lt $FILE_SIZE ]; do
    # Calculate remaining bytes
    REMAINING=$((FILE_SIZE - BYTES_READ))

    # Use smaller of chunk size or remaining bytes
    if [ $REMAINING -lt $CHUNK_SIZE ]; then
        CURRENT_CHUNK_SIZE=$REMAINING
    else
        CURRENT_CHUNK_SIZE=$CHUNK_SIZE
    fi

    # Format chunk number with leading zeros
    PADDED_CHUNK=$(printf "%03d" $CHUNK)

    # Create output filename
    OUTPUT_FILE="${OUTPUT_DIR}/${BASENAME}_${TIMESTAMP}_part${PADDED_CHUNK}${EXTENSION}"

    echo -n "Creating chunk $CHUNK/$TOTAL_CHUNKS: $(basename "$OUTPUT_FILE") "

    # Use dd to copy chunk with minimal memory usage
    # bs=1M means 1MB buffer, count determines how many MB to copy
    BLOCK_COUNT=$((CURRENT_CHUNK_SIZE / 1024 / 1024))
    REMAINING_BYTES=$((CURRENT_CHUNK_SIZE % (1024 * 1024)))

    # Copy full megabytes
    if [ $BLOCK_COUNT -gt 0 ]; then
        dd if="$INPUT_FILE" of="$OUTPUT_FILE" bs=1M skip=$((BYTES_READ / 1024 / 1024)) count=$BLOCK_COUNT 2>/dev/null
    fi

    # Handle remaining bytes if any
    if [ $REMAINING_BYTES -gt 0 ]; then
        dd if="$INPUT_FILE" of="$OUTPUT_FILE" bs=1 skip=$((BYTES_READ + BLOCK_COUNT * 1024 * 1024)) count=$REMAINING_BYTES conv=notrunc oflag=append 2>/dev/null
    fi

    # If no full megabytes but has remaining bytes (file smaller than 1MB chunk)
    if [ $BLOCK_COUNT -eq 0 ] && [ $REMAINING_BYTES -gt 0 ]; then
        dd if="$INPUT_FILE" of="$OUTPUT_FILE" bs=1 skip=$BYTES_READ count=$CURRENT_CHUNK_SIZE 2>/dev/null
    fi

    # Verify chunk was created and get its size
    if [ -f "$OUTPUT_FILE" ]; then
        CHUNK_FILE_SIZE=$(stat -c%s "$OUTPUT_FILE")
        CHUNK_SIZE_MB=$((CHUNK_FILE_SIZE / 1024 / 1024))
        echo "✓ (${CHUNK_SIZE_MB}MB)"
    else
        echo "✗ Failed to create chunk"
        exit 1
    fi

    # Update counters
    BYTES_READ=$((BYTES_READ + CURRENT_CHUNK_SIZE))
    CHUNK=$((CHUNK + 1))
done

echo ""
echo "Split complete! Created $((CHUNK - 1)) parts in $OUTPUT_DIR"
echo "Memory usage: Minimal (streaming approach used)"

# Show total size verification
echo ""
echo "Verification:"
TOTAL_OUTPUT_SIZE=$(find "$OUTPUT_DIR" -name "${BASENAME}_${TIMESTAMP}_part*${EXTENSION}" -exec stat -c%s {} \; | awk '{sum+=$1} END {print sum}')
if [ "$TOTAL_OUTPUT_SIZE" = "$FILE_SIZE" ]; then
    echo "✓ Total output size matches input size: $FILE_SIZE bytes"
else
    echo "⚠ Size mismatch - Input: $FILE_SIZE, Output: $TOTAL_OUTPUT_SIZE"
fi

echo ""
echo "To rejoin the files later:"
echo "cat \"${OUTPUT_DIR}/${BASENAME}_${TIMESTAMP}_part\"*\"${EXTENSION}\" > \"${BASENAME}_rejoined${EXTENSION}\""
