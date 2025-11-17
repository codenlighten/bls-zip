#!/usr/bin/env python3
import re

with open('p2p/src/network.rs', 'r') as f:
    content = f.read()

# Update the NetworkNode::new() return type and implementation
# Find the new() function and update its return type

# Change the return type
content = content.replace(
    'pub fn new(config: NetworkConfig) -> anyhow::Result<(Self, mpsc::UnboundedReceiver<NetworkEvent>)>',
    'pub fn new(config: NetworkConfig) -> anyhow::Result<(Self, crate::service::NetworkHandle, mpsc::UnboundedReceiver<crate::service::NetworkEvent>)>'
)

# Find the return statement at the end of new() and update it
# Looking for: Ok((Self { ... }, event_rx))
# Replace with creating NetworkHandle

old_return_pattern = r'(\s+)(Ok\(\(\s*Self \{[^}]+\},\s*event_rx,?\s*\)\))'

def replace_return(match):
    indent = match.group(1)
    return f'''{indent}// Create command channel
{indent}let (command_tx, command_rx) = mpsc::unbounded_channel();
{indent}let network_handle = crate::service::NetworkHandle::new(command_tx);

{indent}Ok((
{indent}    Self {{
{indent}        swarm,
{indent}        peers: HashMap::new(),
{indent}        event_tx,
{indent}        blocks_topic,
{indent}        transactions_topic,
{indent}    }},
{indent}    network_handle,
{indent}    event_rx,
{indent}))'''

content = re.sub(old_return_pattern, replace_return, content)

# Also need to return command_rx from new() - wait, we need to modify the run signature
# Actually, let me check what we're returning

with open('p2p/src/network.rs', 'w') as f:
    f.write(content)

print("✅ Updated NetworkNode::new() to return NetworkHandle")
