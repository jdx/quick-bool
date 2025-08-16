#!/bin/bash

# Script to update README with benchmark results and Mermaid charts
# This script integrates generated content directly into the README

set -e

echo "Updating README with benchmark results and Mermaid charts..."

# Function to update the README with generated content
update_readme() {
    local readme_file="README.md"
    local temp_readme=$(mktemp)
    
               # Check if the generated files exist
           if [ ! -f "tmp/md/first_access_chart.md" ] || [ ! -f "tmp/md/cached_access_chart.md" ]; then
               echo "Error: Generated chart files not found. Run chart generation first."
               exit 1
           fi
    
    # Find the benchmark section and replace it
    local in_benchmark_section=false
    local benchmark_started=false
    
    while IFS= read -r line; do
        if [[ "$line" == "## Benchmarks" ]]; then
            in_benchmark_section=true
            benchmark_started=true
            
            # Write the new benchmark section with Mermaid charts
            echo "## Benchmarks" >> "$temp_readme"
            echo >> "$temp_readme"

            echo "### First Access Performance" >> "$temp_readme"
            cat tmp/md/first_access_chart.md >> "$temp_readme"
            echo >> "$temp_readme"
            echo "### Cached Access Performance" >> "$temp_readme"
            cat tmp/md/cached_access_chart.md >> "$temp_readme"
            echo >> "$temp_readme"
            continue
        fi
        
        if [[ "$in_benchmark_section" == true ]]; then
            # Skip lines until we hit the next section
            if [[ "$line" =~ ^##[[:space:]] && "$benchmark_started" == true ]]; then
                in_benchmark_section=false
                echo "$line" >> "$temp_readme"
            fi
            continue
        fi
        
        echo "$line" >> "$temp_readme"
    done < "$readme_file"
    
    # Replace the original README
    mv "$temp_readme" "$readme_file"
    echo "README updated successfully with Mermaid charts!"
}

# Main execution
main() {
    echo "Starting README update process..."
    
    # Check if we're in the right directory
    if [ ! -f "Cargo.toml" ] || [ ! -f "README.md" ]; then
        echo "Error: Must be run from the project root directory"
        exit 1
    fi
    
    # Update the README
    update_readme
    
    echo "README update complete!"
}

# Run the main function
main "$@"
