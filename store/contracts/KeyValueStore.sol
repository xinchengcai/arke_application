// SPDX-License-Identifier: MIT
// ---------------------------------------
// File: KeyValueStore.sol
// Date: 01 Sept 2023
// Description: Ethreum Contract
//              Private chat and ether transfer (Storage authority-side)
// ---------------------------------------
pragma solidity >=0.8.0 <0.9.0;

contract KeyValueStore {
    event unread(string[] id);

    struct Discovery {
        string[] id;
        bytes cipher;
        bytes iv;
    }
    mapping(address => Discovery) public map;

    function Write(bytes memory cipher, bytes memory iv, address addr, string[] memory id) public {
        Discovery memory discovery; 
        discovery.id = id;
        discovery.cipher = cipher;
        discovery.iv = iv;
        map[addr] = discovery;
        emit unread(id);
    }

    function Read(address addr) public view returns(bytes memory, bytes memory){
        return (map[addr].cipher, map[addr].iv);
    }

    function Delete(address addr) public {
        delete map[addr];
    }

    // Receive Ether in this contract
    receive() external payable {}

    // Send Ether to the specified address
    function sendEther(address payable to) public payable {
        require(address(this).balance >= msg.value, "Insufficient balance");
        to.transfer(msg.value); 
    }
}