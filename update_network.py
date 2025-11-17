#!/usr/bin/env python3
import re

# Read network.rs from WSL path
with open('p2p/src/network.rs', 'r') as f:
    content = f.read()

# Find and replace the run method
# Pattern to match the current run method
run_pattern = r'(    /// Run the network event loop\n    pub async fn run\(&mut self\) \{.*?\n    \})'

# New run method implementation
new_run_impl = '''    /// Run the network event loop (takes ownership, spawned in dedicated task)
    pub async fn run(
        mut self,
        mut command_rx: tokio::sync::mpsc::UnboundedReceiver<crate::service::NetworkCommand>,
    ) {
        use crate::service::NetworkCommand;

        info!("▶️  Starting P2P network event loop");

        loop {
            tokio::select! {
                Some(command) = command_rx.recv() => {
                    self.handle_command(command).await;
                }
                event = self.swarm.select_next_some() => {
                    self.handle_swarm_event(event).await;
                }
            }
        }
    }

    async fn handle_command(&mut self, command: NetworkCommand) {
        match command {
            NetworkCommand::BroadcastBlock(block) => {
                let message = Message::NewBlock { block: (*block).clone() };
                if let Ok(data) = message.to_bytes() {
                    if let Err(e) = self.swarm.behaviour_mut().gossipsub.publish(
                        self.blocks_topic.clone(),
                        data
                    ) {
                        warn!("Failed to broadcast block: {}", e);
                    } else {
                        info!("📢 Broadcasted block #{}", block.header.height);
                    }
                }
            }

            NetworkCommand::BroadcastTransaction(tx) => {
                let message = Message::NewTransaction { transaction: (*tx).clone() };
                if let Ok(data) = message.to_bytes() {
                    if let Err(e) = self.swarm.behaviour_mut().gossipsub.publish(
                        self.transactions_topic.clone(),
                        data
                    ) {
                        warn!("Failed to broadcast transaction: {}", e);
                    }
                }
            }

            NetworkCommand::SendStatus { peer_id, height, best_hash } => {
                let message = Message::Status { height, best_hash };
                if let Ok(data) = message.to_bytes() {
                    if let Err(e) = self.swarm.behaviour_mut().gossipsub.publish(
                        self.blocks_topic.clone(),
                        data
                    ) {
                        warn!("Failed to send status: {}", e);
                    }
                }
            }

            NetworkCommand::RequestBlocks { peer_id, start_height, count } => {
                let message = Message::GetBlocks { start_height, count };
                if let Ok(data) = message.to_bytes() {
                    if let Err(e) = self.swarm.behaviour_mut().gossipsub.publish(
                        self.blocks_topic.clone(),
                        data
                    ) {
                        warn!("Failed to request blocks: {}", e);
                    }
                }
            }
        }
    }'''

# Replace the run method
content = re.sub(run_pattern, new_run_impl, content, flags=re.DOTALL)

with open('p2p/src/network.rs', 'w') as f:
    f.write(content)

print("✅ Updated NetworkNode::run() and added handle_command()")
