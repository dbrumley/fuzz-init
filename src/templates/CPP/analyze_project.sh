#!/bin/bash
# C++ Project Analysis Script for Fuzzing Target Discovery
# Analyzes C++ projects to suggest high-value fuzzing targets
# Usage: analyze_project.sh <project_directory> <output_json_file>

set -e

PROJECT_DIR="$1"
OUTPUT_JSON="$2"

if [ -z "$PROJECT_DIR" ] || [ -z "$OUTPUT_JSON" ]; then
    echo "Usage: $0 <project_directory> <output_json_file>" >&2
    exit 1
fi

if [ ! -d "$PROJECT_DIR" ]; then
    echo "Error: Project directory '$PROJECT_DIR' does not exist" >&2
    exit 1
fi

# Function to safely escape JSON strings
escape_json() {
    echo "$1" | sed 's/\\/\\\\/g; s/"/\\"/g; s/	/\\t/g; s/$/\\n/' | tr -d '\n'
}

# Initialize variables
SUGGESTED_TARGETS=""
PRIMARY_TARGET=""
EXAMPLE_CODE=""

# Search patterns for high-value fuzzing targets
# Pattern format: "grep_pattern:description:priority"
PATTERNS=(
    "parseXml.*(:XML parsing - primary entry point:true"
    "parseDocument.*(:Document parsing - complex processing:true"
    "parse.*XML.*(:XML document parsing - high bug potential:true"
    "parse.*string:String parsing - untrusted input:false"
    "parse.*stream:Stream parsing - complex data:false"
    "decode.*(:Data decoding - format processing:false"
    "read.*string:String reading - input processing:false"
    "fromString.*(:String conversion - parsing logic:false"
)

# Look for headers in common locations
SEARCH_DIRS=""
for dir in "include" "src" "."; do
    if [ -d "$PROJECT_DIR/$dir" ]; then
        SEARCH_DIRS="$SEARCH_DIRS $PROJECT_DIR/$dir"
    fi
done

# Extract function patterns with priorities
find_targets() {
    local found_any=false
    
    for pattern_spec in "${PATTERNS[@]}"; do
        # Parse pattern spec: "grep_pattern:description:priority"
        IFS=':' read -r pattern description is_primary <<< "$pattern_spec"
        
        # Search for pattern in C++ headers and source files
        matches=$(find $SEARCH_DIRS -type f \( -name "*.h" -o -name "*.hpp" -o -name "*.hxx" -o -name "*.cpp" -o -name "*.cc" \) \
            -exec grep -l "$pattern" {} \; 2>/dev/null | head -3)
        
        if [ -n "$matches" ]; then
            # Extract actual function names from the matches
            for file in $matches; do
                # Get function names from this file
                functions=$(grep -o "$pattern" "$file" 2>/dev/null | sed 's/\s*(.*//' | head -2)
                
                for func in $functions; do
                    if [ -n "$func" ] && [[ "$func" =~ ^[a-zA-Z_][a-zA-Z0-9_:]*$ ]]; then
                        found_any=true
                        
                        # Set primary target if this is high priority
                        if [ "$is_primary" = "true" ] && [ -z "$PRIMARY_TARGET" ]; then
                            PRIMARY_TARGET="$func"
                        fi
                        
                        # Escape function name and description for JSON
                        func_escaped=$(escape_json "$func")
                        desc_escaped=$(escape_json "$description")
                        file_basename=$(basename "$file")
                        file_escaped=$(escape_json "$file_basename")
                        
                        # Add to targets list (avoid duplicates)
                        if ! echo "$SUGGESTED_TARGETS" | grep -q "\"$func_escaped\""; then
                            if [ -n "$SUGGESTED_TARGETS" ]; then
                                SUGGESTED_TARGETS="$SUGGESTED_TARGETS,"
                            fi
                            SUGGESTED_TARGETS="$SUGGESTED_TARGETS
    {
      \"function\": \"$func_escaped\",
      \"description\": \"$desc_escaped\",
      \"primary\": $is_primary,
      \"file\": \"$file_escaped\"
    }"
                        fi
                    fi
                done
            done
        fi
    done
    
    echo $found_any
}

# Project-specific enhancements
enhance_for_known_projects() {
    local project_name=$(basename "$PROJECT_DIR")
    
    case "$project_name" in
        "libadm"|"adm")
            # Special handling for libadm - we know the good targets
            if [ -f "$PROJECT_DIR/include/adm/parse.hpp" ]; then
                PRIMARY_TARGET="adm::parseXml"
                SUGGESTED_TARGETS='
    {
      "function": "adm::parseXml",
      "description": "XML document parsing - primary ADM entry point",
      "primary": true,
      "file": "parse.hpp"
    },
    {
      "function": "adm::parseFrameHeader", 
      "description": "Frame header parsing - metadata processing",
      "primary": false,
      "file": "parse.hpp"
    },
    {
      "function": "parseAudioObjectId",
      "description": "Audio object ID parsing - identifier validation",
      "primary": false,
      "file": "audio_object_id.hpp"
    }'
                
                EXAMPLE_CODE='// TARGET: Parse ADM XML document - high crash potential!
        auto document = adm::parseXml(input);
        
        // Force evaluation of parsed content
        if (document) {
            auto programmes = document->getElements<adm::AudioProgramme>();
            (void)programmes; // Suppress unused warning
        }'
                return
            fi
            ;;
    esac
}

# Run analysis
echo "Analyzing C++ project: $PROJECT_DIR" >&2

# Try project-specific analysis first
enhance_for_known_projects

# If no project-specific targets found, run generic analysis
if [ -z "$SUGGESTED_TARGETS" ]; then
    found=$(find_targets)
    if [ "$found" = "false" ]; then
        echo "No obvious parsing functions found. Using generic suggestions." >&2
        SUGGESTED_TARGETS='
    {
      "function": "parse_input",
      "description": "Generic input parsing - replace with your function",
      "primary": true,
      "file": "your_header.h"
    }'
    fi
fi

# Generate default example code if not set
if [ -z "$EXAMPLE_CODE" ]; then
    if [ -n "$PRIMARY_TARGET" ]; then
        EXAMPLE_CODE="// TODO: Replace with actual $PRIMARY_TARGET call
// Example:
// auto result = $PRIMARY_TARGET(input);
// if (result) { (void)result->some_method(); }"
    else
        EXAMPLE_CODE="// TODO: Replace with your target function call
// Examples:
// auto result = your_project::parse_document(input);
// your_project::process_data(data, size);"
    fi
fi

# Escape example code for JSON
EXAMPLE_CODE_ESCAPED=$(escape_json "$EXAMPLE_CODE")

# Generate JSON output
cat > "$OUTPUT_JSON" << EOF
{
  "suggested_targets": [$SUGGESTED_TARGETS
  ],
  "suggested_example_code": "$EXAMPLE_CODE_ESCAPED",
  "analysis_metadata": {
    "project_dir": "$(escape_json "$PROJECT_DIR")",
    "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "analyzer": "cpp_project_analyzer_v1.0"
  }
}
EOF

echo "Analysis complete. Found $(echo "$SUGGESTED_TARGETS" | grep -c '"function"') target suggestions." >&2
echo "Results written to: $OUTPUT_JSON" >&2