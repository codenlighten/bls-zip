#!/usr/bin/env python3
import re

with open('p2p/src/network.rs', 'r') as f:
    content = f.read()

# Find and replace the gossipsub message handling
old_handling = '''            SwarmEvent::Behaviour(BoundlessBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                propagation_source: peer_id,
                message_id,
                message,
            })) => {
                // Deserialize and forward message
                match Message::from_bytes(&message.data) {
                    Ok(msg) => {
                        info!("📩 Received {} from {}", msg.message_type(), peer_id);
                        let _ = self.event_tx.send(NetworkEvent::MessageReceived {
                            peer_id,
                            message: msg,
                        });
                    }
                    Err(e) => {
                        warn!("Failed to deserialize message from {}: {}", peer_id, e);
                    }
                }
            }'''

new_handling = '''            SwarmEvent::Behaviour(BoundlessBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                propagation_source: peer_id,
                message_id,
                message,
            })) => {
                // Deserialize and forward message
                match Message::from_bytes(&message.data) {
                    Ok(msg) => {
                        use crate::service::NetworkEvent;
                        
                        info!("📩 Received {} from {}", msg.message_type(), peer_id);
                        
                        // Convert Message to NetworkEvent
                        match msg {
                            Message::NewBlock { block } => {
                                let _ = self.event_tx.send(NetworkEvent::BlockReceived {
                                    peer_id,
                                    block,
                                });
                            }
                            Message::NewTransaction { transaction } => {
                                let _ = self.event_tx.send(NetworkEvent::TransactionReceived {
                                    peer_id,
                                    transaction,
                                });
                            }
                            Message::Status { height, best_block_hash, .. } => {
                                let _ = self.event_tx.send(NetworkEvent::StatusReceived {
                                    peer_id,
                                    height,
                                    best_hash: best_block_hash,
                                });
                            }
                            Message::GetBlocks { start_height, count } => {
                                let _ = self.event_tx.send(NetworkEvent::BlocksRequested {
                                    peer_id,
                                    start_height,
                                    count,
                                });
                            }
                            Message::Blocks { blocks } => {
                                // Handle multiple blocks by emitting multiple BlockReceived events
                                for block in blocks {
                                    let _ = self.event_tx.send(NetworkEvent::BlockReceived {
                                        peer_id,
                                        block,
                                    });
                                }
                            }
                            Message::GetStatus => {
                                // Ignore or log - we could add StatusRequested event if needed
                                info!("Received GetStatus request from {}", peer_id);
                            }
                            Message::Ping { nonce } => {
                                // Respond with pong
                                info!("Received ping from {} (nonce: {})", peer_id, nonce);
                            }
                            Message::Pong { nonce } => {
                                info!("Received pong from {} (nonce: {})", peer_id, nonce);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to deserialize message from {}: {}", peer_id, e);
                    }
                }
            }'''

content = content.replace(old_handling, new_handling)

with open('p2p/src/network.rs', 'w') as f:
    f.write(content)

print("✅ Updated message handling to emit specific NetworkEvent variants")
