S2D2 Technical Notes
Core Concept
Spatial database that automatically shards based on 2D entity density. Clusters split when overloaded, merge when underutilized.

Architecture
Spatial Cluster: Database node responsible for a rectangular region of 2D space. Stores entities within its bounds.
Entity: Has an ID, position (x, y), and component data.
Global Lookup Table: Maps any coordinate to the cluster that owns it. Implemented with etcd for consistency.
Gateway: Stateless router. Queries lookup table to find which cluster owns a coordinate, then forwards requests there.

Data Storage Configurations
1. Lean Entity

Cluster stores only position and pointers to components
Component data lives in remote service
Low migration cost (just move pointers)
High query latency (network fetch per access)

2. Hybrid Tiering

Frequently accessed data stored locally (position, velocity)
Rarely accessed data stored remotely (inventory, achievements)
Balanced migration cost and query latency
Adapts to access patterns

3. Monolithic

All entity data colocated in cluster
Zero network latency for queries
High migration cost (move all data during split)
Best for high-frequency updates


Split Algorithm

Trigger: Cluster CPU above 80% or entity count exceeds threshold.
Steps:

Choose split axis (pick the longer dimension)
Find split point at median entity position
Spawn new cluster node
Migrate entities on far side of split to new cluster
Update lookup table atomically with new boundaries
Establish neighbor relationships between clusters

Example: Cluster covering X:[0,1000], Y:[0,1000] splits into two clusters covering X:[0,500] and X:[501,1000] respectively.

Merge Algorithm

Trigger: Both neighboring clusters below 20% CPU and combined entity count under threshold.
Steps:

Freeze writes on secondary cluster
Stream all entities to primary cluster
Update lookup table to remove secondary
Decommission secondary cluster node


Boundary Crossing

Problem: Entity moves from one cluster's territory into another. Must maintain consistency during handover.
Solution: Buffer Zone with Dual-Write
When entity is within 50 units of boundary:

Write to both current and target cluster
Current cluster remains authoritative for reads

When entity crosses boundary:

Flip read authority to target cluster
Keep copy in source cluster temporarily

When entity is 100+ units into new territory:

Delete copy from old cluster

Four States:

NORMAL: Single cluster owns entity
BUFFER: Dual-write active
CROSSED: Authority transferred
CLEANUP: Old copy deleted


Velocity Backstop

Problem: Fast-moving entity crosses buffer zone before replication completes. Target cluster doesn't have data when needed.
Detection: Trigger alarm when entity velocity multiplied by replication latency exceeds buffer width.

Response Actions:
Widen Buffer: Dynamically increase buffer zone size for this boundary
Simulation Brake: Pause entity movement until replication completes
Proxy Mode: Target cluster forwards queries back to source until sync completes

Interface 

-> Have subscribers to each shard, any changes in the shard will be sent over as state change, not the whole objects


Shared data 
-> Configure a shard that will contain common data and which can be derived based on different states in the other cluster and would have subscriber for state change


Test 

Will build a simulation in Bevy 