#!/usr/bin/env python3
"""
YAML Validation Script for fuzz-init

Validates all YAML files in the project, especially GitHub Actions workflows.
"""

import yaml
import os
import sys
from pathlib import Path

def validate_yaml_file(file_path):
    """Validate a single YAML file."""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            yaml.safe_load(f)
        return True, None
    except yaml.YAMLError as e:
        return False, str(e)
    except Exception as e:
        return False, f"Unexpected error: {e}"

def find_yaml_files(directory):
    """Find all YAML files in the given directory and subdirectories."""
    yaml_files = []
    for root, dirs, files in os.walk(directory):
        # Skip .git directory
        if '.git' in dirs:
            dirs.remove('.git')
        
        for file in files:
            if file.endswith(('.yml', '.yaml')):
                yaml_files.append(os.path.join(root, file))
    
    return yaml_files

def main():
    """Main validation function."""
    print("🔍 Validating YAML files in fuzz-init...")
    print("=" * 50)
    
    # Find all YAML files
    yaml_files = find_yaml_files('.')
    
    if not yaml_files:
        print("❌ No YAML files found")
        return 1
    
    # Validate each file
    valid_count = 0
    invalid_count = 0
    
    for file_path in sorted(yaml_files):
        is_valid, error = validate_yaml_file(file_path)
        
        if is_valid:
            print(f"✅ {file_path}")
            valid_count += 1
        else:
            print(f"❌ {file_path}")
            print(f"   Error: {error}")
            invalid_count += 1
    
    print("=" * 50)
    print(f"📊 Results: {valid_count} valid, {invalid_count} invalid")
    
    if invalid_count > 0:
        print("❌ Some YAML files have syntax errors")
        return 1
    else:
        print("✅ All YAML files are valid!")
        return 0

if __name__ == "__main__":
    sys.exit(main()) 