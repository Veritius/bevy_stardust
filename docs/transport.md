# Writing a transport layer
The transport layer is comprised of systems that run during the `TransportReadPackets` and `TransportSendPackets` schedules, responsible for things like compression, fragmentation, ordering, and encryption.

Systems in `TransportReadPackets` read incoming data (ie packets over UDP) and process them into octet strings for use by systems in the `ReadOctetStrings` schedule, which runs after `TransportReadPackets`. While `TransportReadPackets` systems can do things like mutating the world, they really shouldn't do anything beyond write octet strings and store information relevant to themselves.

Systems in `TransportWritePackets` reads octet strings created by systems in `WriteOctetStrings` and send them over the internet, performing necessary actions like fragmentation and encryption. Again, this schedule should not mutate the world.