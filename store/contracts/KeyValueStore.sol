// SPDX-License-Identifier: MIT

pragma solidity >=0.6.0 <0.9.0;

contract KeyValueStore {
    event unread(string id);

    // A discovery object holding a cipher and id of the user making transaction.
    // {id_{A}, c_{AB}}
    struct Discovery {
        string id;
        bytes cipher;
        bytes iv;
    }
    
    // A key-to-value map of the pairs (addr_{AB}, {id_{A}, c_{AB}})
    // addr_{AB} is the locally derived from the key loc_{AB}
    mapping(address => Discovery) public map;

    function Write(bytes memory cipher, bytes memory iv, address addr, string memory id) public {
        Discovery memory discovery; 
        discovery.id = id;
        discovery.cipher = cipher;
        discovery.iv = iv;
        map[addr] = discovery;
        emit unread(id);
    }

    function Read(address addr) public view returns(bytes memory, bytes memory){
        bytes memory stored_id = bytes(map[addr].id);
        require(stored_id.length != 0, "No discovery object found."); 
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