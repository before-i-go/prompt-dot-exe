#!/bin/bash
#
# Gitignore Folder Content Archiver Bash Script for LLM consumption
# -----------------------
#
# PURPOSE:
# This script was created based on the requirement to archive the contents of a folder
# into a single text file to be consumed by LLMs
# Product Requirements Document (PRD) specified the need for a tool that could:
#   1. Create a text file with a timestamped name
#   2. Document the folder structure using the 'tree' command
#   3. Copy the contents of all files in the folder with proper labeling
#
# WHY THIS SOLUTION:
# We developed this bash script primarily to facilitate sharing code context with AI assistants
# like Grok or Google Gemini. These AI tools work best when provided with comprehensive context
# about a codebase, but have limitations on how files can be uploaded or referenced. This script
# solves that problem by creating a single, well-structured text file containing all relevant code.
#
# HOW IT WORKS:
# The script takes two parameters:
#   1. The target folder to archive
#   2. The output directory where the archive file will be saved
#
# It then:
#   - Creates a timestamped text file named after the target folder
#   - Records the directory structure using 'tree' or git-aware alternatives
#   - Copies the content of each file with clear demarcation of file boundaries
#   - Handles git repositories specially by respecting .gitignore rules
#
# BENEFITS:
#   - Optimized for sharing code context with AI assistants like Grok and Google Gemini
#   - Provides a complete snapshot of code without requiring version control
#   - Creates human-readable archives that can be easily searched and reviewed
#   - Works with both git and non-git repositories
#   - Preserves file paths and structure for context
#   - Timestamps archives for historical reference
#   - Lightweight alternative to binary archives (zip, tar) when content readability is important
#   - GITIGNORE ADVANTAGES:
#     * Automatically excludes build artifacts, dependencies, and other non-essential files
#     * Prevents large binary files from bloating the output text file
#     * Focuses the AI assistant on relevant source code by filtering out temporary files
#     * Respects project-specific exclusions already defined by developers
#     * Reduces noise from node_modules, target directories, and other dependency folders
#     * Maintains consistency with what developers consider important in the codebase


#!/bin/bash

# Check for correct number of arguments
if [ $# -ne 2 ]; then
    echo "Usage: $0 <target_folder> <output_dir>"
    exit 1
fi

# Assign arguments to variables
target_folder="$1"
output_dir="$2"

# Validate that target_folder is a directory
if [ ! -d "$target_folder" ]; then
    echo "Error: $target_folder is not a directory."
    exit 1
fi

# Generate timestamp in YYYYMMDDHHMMSS format
timestamp=$(date +%Y%m%d%H%M%S)

# Get folder name from target_folder path
folder_name=$(basename "$target_folder")

# Construct output file path
output_file="$output_dir/${folder_name}-${timestamp}.txt"

# Ensure output directory exists
mkdir -p "$output_dir"

# Check if git is available and if the target folder is a git repository
is_git_repo=false
if command -v git >/dev/null 2>&1 && git -C "$target_folder" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    is_git_repo=true
    echo "Git repository detected. Will respect .gitignore rules." >> "$output_file"
    # Find the root of the git repository
    git_root=$(git -C "$target_folder" rev-parse --show-toplevel)
else
    echo "Not a git repository or git not available. Will process all files." >> "$output_file"
fi

# Compute relative path from git_root to target_folder
if [ "$is_git_repo" = true ]; then
    rel_path="${target_folder#$git_root}"
    if [ -z "$rel_path" ]; then
        rel_path="."
    else
        rel_path="${rel_path#/}"
    fi
fi

# Append tree command output to output file
echo "Directory structure:" >> "$output_file"
if [ "$is_git_repo" = true ]; then
    # Use git ls-files for tree-like output that includes tracked and untracked files (respecting .gitignore)
    (cd "$git_root" && { git ls-files "$rel_path"; git ls-files -o --exclude-standard "$rel_path"; } | sort -u | sed -e 's/[^\/]*\//│   /g' -e 's/[^\/]*$/├── &/') >> "$output_file"
else
    tree "$target_folder" >> "$output_file"
fi
echo "" >> "$output_file"

# Process files based on git status if it's a git repo
if [ "$is_git_repo" = true ]; then
    echo "Processing tracked and untracked files (respecting .gitignore)..." >> "$output_file"
    echo "" >> "$output_file"
    # Get list of tracked and untracked files (excluding ignored files)
    (cd "$git_root" && { git ls-files "$rel_path"; git ls-files -o --exclude-standard "$rel_path"; } | sort -u) | while IFS= read -r file; do
        if [ -f "$git_root/$file" ]; then
            abs_file="$git_root/$file"
            echo "Absolute path: $abs_file" >> "$output_file"
            if file -b --mime-type "$abs_file" | grep -q "^text/"; then
                echo "<text starts>" >> "$output_file"
                cat "$abs_file" >> "$output_file"
                echo "<text ends>" >> "$output_file"
            else
                echo "Binary file, content not included." >> "$output_file"
            fi
            echo "" >> "$output_file"
        fi
    done
else
    # Process all files in target_folder and subfolders
    echo "Processing all files..." >> "$output_file"
    echo "" >> "$output_file"
    find "$target_folder" -type f -print0 | while IFS= read -r -d '' file; do
        echo "Absolute path: $file" >> "$output_file"
        if file -b --mime-type "$file" | grep -q "^text/"; then
            echo "<text starts>" >> "$output_file"
            cat "$file" >> "$output_file"
            echo "<text ends>" >> "$output_file"
        else
            echo "Binary file, content not included." >> "$output_file"
        fi
        echo "" >> "$output_file"
    done
fi

echo "Output saved to: $output_file"

## command to make the script executable
# chmod +x copy-paste-20250506.sh



## example of how to invoke the script
# ./copy-paste-20250506.sh /home/webapp-folder /home/code-txt-output-folder


## /home/amuldotexe/Desktop/GitHub202410/pensieve2024/A01Play2025Q1/PublishSoftware/copy-paste-20250506.sh /home/amuldotexe/Desktop/GitHub202410/wandlorelabs2025 /home/amuldotexe/a01_logs


## /home/amuldotexe/Desktop/GitHub202410/pensieve2024/A01Play2025Q1/PublishSoftware/copy-paste-20250506.sh /home/amuldotexe/AndroidStudioProjects/Arithmancy /home/amuldotexe/a01_logs

## /home/amuldotexe/Desktop/GitHub202410/pensieve2024/A01Play2025Q1/PublishSoftware/copy-paste-20250506.sh /home/amuldotexe/Desktop/GitHub202410/that-in-rails/rails01 /home/amuldotexe/a01_logs


# while IFS= read -r -d '' folder_name; do
#     bash /home/amuldotexe/Desktop/GitHub202410/pensieve2024/A01Play2025Q1/PublishSoftware/copy-paste-20250506.sh "$folder_name" /home/amuldotexe/a01_logs
# done < <(find /home/amuldotexe/Downloads/fm-transcripts-master -mindepth 1 -maxdepth 1 -type d -not -name ".*" -print0)