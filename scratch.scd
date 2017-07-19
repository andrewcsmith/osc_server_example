// Test code for the supercollider side of interacting with the osc example

~addr = NetAddr.new("127.0.0.1", 6667);
~addr.sendMsg("/freq", 440);
~addr.sendMsg("/freq", 441.1);

