#!/usr/bin/env python3
import re

with open('node/src/main.rs', 'r') as f:
    content = f.read()

# 1. Update imports
content = content.replace(
    'use boundless_p2p::{NetworkNode, NetworkConfig, Message};',
    'use boundless_p2p::{NetworkNode, NetworkConfig, NetworkHandle, NetworkEvent};'
)

# 2. Update network initialization section
old_init = r'''    let network_handle = match NetworkNode::new\(p2p_config\) \{
        Ok\(\(network, mut events\)\) => \{
            info!\(🌐
