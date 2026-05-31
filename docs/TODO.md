1. When master is killed it should kill all the zone node. (Done)
2. Integrate Spatial awareness into master and make sure it creates and divides node accordiningly. (For the input should ask for tenant id, bound box coordinates and number of servers)
3. take coordinate as an input and provide the udp or tcp connection detail based on that 
4. Research into proper ways to share tcp/udp connection details 
5. Zone node can accept subscribers and they are passed on any change to the data
6. Zone node has a running environment for rust 
7. Zone node has a entity initiation code environment 
8. Create a migration node, in zone node have a dummy command to copy, migration node can reach out to the zone node
9. Based on command copy the data to other zone node. 
10. Zone node sends out location of each entity to migration node 
11. Make the system multi tenanted and take care of mutex rules
12. Create locally deployed containers and run the program inside. 
13. Automate creation through kubernetese. 
14. Attach ability to master node to automate kubernetese operations.


Will keep adding new things to do here