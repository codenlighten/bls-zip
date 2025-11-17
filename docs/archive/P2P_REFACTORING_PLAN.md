# P2P Network Refactoring Plan

## Problem Statement

The current P2P implementation has the network event loop and handlers commented out due to Send/Sync trait issues with libp2p's `Swarm`. This makes nodes "deaf and mute" - they can mine blocks but cannot communicate with peers.

**Root Cause:** libp2p's `Swarm` contains types that aren't `Sync`, preventing it from being shared across async tasks using `Arc<RwLock<>>`.

## Solution: Channel-Based Architecture

### Architecture Overview

```
┌─────────────────┐         Commands          ┌──────────────────┐
│                 │ ────────────────────────> │                  │
│  Node (main.rs) │                           │  NetworkTask     │
│                 │ <──────────────────────── │  (owns Swarm)    │
└─────────────────┘         Events            └──────────────────┘
        │                                              │
        │                                              │
        v                                              v
┌─────────────────┐                          ┌──────────────────┐
│  NetworkHandle  │                          │  Swarm (libp2p)  │
│  (Clone + Send) │                          │  (!Send, !Sync)  │
└─────────────────┘                          └──────────────────┘
```

### Components

#### 1. `NetworkCommand` (p2p/src/service.rs)
Commands sent TO the network task:
- `BroadcastBlock(Arc<Block>)`
- `BroadcastTransaction(Arc<Transaction>)`
- `SendStatus { peer_id, height, best_hash }`
- `RequestBlocks { peer_id, start_height, count }`

#### 2. `NetworkEvent` (p2p/src/service.rs)
Events sent FROM the network task:
- `PeerConnected(PeerId)`
- `PeerDisconnected(PeerId)`
- `BlockReceived { peer_id, block }`
- `TransactionReceived { peer_id, transaction }`
- `StatusReceived { peer_id, height, best_hash }`
- `BlocksRequested { peer_id, start_height, count }`
- `ListeningOn(Multiaddr)`

#### 3. `NetworkHandle` (p2p/src/service.rs)
Clone-able, Send-able handle:
```rust
#[derive(Clone)]
pub struct NetworkHandle {
    command_tx: mpsc::UnboundedSender<NetworkCommand>,
}
```

Methods:
- `broadcast_block(Arc<Block>)`
- `broadcast_transaction(Arc<Transaction>)`
- `send_status(peer_id, height, best_hash)`
- `request_blocks(peer_id, start_height, count)`

#### 4. `NetworkNode::run()` (p2p/src/network.rs)
Takes ownership of `self` and runs the event loop:
```rust
pub async fn run(
    mut self,
    mut command_rx: UnboundedReceiver<NetworkCommand>,
) {
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
```

## Implementation Steps

### Step 1: Create Service Layer ✅
- [x] Create `p2p/src/service.rs` with `NetworkCommand`, `NetworkEvent`, `NetworkHandle`
- [x] Update `p2p/src/lib.rs` to export new types

### Step 2: Refactor NetworkNode
- [ ] Update `NetworkNode::new()` to return `(NetworkNode, NetworkHandle, EventReceiver)`
- [ ] Rewrite `NetworkNode::run()` to:
  - Take ownership of self
  - Accept `command_rx` channel
  - Process commands and swarm events in `select!` loop
- [ ] Add `handle_command()` method to process `NetworkCommand`s
- [ ] Update `handle_swarm_event()` to emit `NetworkEvent`s

### Step 3: Wire Up in Main
Update `node/src/main.rs`:

```rust
// Create network
let (network_node, network_handle, mut network_events) = NetworkNode::new(p2p_config)?;

// Spawn dedicated network task (this is Send-safe!)
tokio::spawn(async move {
    network_node.run(command_rx).await;
});

// Handle network events in main task
tokio::spawn(async move {
    while let Some(event) = network_events.recv().await {
        match event {
            NetworkEvent::BlockReceived { peer_id, block } => {
                // Apply block to blockchain
            }
            NetworkEvent::TransactionReceived { peer_id, transaction } => {
                // Add to mempool
            }
            // ... handle other events
        }
    }
});

// Use network_handle to send commands
network_handle.broadcast_block(Arc::new(block))?;
```

### Step 4: Update Mining Loop
```rust
async fn mining_loop(
    blockchain: Arc<RwLock<Blockchain>>,
    mempool: Arc<RwLock<Mempool>>,
    coinbase: [u8; 32],
    threads: usize,
    network: NetworkHandle, // Now takes NetworkHandle, not Option<Arc<RwLock<>>>
) {
    // ... mine block ...

    // Broadcast mined block
    network.broadcast_block(Arc::new(block))?;
}
```

### Step 5: Update Message Handling
Update `handle_swarm_event()` to emit events instead of directly modifying state:

```rust
async fn handle_swarm_event(&mut self, event: SwarmEvent<BoundlessBehaviourEvent>) {
    match event {
        SwarmEvent::Behaviour(BoundlessBehaviourEvent::Gossipsub(gossipsub::Event::Message {
            propagation_source: peer_id,
            message,
            ..
        })) => {
            if let Ok(msg) = bincode::deserialize::<Message>(&message.data) {
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
                    // ... other messages
                }
            }
        }
        // ... other swarm events
    }
}
```

## Testing Plan

### Unit Tests
```bash
cargo test --package boundless-p2p
```

### Integration Test: Two-Node Block Propagation
```bash
# Terminal 1: Start node A (mining)
./target/release/boundless-node --dev --mining --port 30333

# Terminal 2: Start node B (sync only)
./target/release/boundless-node --dev --port 30334 --bootnodes /ip4/127.0.0.1/tcp/30333

# Terminal 3: Verify node B receives blocks from node A
curl -X POST http://localhost:9934 -d '{"jsonrpc":"2.0","method":"chain_getBlockHeight","id":1}'
```

### Integration Test: Transaction Propagation
```bash
# Terminal 1: Submit transaction to node A
curl -X POST http://localhost:9933 -d '{
  "jsonrpc":"2.0",
  "method":"author_submitTransaction",
  "params":["<hex_encoded_tx>"],
  "id":1
}'

# Terminal 2: Check mempool on node B
# Should see transaction propagated
```

## Success Criteria

- [ ] `cargo build --release` succeeds with no commented-out network code
- [ ] Single node mines blocks successfully
- [ ] Two nodes connect via mDNS/bootnodes
- [ ] Block mined on node A appears on node B within 2 seconds
- [ ] Transaction submitted to node A propagates to node B
- [ ] No panics or deadlocks under normal operation
- [ ] `cargo test --all` passes

## Estimated Effort

- **Core refactoring:** 2-3 hours
- **Testing & debugging:** 1-2 hours
- **Documentation:** 30 minutes
- **Total:** 4-6 hours

## References

- libp2p Swarm docs: https://docs.rs/libp2p-swarm/latest/libp2p_swarm/struct.Swarm.html
- Tokio channels: https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html
- Similar pattern in Substrate: https://github.com/paritytech/substrate/tree/master/client/network
