#!/bin/bash
while true;
 do curl http://localhost:8545 -X POST -H "Content-Type: application/json" --data '{"method":"eth_getLogs","params":[{"address": "0xe7f1725e7734ce288f8367e1bb143e90bb3f0512"}],"id":1,"jsonrpc":"2.0"}';
 sleep 1;
done
