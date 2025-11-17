#!/usr/bin/env python3

with open('p2p/src/network.rs', 'r') as f:
    lines = f.readlines()

# Remove the leftover NetworkEvent enum fragment (lines with comma and NewListenAddr)
# We need to find and remove these specific lines
cleaned_lines = []
skip_next = 0

for i, line in enumerate(lines):
    # Skip the leftover fragment
    if i < len(lines) - 1 and line.strip() == ',' and 'NewListenAddr' in lines[i+1]:
        skip_next = 3  # Skip comma line, comment line, NewListenAddr line, and closing brace
        continue
    
    if skip_next > 0:
        skip_next -= 1
        continue
    
    cleaned_lines.append(line)

with open('p2p/src/network.rs', 'w') as f:
    f.writelines(cleaned_lines)

print("✅ Fixed syntax error in network.rs")
